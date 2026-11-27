//! The `checked` command (docs/design.md section 25.5, R-WRITE-18, R-ENTRY-21, D62): a person says
//! that the check of a lesson was shown to fail on the mistake the lesson is about. The field
//! `check-state` becomes `checked`, `--check` sets `check` when it is given, and the history line
//! `checked` carries the `--by` text. An LLM carries it out for its person with `--said-by` (D57),
//! as `reviewed` does. An approved lesson gets its approved text recorded again, so that the change
//! of its check fields is not taken for a hand change (R-LIFE-6).

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::names;
use crate::output::Output;
use crate::store;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;

/// The field that names the check that catches the lesson's mistake (format.md §38).
const CHECK: &str = "check";

/// The field that says whether the check was shown to catch the mistake (format.md §38).
const CHECK_STATE: &str = "check-state";

/// The value of `check-state` after this command.
const CHECKED: &str = "checked";

/// The history event and the journal event of this command (format.md §6).
const EVENT: &str = "checked";

/// The arguments of `checked`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The lesson, such as LES-004; any case, with or without a place prefix
    pub id: String,
    /// How the check was shown to fail on the mistake, such as "the test fails without the fix"
    #[arg(long)]
    pub by: String,
    /// The test, lint, hook or review step that catches the mistake; needed when the lesson names
    /// none
    #[arg(long)]
    pub check: Option<String>,
    /// The person who decided this; needed when an LLM runs the command (D57)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `checked`: checks who acts (names::said_by_gate, D57), that a person is at a terminal
/// unless an LLM carries it out for its person with `--said-by`, and that `--by` is not empty.
/// Then, in one transaction, refuses an entry that is not a lesson, a lesson that is neither a
/// draft nor approved, and a lesson with no check when `--check` is not given. Otherwise it sets
/// `check` when `--check` is given and `check-state` to `checked`, and adds the history line
/// `checked` with the `--by` text. The first output line is `<ID>: checked`. The data is
/// {id, check, by}, where check is the check the lesson now names.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "checked")?;
    let who = ctx.writer()?.clone();
    let said = names::said_by_gate(&who, args.said_by.as_deref(), "checked")?;
    if !said && !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    let by = args.by.trim();
    if by.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "say with --by how the check was shown to catch the mistake",
        ));
    }
    let given = args
        .check
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let id = item.entry.id();
        if item.entry.kind() != Some(Kind::Lesson) {
            return Err(Error::new(Code::Refused, format!("{id} is not a lesson")));
        }
        match item.entry.status() {
            Some(Status::Draft | Status::Approved) => {}
            Some(status) => {
                return Err(Error::new(
                    Code::Refused,
                    format!(
                        "{id} is {}; only a draft or approved lesson can be checked",
                        status.name()
                    ),
                ));
            }
            None => {
                return Err(Error::new(
                    Code::Refused,
                    format!("{id} has no status; only a draft or approved lesson can be checked"),
                ));
            }
        }

        let mut entry = item.entry.clone();
        let named = entry
            .field(CHECK)
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty());
        let check = match (given, named) {
            (Some(text), _) => text.to_string(),
            (None, Some(text)) => text,
            (None, None) => {
                return Err(Error::new(
                    Code::Refused,
                    format!("{id} names no check: give --check"),
                ));
            }
        };

        if let Some(text) = given {
            entry.set_field(CHECK, text);
        }
        entry.set_field(CHECK_STATE, CHECKED);
        let time = tx.time().to_string();
        entry.add_history(&time, &who, EVENT, by);

        let text = entry.text();
        if entry.status() == Some(Status::Approved) && !tx.ctx().dry_run {
            approval::record(&tx.ctx().state, &id, text.as_bytes())?;
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: EVENT.to_string(),
                detail: by.to_string(),
            },
        );

        Ok(Output::with(
            format!("{id}: checked"),
            json!({"id": id, "check": check, "by": by}),
        ))
    })
}
