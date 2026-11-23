//! The `tried` command (docs/design.md section 15, R-INV-3): adds one attempt at the end of the
//! Attempts section of an investigation. The attempt gets the next number, its verdict, its text
//! and the optional parts the flags give, with the person (or LLM) and the time. An investigation
//! that is solved or given up is opened again (R-INV-4). Only an approved investigation takes an
//! attempt, and a team dossier has none. Nothing is written when a check fails.

use crate::attempts::{self, Attempt, SECTION, VERDICTS};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names;
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The value of `kind` in dossier.toml of a team dossier.
const TEAM: &str = "team";

/// The state of an investigation that takes attempts without being opened again.
const OPEN: &str = "open";

/// The fields that a closed investigation loses when it is opened again.
const CLOSED_FIELDS: [&str; 2] = ["closed", "given-up-reason"];

/// The arguments of `tried`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the investigation, such as INV-003; any case, with or without a place prefix
    pub id: String,
    /// The hypothesis, or what was tried, in one line
    pub text: String,
    /// The attempt did not solve it
    #[arg(long)]
    pub failed: bool,
    /// The attempt worked, at least for a part of the problem
    #[arg(long)]
    pub worked: bool,
    /// It is not clear what the attempt showed
    #[arg(long)]
    pub unclear: bool,
    /// What was changed, in one line
    #[arg(long)]
    pub did: Option<String>,
    /// What happened, in one line
    #[arg(long)]
    pub result: Option<String>,
    /// The usage record of the run that tested it, such as USE-007
    #[arg(long)]
    pub run: Option<String>,
    /// The todo task that carried out the attempt, such as T-013
    #[arg(long)]
    pub task: Option<String>,
}

/// Runs `tried`: checks the arguments and the investigation and the usage record, then, in one
/// transaction, appends the attempt, opens the investigation again when it is closed, and adds the
/// history lines. The output is `<ID>: attempt <n> recorded (<verdict>)`, with `, opened again`
/// when it was closed; the data is {id, attempt, verdict, state}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "tried")?;
    if ctx.cfg.kind == TEAM {
        return Err(Error::new(
            Code::Refused,
            "investigations belong to a project, not to the team dossier",
        ));
    }
    let who = ctx.writer()?.clone();
    let verdict = verdict_of(args)?;
    let text = args.text.trim();
    if text.is_empty() {
        return Err(Error::new(Code::BadValue, "the attempt must not be empty"));
    }
    check_line(text, "the attempt")?;
    let did = optional_line(args.did.as_deref(), "--did")?.map(|did| unseparated(&did));
    let result =
        optional_line(args.result.as_deref(), "--result")?.map(|result| unseparated(&result));
    let run = optional_line(args.run.as_deref(), "--run")?;
    let task = optional_line(args.task.as_deref(), "--task")?.map(|task| task.to_uppercase());

    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let item = store::get(&dossier, &args.id)?;
        let id = item.entry.id();
        ensure_investigation(&item.entry, &id)?;
        let run = run
            .as_deref()
            .map(|typed| usage_id(&dossier, typed))
            .transpose()?;

        let time = tx.time().to_string();
        let mut entry = item.entry.clone();
        let number = attempts::next_number(&entry.section(SECTION).unwrap_or_default());
        let attempt = Attempt {
            n: number,
            verdict: verdict.to_string(),
            text: unseparated(text),
            did,
            result,
            run,
            task,
            time: time.clone(),
            who: names::label(&who),
        };
        let line = attempts::line(&attempt);
        let body = match entry.section(SECTION) {
            Some(body) if !body.is_empty() => format!("{body}\n{line}"),
            _ => line,
        };
        entry.set_section(SECTION, &body);

        let reopened = entry.field("state").as_deref() != Some(OPEN);
        if reopened {
            entry.set_field("state", OPEN);
            for key in CLOSED_FIELDS {
                entry.remove_field(key);
            }
            entry.add_history(&time, &who, "reopened", "");
        }
        let detail = format!("{number} {verdict}");
        entry.add_history(&time, &who, "tried", &detail);
        tx.put(
            item.path.clone(),
            Some(&item),
            entry.text(),
            Change {
                id: id.clone(),
                event: "tried".to_string(),
                detail,
            },
        );

        let mut message = format!("{id}: attempt {number} recorded ({verdict})");
        if reopened {
            message.push_str(", opened again");
        }
        let data = json!({"id": id, "attempt": number, "verdict": verdict, "state": OPEN});
        Ok(Output::with(message, data))
    })
}

/// The verdict the flags give: exactly one of --failed, --worked and --unclear (Code::Usage). The
/// flags are read in the order of `VERDICTS`.
fn verdict_of(args: &Args) -> Result<&'static str> {
    let flags = [args.failed, args.worked, args.unclear];
    let given: Vec<&'static str> = VERDICTS
        .into_iter()
        .zip(flags)
        .filter_map(|(verdict, on)| on.then_some(verdict))
        .collect();
    match given.as_slice() {
        [verdict] => Ok(*verdict),
        _ => Err(Error::new(
            Code::Usage,
            "give exactly one verdict: --failed, --worked or --unclear",
        )),
    }
}

