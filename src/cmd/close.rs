//! The `close` command (docs/design.md section 12, R-TEAM-4, D28): a decider at a terminal goes
//! through a finished project's knowledge. Each approved finding, lesson and how-to that has no
//! `promoted` field and no `promote: no` is shown in full on standard error, and the person
//! answers `p` (promoted: copied to the team dossier by `promote`), `k` (kept in the project:
//! `promote: no` with a history line) or anything else (left for later: nothing written).
//!
//! Each answer is written when it is given, so the answers survive a run that stops early, and a
//! second run asks only about what is left.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::cmd::promote;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names::{self, Who};
use crate::output::Output;
use crate::store::{self, Item};
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;
use std::io::Write;

/// The field that says an entry was promoted to the team dossier (format.md §23).
const PROMOTED: &str = "promoted";

/// The field that says an entry stays in the project (format.md §23).
const PROMOTE: &str = "promote";

/// The history event and the journal event of an entry kept in the project.
const CHANGED: &str = "changed";

/// The detail of that change: the history line and the journal line both name it.
const KEPT: &str = "kept in the project";

/// The arguments of `close`: none, the entries are asked about one by one.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// What the answers did, in the order of the IDs, and the warnings of the run.
#[derive(Default)]
struct Tally {
    promoted: Vec<String>,
    kept: Vec<String>,
    left: Vec<String>,
    warnings: Vec<String>,
}

/// Runs `close`: checks who acts and that a person at a terminal may decide, then asks about each
/// waiting entry in ID order. A `p` promotes the entry (a team dossier must be linked, and the
/// confidential check of `promote` must pass; a refusal stops the run, and the answers before it
/// are kept). The output is `<n> promoted, <m> kept, <k> left for later`; the data lists the IDs
/// of each answer.
pub fn run(_args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "close")?;
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("close is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    if !names::may_decide(&ctx.cfg, &who.person) {
        return Err(Error::new(
            Code::Refused,
            format!("{} is not in deciders", who.person),
        ));
    }

    let (items, warnings) = store::all(&ctx.dossier);
    let ids: Vec<String> = items
        .iter()
        .filter(|item| waiting(item))
        .map(|item| item.entry.id())
        .collect();
    let mut tally = Tally {
        warnings,
        ..Tally::default()
    };
    for id in ids {
        // Read again, so that what is shown and checked is what is stored now.
        let item = store::get(&ctx.dossier, &id)?;
        say(&item.entry.text());
        say("");
        let answer = term::ask(&format!(
            "{id}: promote, keep in the project, or later? (p/k/l) "
        ))?;
        match answer.to_lowercase().as_str() {
            "p" => {
                let team_id = promote::promote_one(&ctx, &id, None)?;
                say(&format!("{id}: promoted as team:{team_id}"));
                tally.promoted.push(id);
            }
            "k" => {
                let output = keep(&ctx, &who, &id)?;
                tally.warnings.extend(output.warnings);
                say(&format!("{id}: {KEPT}"));
                tally.kept.push(id);
            }
            _ => {
                say(&format!("{id}: left for later"));
                tally.left.push(id);
            }
        }
    }

    let text = format!(
        "{} promoted, {} kept, {} left for later",
        tally.promoted.len(),
        tally.kept.len(),
        tally.left.len()
    );
    let data = json!({
        "promoted": tally.promoted,
        "kept": tally.kept,
        "left": tally.left,
    });
    let output = Output::with(text, data);
    Ok(tally
        .warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// Whether an entry waits for the decision: an approved finding, lesson or how-to with no
/// `promoted` field and no `promote: no` (format.md §23).
fn waiting(item: &Item) -> bool {
    let entry = &item.entry;
    let kind = entry.kind().or_else(|| Kind::of_id(&entry.id()));
    matches!(kind, Some(Kind::Finding | Kind::Lesson | Kind::Howto))
        && entry.status() == Some(Status::Approved)
        && entry.field(PROMOTED).is_none()
        && entry
            .field(PROMOTE)
            .is_none_or(|value| !value.trim().eq_ignore_ascii_case("no"))
}

/// Keeps an entry in the project: `promote: no`, with the history line `changed — kept in the
/// project`. An approved lesson, how-to or tool whose approval still held keeps it: the new line is
/// not a change of content (as in `promote`), so its checksum is recorded again, unless this is a
/// dry run.
fn keep(ctx: &Ctx, who: &Who, id: &str) -> Result<Output> {
    tx::write(ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, id)?;
        let kind = item.entry.kind().or_else(|| Kind::of_id(id));
        let held = kind.is_some_and(approval::guarded)
            && matches!(
                approval::mark(&tx.ctx().state, id, &item.raw),
                Mark::Unchanged
            );
        let mut entry = item.entry.clone();
        entry.set_field(PROMOTE, "no");
        entry.add_history(tx.time(), who, CHANGED, KEPT);
        let text = entry.text();
        if held && !tx.ctx().dry_run {
            approval::record(&tx.ctx().state, id, text.as_bytes())?;
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.to_string(),
                event: CHANGED.to_string(),
                detail: KEPT.to_string(),
            },
        );
        Ok(Output::text(format!("{id}: {KEPT}")))
    })
}

/// Writes a line to standard error: the entries shown and the answers. A failed write is ignored:
/// the answer is what matters.
fn say(text: &str) {
    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr, "{}", text.trim_end_matches(['\r', '\n']));
}
