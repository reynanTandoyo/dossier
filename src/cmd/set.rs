//! The `set` command: changes the fields and sections named on the command line and nothing else
//! (R-WRITE-8, R-EDIT-1, R-HIST-5). A part that is not named keeps its bytes. An LLM's change to
//! an approved lesson, how-to or tool takes it back to draft until a person approves it again; a
//! person's change to one of them is recorded as its new approved text (approval::guarded).

use crate::approval;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;

/// The fields the tool writes itself (format.md §5). `set` refuses them.
const TOOL_OWNED: [&str; 8] = [
    "status",
    "created",
    "author",
    "via",
    "approved",
    "valid-from",
    "valid-until",
    "history",
];

/// The detail added to the history line when an LLM's change takes an approved lesson, how-to or
/// tool back to draft.
const BACK_TO_DRAFT: &str = "back to draft";

/// The arguments of `set`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012
    pub id: String,
    /// A field, as key=value; an empty value removes the field (repeatable)
    #[arg(long = "field", value_name = "key=value")]
    pub fields: Vec<String>,
    /// A section, as Name=text; an empty text removes the section (repeatable)
    #[arg(long = "section", value_name = "Name=text")]
    pub sections: Vec<String>,
}

/// Runs `set`: checks the options, then changes the named parts of the entry in one transaction.
/// The first output line is `<ID>: changed`, with `(back to draft, waiting for approval)` after
/// it when an LLM took an approved lesson, how-to or tool back to draft, or `<ID>: no change`
/// when the named parts already had these values.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "set")?;
    if args.fields.is_empty() && args.sections.is_empty() {
        return Err(Error::new(
            Code::Usage,
            "nothing to change: give --field key=value or --section 'Name=text'",
        ));
    }
    let fields = field_pairs(&args.fields)?;
    check_topics(&ctx.cfg.topics, &fields)?;
    let sections = section_pairs(&args.sections)?;
    let who = ctx.writer()?.clone();

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let mut entry = item.entry.clone();

        // Each named part is applied on its own. A part counts as changed only when the text
        // of the entry changed, so naming a value the entry already has changes nothing.
        let mut changed: Vec<String> = Vec::new();
        for (key, value) in &fields {
            let before = entry.text();
            entry.set_field(key, value);
            if entry.text() != before {
                note(&mut changed, format!("field {key}"));
            }
        }
        for (name, body) in &sections {
            let before = entry.text();
            entry.set_section(name, body);
            if entry.text() != before {
                note(&mut changed, format!("section {name}"));
            }
        }
        let id = item.entry.id();
        if changed.is_empty() {
            let status = status_name(&entry);
            return Ok(Output::with(
                format!("{id}: no change"),
                json!({"id": id, "status": status, "changed": changed}),
            ));
        }

        let approved_guarded =
            entry.status() == Some(Status::Approved) && entry.kind().is_some_and(approval::guarded);
        let mut detail = changed.join(", ");
        let mut back_to_draft = false;
        if approved_guarded && who.is_llm {
            entry.set_field("status", "draft");
            entry.remove_field("approved");
            entry.remove_field("valid-from");
            detail.push_str(", ");
            detail.push_str(BACK_TO_DRAFT);
            back_to_draft = true;
        }
        entry.add_history(tx.time(), &who, "changed", &detail);

        let text = entry.text();
        let dry_run = tx.ctx().dry_run;
        if approved_guarded && !dry_run {
            let state = &tx.ctx().state;
            if who.is_llm {
                approval::forget(state, &id)?;
            } else {
                approval::record(state, &id, text.as_bytes())?;
            }
        }

        let status = status_name(&entry);
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: id.clone(),
                event: "changed".to_string(),
                detail,
            },
        );
        let message = if back_to_draft {
            format!("{id}: changed (back to draft, waiting for approval)")
        } else {
            format!("{id}: changed")
        };
        Ok(Output::with(
            message,
            json!({"id": id, "status": status, "changed": changed}),
        ))
    })
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// Adds a changed part to the list once, keeping the order in which the parts changed.
fn note(changed: &mut Vec<String>, part: String) {
    if !changed.contains(&part) {
        changed.push(part);
    }
}

/// The name of the entry's status, or the empty text when it has none.
fn status_name(entry: &Entry) -> &'static str {
    entry.status().map_or("", Status::name)
}

/// The `--field key=value` options as pairs: the key lower-cased, key and value trimmed. An
/// option without `=`, a key that is not letters, digits and `-`, a key the tool writes itself,
/// and `kind` (the folder and the ID prefix depend on it) are refused (Code::BadValue).
fn field_pairs(items: &[String]) -> Result<Vec<(String, String)>> {
    items
        .iter()
        .map(|item| {
            let (key, value) = item
                .split_once('=')
                .ok_or_else(|| bad(format!("a field is written key=value: {item}")))?;
            let key = key.trim().to_lowercase();
            if !valid_key(&key) {
                return Err(bad(format!(
                    "a field key has only letters, digits and -: {key}"
                )));
            }
            if TOOL_OWNED.contains(&key.as_str()) {
                return Err(bad("this field is written by the tool"));
            }
            if key == "kind" {
                return Err(bad(
                    "the kind of an entry cannot change; add a new entry instead",
                ));
            }
            Ok((key, value.trim().to_string()))
        })
        .collect()
}

/// When the project has topics, each comma-separated value of a `topic` field, trimmed, must equal
/// a topic of the list ignoring case (R-TOPIC-2). Empty pieces are skipped, as create skips them.
/// An empty value removes the field and is allowed; with no topics in the settings any value is
/// allowed. Code::Usage names the unknown value and the topics of the project.
fn check_topics(topics: &[String], fields: &[(String, String)]) -> Result<()> {
    if topics.is_empty() {
        return Ok(());
    }
    for (key, value) in fields {
        if key != "topic" || value.is_empty() {
            continue;
        }
        for name in value
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            let wanted = name.to_lowercase();
            if !topics.iter().any(|topic| topic.to_lowercase() == wanted) {
                return Err(Error::new(
                    Code::Usage,
                    format!(
                        "unknown topic {name}; the topics of this project: {} (dossier topics)",
                        topics.join(", ")
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Whether a field key is one or more letters, digits or `-` (format.md §3).
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The `--section Name=text` options as pairs: the name trimmed, the text as given. An option
/// without `=`, a name that is blank or runs over a line, and a text with a line that starts with
/// `## ` (it would start a section of its own) are refused (Code::BadValue).
fn section_pairs(items: &[String]) -> Result<Vec<(String, String)>> {
    items
        .iter()
        .map(|item| {
            let (name, text) = item
                .split_once('=')
                .ok_or_else(|| bad(format!("a section is written Name=text: {item}")))?;
            let name = name.trim();
            if name.is_empty() {
                return Err(bad(format!("a section needs a name: {item}")));
            }
            if name.contains(['\r', '\n']) {
                return Err(bad(format!("a section name is one line: {name}")));
            }
            if text.lines().any(|line| line.starts_with("## ")) {
                return Err(bad(format!(
                    "a line of a section cannot start with ## : {name}"
                )));
            }
            Ok((name.to_string(), text.to_string()))
        })
        .collect()
}