/// Refuses (Code::Refused) an entry that is not an approved investigation. The message names what
/// the entry is instead.
fn ensure_investigation(entry: &Entry, id: &str) -> Result<()> {
    match kind_of(entry, id) {
        Some(Kind::Investigation) => {}
        Some(kind) => {
            return Err(refused(format!(
                "tried records attempts on investigations only; {id} is of kind {}",
                kind.name()
            )));
        }
        None => {
            return Err(refused(format!(
                "{id} has no valid kind, so it takes no attempt"
            )));
        }
    }
    match entry.status() {
        Some(Status::Approved) => Ok(()),
        Some(status) => Err(refused(format!(
            "only an approved investigation takes an attempt; {id} is {}",
            status.name()
        ))),
        None => Err(refused(format!(
            "{id} has no valid status, so it takes no attempt"
        ))),
    }
}

/// The ID, in upper case, of the usage record that `typed` names. Code::NotFound when there is no
/// such entry; Code::Refused when it is another kind.
fn usage_id(dossier: &Path, typed: &str) -> Result<String> {
    let item = store::get(dossier, typed)?;
    let id = item.entry.id();
    match kind_of(&item.entry, &id) {
        Some(Kind::Usage) => Ok(id.to_uppercase()),
        Some(kind) => Err(refused(format!(
            "--run must name a usage record; {id} is of kind {}",
            kind.name()
        ))),
        None => Err(refused(format!(
            "--run must name a usage record; {id} has no valid kind"
        ))),
    }
}

/// The kind of an entry: its `kind` field, else the kind its ID names.
fn kind_of(entry: &Entry, id: &str) -> Option<Kind> {
    entry.kind().or_else(|| Kind::of_id(id))
}

/// The value of an option trimmed and checked as one line (`check_line`); None when it is not
/// given or blank.
fn optional_line(value: Option<&str>, what: &str) -> Result<Option<String>> {
    match trimmed(value) {
        Some(text) => {
            check_line(text, what)?;
            Ok(Some(text.to_string()))
        }
        None => Ok(None),
    }
}

/// Refuses (Code::BadValue) a text that is more than one line or starts with a heading mark.
fn check_line(text: &str, what: &str) -> Result<()> {
    if text.contains(['\r', '\n']) {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must be one line"),
        ));
    }
    if text.starts_with("## ") {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must not start with the heading mark ##"),
        ));
    }
    Ok(())
}

/// The text with its ends trimmed, or None when it is not given or blank.
fn trimmed(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// The separator forms of an attempt line, as attempts.rs writes them (format.md §28): did, result,
/// run and task. attempts.rs keeps its own copy private, so the forms are repeated here.
const SEPARATOR_FORMS: [&str; 4] = [" — did: ", " — result: ", " — run: ", " — task: "];

/// The text with each separator form (`SEPARATOR_FORMS`) written with an en dash (U+2013) in place
/// of its em dash, so that the line reads back the same (format.md §28). Every other character,
/// including a plain em dash, stays as it is.
fn unseparated(text: &str) -> String {
    let mut text = text.to_string();
    for form in SEPARATOR_FORMS {
        text = text.replace(form, &form.replace('—', "–"));
    }
    text
}

/// A refusal with the given message, Code::Refused.
fn refused(message: String) -> Error {
    Error::new(Code::Refused, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_separator_form_in_a_text_gets_an_en_dash() {
        assert_eq!(
            unseparated("CFL 5 — result: diverges"),
            "CFL 5 – result: diverges"
        );
    }

    #[test]
    fn a_plain_em_dash_in_a_text_stays() {
        assert_eq!(
            unseparated("CFL 5 — still diverges"),
            "CFL 5 — still diverges"
        );
    }

    #[test]
    fn two_separator_forms_in_a_text_are_both_replaced() {
        assert_eq!(
            unseparated("CFL 5 — did: one — result: two"),
            "CFL 5 – did: one – result: two"
        );
    }

    #[test]
    fn all_four_separator_forms_are_replaced_and_the_rest_is_kept() {
        assert_eq!(
            unseparated("a — did: b — result: c — run: d — task: e — f"),
            "a – did: b – result: c – run: d – task: e — f"
        );
    }

    #[test]
    fn a_text_with_a_separator_form_reads_back_as_the_text_written() {
        let text = unseparated("CFL 5 — result: diverges");
        let line = format!("1. [failed] {text} (2026-11-20T10:00:00+01:00 anna)");
        let attempts = attempts::parse(&line);
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].text, "CFL 5 – result: diverges");
        assert_eq!(attempts[0].result, None);
    }
}
