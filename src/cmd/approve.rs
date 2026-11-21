//! The `approve` command: a person at a terminal approves or rejects drafts, one by one (R-DRAFT-4,
//! D5, D6). Each draft is shown in full on standard error and the person answers: `a` or `approve`
//! approves it, `r` or `reject` rejects it, and anything else skips it. An LLM cannot approve, and
//! nothing is asked without a terminal. The answers are written together, in one transaction.

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::rules;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names::Who;
use crate::output::Output;
use crate::store::{self, Item};
use crate::term;
use crate::tx::{self, Change, Tx};
use serde_json::json;
use std::io::Write;

/// The arguments of `approve`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The drafts to approve, such as LES-004; each is shown and asked about in turn
    #[arg(required = true, value_name = "ID")]
    pub ids: Vec<String>,
}

/// What became of one ID.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Approved,
    Rejected,
    Skipped,
}

impl Verdict {
    /// The word of the output line for this verdict.
    fn word(self) -> &'static str {
        match self {
            Verdict::Approved => "approved",
            Verdict::Rejected => "rejected",
            Verdict::Skipped => "skipped",
        }
    }
}

/// The verdicts in the order of the IDs given, and the warnings of the run.
#[derive(Default)]
struct Tally {
    verdicts: Vec<(String, Verdict)>,
    warnings: Vec<String>,
}

impl Tally {
    /// Whether `id` already has a verdict.
    fn has(&self, id: &str) -> bool {
        self.verdicts.iter().any(|(done, _)| done == id)
    }

    /// Records the verdict of `id`.
    fn verdict(&mut self, id: &str, verdict: Verdict) {
        self.verdicts.push((id.to_string(), verdict));
    }

    /// Adds a warning.
    fn warn(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Runs `approve`: checks who acts and that a person is at a terminal, then asks about each draft
/// and writes the answers in one transaction. The first line of the output is `<ID>: approved`,
/// `<ID>: rejected` or `<ID>: skipped` for each ID, in the order given.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "approve")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("only a person can approve; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    tx::write(&ctx, |tx| {
        // Every ID is read before the first question, so that an unknown ID stops the command
        // before anyone is asked anything.
        let items = args
            .ids
            .iter()
            .map(|id| store::get(&tx.ctx().dossier, id))
            .collect::<Result<Vec<Item>>>()?;
        let mut tally = Tally::default();
        for item in &items {
            review(tx, item, &who, &mut tally)?;
        }
        let output = Output::with(text_of(&tally), data_of(&tally));
        Ok(tally
            .warnings
            .into_iter()
            .fold(output, |output, warning| output.warn(warning)))
    })
}

/// Shows one draft and records what the person answers for it. Anything that is not a draft, and
/// a draft that is missing something, is skipped with a warning and no question is asked. An ID
/// given a second time is left alone with a warning.
fn review(tx: &mut Tx, item: &Item, who: &Who, tally: &mut Tally) -> Result<()> {
    let entry = &item.entry;
    let id = entry.id();
    if tally.has(&id) {
        tally.warn(format!("{id} is given more than once; it is handled once"));
        return Ok(());
    }
    match entry.status() {
        Some(Status::Draft) => {}
        Some(status) => {
            tally.verdict(&id, Verdict::Skipped);
            tally.warn(format!("{id} is {}, not a draft; skipped", status.name()));
            return Ok(());
        }
        None => {
            tally.verdict(&id, Verdict::Skipped);
            tally.warn(format!("{id} has no valid status; skipped"));
            return Ok(());
        }
    }
    show(&entry.text());
    let problems = rules::problems(entry);
    if !problems.is_empty() {
        let what: Vec<String> = problems.into_iter().map(|problem| problem.what).collect();
        tally.verdict(&id, Verdict::Skipped);
        tally.warn(format!("{id} cannot be approved yet: {}", what.join("; ")));
        return Ok(());
    }
    let answer = term::ask(&format!("{id}: approve, reject or skip? (a/r/s) "))?;
    let verdict = match answer.to_lowercase().as_str() {
        "a" | "approve" => {
            approve(tx, item, who)?;
            Verdict::Approved
        }
        "r" | "reject" => {
            reject(tx, item, who);
            Verdict::Rejected
        }
        _ => Verdict::Skipped,
    };
    tally.verdict(&id, verdict);
    Ok(())
}

/// Queues the approval of a draft: status, who approved it and when, and the history line. The
/// checksum of an approved lesson is recorded now, unless this is a dry run (R-LIFE-6).
fn approve(tx: &mut Tx, item: &Item, who: &Who) -> Result<()> {
    let time = tx.time().to_string();
    let mut entry = item.entry.clone();
    entry.set_field("status", Status::Approved.name());
    entry.set_field("approved", &format!("{time} by {}", who.person));
    entry.set_field("valid-from", &time);
    entry.add_history(&time, who, "approved", "");
    let id = entry.id();
    let text = entry.text();
    if entry.kind() == Some(Kind::Lesson) && !tx.ctx().dry_run {
        approval::record(&tx.ctx().state, &id, text.as_bytes())?;
    }
    tx.put(item.path.clone(), Some(item), text, change(id, "approved"));
    Ok(())
}

/// Queues the rejection of a draft: status and the history line.
fn reject(tx: &mut Tx, item: &Item, who: &Who) {
    let time = tx.time().to_string();
    let mut entry = item.entry.clone();
    entry.set_field("status", Status::Rejected.name());
    entry.add_history(&time, who, "rejected", "");
    let id = entry.id();
    let text = entry.text();
    tx.put(item.path.clone(), Some(item), text, change(id, "rejected"));
}

/// The journal line of a change of `id` with this event.
fn change(id: String, event: &str) -> Change {
    Change {
        id,
        event: event.to_string(),
        detail: String::new(),
    }
}

/// Writes the text of a draft to standard error, the review screen. A failed write is ignored:
/// the answer is what matters.
fn show(text: &str) {
    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "{text}");
    if !text.ends_with('\n') {
        let _ = writeln!(stderr);
    }
    let _ = writeln!(stderr);
}

/// The output lines: `<ID>: <verdict>` for each ID, in the order given.
fn text_of(tally: &Tally) -> String {
    tally
        .verdicts
        .iter()
        .map(|(id, verdict)| format!("{id}: {}", verdict.word()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The data for `--json`: the IDs of each verdict, in the order given.
fn data_of(tally: &Tally) -> serde_json::Value {
    let of = |wanted: Verdict| -> Vec<&str> {
        tally
            .verdicts
            .iter()
            .filter(|(_, verdict)| *verdict == wanted)
            .map(|(id, _)| id.as_str())
            .collect()
    };
    json!({
        "approved": of(Verdict::Approved),
        "rejected": of(Verdict::Rejected),
        "skipped": of(Verdict::Skipped),
    })
}
