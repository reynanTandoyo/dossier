//! The `approve` command: a person at a terminal approves, edits, rejects or skips drafts, one by
//! one (R-DRAFT-4, D5, D6). Each draft is shown in full on standard error and the person answers:
//! `a` or `approve` approves it, `e` or `edit` opens it in the editor first (then it is shown and
//! asked about again), `r` or `reject` rejects it, and anything else skips it. An LLM cannot
//! approve, and nothing is asked without a terminal. The answers are written together, in one
//! transaction.

use crate::approval;
use crate::cli::Global;
use crate::cmd::edit;
use crate::ctx::Ctx;
use crate::entry::rules;
use crate::entry::{Entry, Status};
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

/// Shows one draft and records what the person answers for it (R-DRAFT-4): `a` approves it, `e`
/// edits it first and asks again, `r` rejects it, and anything else skips it. Anything that is not
/// a draft, and a draft that is missing something, is skipped with a warning and no question is
/// asked. An edit is kept when the draft is then skipped; an edit that fails leaves the draft as it
/// was, with a warning, and is asked about again. An ID given a second time is left alone with a
/// warning.
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
    let mut current = item.entry.clone();
    show(&current.text());
    // Whether the person has edited the draft: an edited draft that is then skipped keeps its edit.
    let mut edited = false;
    loop {
        let problems = rules::problems(&current);
        if !problems.is_empty() {
            let what: Vec<String> = problems.into_iter().map(|problem| problem.what).collect();
            keep_edit(tx, item, &current, edited);
            tally.verdict(&id, Verdict::Skipped);
            tally.warn(format!("{id} cannot be approved yet: {}", what.join("; ")));
            return Ok(());
        }
        let answer = term::ask(&format!(
            "{id}: approve, edit first, reject or skip? (a/e/r/s) "
        ))?;
        let verdict = match answer.to_lowercase().as_str() {
            "a" | "approve" => {
                approve(tx, item, &current, who)?;
                Verdict::Approved
            }
            "r" | "reject" => {
                reject(tx, item, &current, who);
                Verdict::Rejected
            }
            "e" | "edit" => {
                // A failed edit (the editor cannot start or ends with an error, or the saved text is
                // refused) ends only this edit: the draft stays as it was and is asked about again.
                match edit_draft(tx.time(), who, &id, &mut current) {
                    Ok(true) => {
                        edited = true;
                        show(&current.text());
                    }
                    Ok(false) => {}
                    Err(error) => tally.warn(format!("{id}: {error}; the draft is unchanged")),
                }
                continue;
            }
            _ => {
                keep_edit(tx, item, &current, edited);
                Verdict::Skipped
            }
        };
        tally.verdict(&id, verdict);
        return Ok(());
    }
}

/// Opens the draft in the editor, as `edit` does (R-EDIT-2), and returns whether its text changed.
/// A changed text is checked as `edit` checks it, gets the history line of the edit before the
/// approval, and replaces `current`. When the editor saves the text as it was, nothing changes.
fn edit_draft(time: &str, who: &Who, id: &str, current: &mut Entry) -> Result<bool> {
    let before = current.text();
    let after = edit::edit_in_temp_file(&edit::editor_program(), id, &before)?;
    if after == before {
        return Ok(false);
    }
    let mut edited = Entry::parse(&after)?;
    edit::refuse_tool_lines(current, &edited)?;
    edited.add_history(time, who, "changed", "edited before approval");
    *current = edited;
    Ok(true)
}

/// Queues the draft as the person left it when it was edited: a draft skipped after an edit, or an
/// edited draft that cannot be approved yet, keeps its edit and stays a draft. Without an edit
/// nothing is queued.
fn keep_edit(tx: &mut Tx, item: &Item, current: &Entry, edited: bool) {
    if edited {
        tx.put(
            item.path.clone(),
            Some(item),
            current.text(),
            change(current.id(), "changed"),
        );
    }
}

/// Queues the approval of `entry`, the draft as the person left it (edited or not): status, who
/// approved it and when, and the history line. The checksum of an approved lesson, how-to or tool
/// is recorded now, unless this is a dry run (R-LIFE-6).
fn approve(tx: &mut Tx, item: &Item, entry: &Entry, who: &Who) -> Result<()> {
    let time = tx.time().to_string();
    let mut entry = entry.clone();
    entry.set_field("status", Status::Approved.name());
    entry.set_field("approved", &format!("{time} by {}", who.person));
    entry.set_field("valid-from", &time);
    entry.add_history(&time, who, "approved", "");
    let id = entry.id();
    let text = entry.text();
    if entry.kind().is_some_and(approval::guarded) && !tx.ctx().dry_run {
        approval::record(&tx.ctx().state, &id, text.as_bytes())?;
    }
    tx.put(item.path.clone(), Some(item), text, change(id, "approved"));
    Ok(())
}

/// Queues the rejection of `entry`, the draft as the person left it: status and the history line.
fn reject(tx: &mut Tx, item: &Item, entry: &Entry, who: &Who) {
    let time = tx.time().to_string();
    let mut entry = entry.clone();
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
