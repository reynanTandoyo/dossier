//! The `add` command: writes one entry of any kind, from options, a file or standard input
//! (R-WRITE-1, R-HIST-4). The short commands are built the same way.

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;

/// The fields the tool writes itself (format.md §5). `add` refuses them.
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

/// The arguments of `add`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The kind of the entry: decision, assumption, finding or lesson
    pub kind: String,
    /// The title of the entry
    #[arg(long)]
    pub title: String,
    /// A field, as key=value, such as machine=axial compressor (repeatable)
    #[arg(long = "field", value_name = "key=value")]
    pub fields: Vec<String>,
    /// A section, as Name=text (repeatable)
    #[arg(long = "section", value_name = "Name=text")]
    pub sections: Vec<String>,
    /// A file with the entry body in Markdown: each line that starts with `## ` starts a section
    #[arg(long, value_name = "file", conflicts_with = "stdin")]
    pub from: Option<PathBuf>,
    /// Reads the entry body in Markdown from standard input
    #[arg(long)]
    pub stdin: bool,
    /// The todo task the entry came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The person an LLM writes for
    #[arg(long)]
    pub said_by: Option<String>,
    /// Writes a draft that waits for a person's approval
    #[arg(long)]
    pub draft: bool,
}

/// Runs `add`: checks the options, reads the body, and writes the entry in one transaction.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "add")?;
    let kind = Kind::parse(&args.kind).ok_or_else(|| {
        bad(format!(
            "unknown kind {}: use decision, assumption, finding or lesson",
            args.kind
        ))
    })?;
    let title = title_of(&args.title)?;
    let mut fields = field_pairs(&args.fields)?;
    if let Some(task) = task_of(args.task.as_deref()) {
        if fields
            .iter()
            .any(|(key, value)| key.as_str() == "task" && !value.is_empty())
        {
            return Err(bad(
                "the task is given twice: with --task and with --field task=",
            ));
        }
        fields.push(("task".to_string(), task));
    }
    // An assumption starts open, as `assume` writes it, unless a state with a value is given.
    // Without a state, `approve` refuses the draft (rules::problems).
    if kind == Kind::Assumption
        && !fields
            .iter()
            .any(|(key, value)| key.as_str() == "state" && !value.is_empty())
    {
        fields.push(("state".to_string(), "open".to_string()));
    }
    // The sections of the body come before the ones given with --section.
    let mut sections = match body_text(args)? {
        Some(text) => body_sections(&text)?,
        None => Vec::new(),
    };
    sections.extend(section_pairs(&args.sections)?);
    let new = New {
        kind,
        title,
        fields,
        sections,
        said_by: args.said_by.clone(),
        draft: args.draft,
    };

    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = status_of(&text)?;
        Ok(Output::with(
            format!("{id}: added ({})", status_words(status)),
            json!({"id": id, "status": status.name()}),
        ))
    })
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// The title without its blank ends. A blank title is refused.
fn title_of(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        Err(bad("the title is empty"))
    } else {
        Ok(title.to_string())
    }
}

/// The `--task` option, trimmed and upper-cased. A blank task is no task.
fn task_of(task: Option<&str>) -> Option<String> {
    task.map(|task| task.trim().to_uppercase())
        .filter(|task| !task.is_empty())
}

/// The `--field key=value` options as pairs: the key lower-cased, key and value trimmed. An
/// option without `=`, a key that is not letters, digits and `-`, and a key the tool writes
/// itself are refused (Code::BadValue).
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
            Ok((key, value.trim().to_string()))
        })
        .collect()
}

/// Whether a field key is one or more letters, digits or `-` (format.md §3).
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The `--section Name=text` options as pairs: the name trimmed, the text as given. An option
/// without `=`, or with a blank name, is refused (Code::BadValue).
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
            Ok((name.to_string(), text.to_string()))
        })
        .collect()
}

/// The text of `--from <file>` or of standard input with `--stdin`, or None when neither is
/// given. The file is read as UTF-8 from the current folder.
fn body_text(args: &Args) -> Result<Option<String>> {
    if let Some(path) = &args.from {
        let text = std::fs::read_to_string(path)
            .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))?;
        return Ok(Some(text));
    }
    if args.stdin {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|error| Error::new(Code::Io, format!("standard input: {error}")))?;
        return Ok(Some(text));
    }
    Ok(None)
}

/// The sections of a Markdown body. The text is split at each line that starts with `## `; the
/// name is the rest of that line, trimmed, and the body is the lines up to the next such line.
/// Text before the first heading is ignored. A heading with no name is refused (Code::BadValue).
/// The blank lines at the ends of a body are left to `create`, which drops them.
fn body_sections(text: &str) -> Result<Vec<(String, String)>> {
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            let name = rest.trim();
            if name.is_empty() {
                return Err(bad("a section heading needs a name after ##"));
            }
            sections.push((name.to_string(), Vec::new()));
        } else if let Some((_, body)) = sections.last_mut() {
            body.push(line);
        }
    }
    Ok(sections
        .into_iter()
        .map(|(name, body)| (name, body.join("\n")))
        .collect())
}

/// The status of the text that the transaction created.
fn status_of(text: &str) -> Result<Status> {
    Entry::parse(text)?
        .status()
        .ok_or_else(|| Error::new(Code::Parse, "the new entry has no status"))
}

/// The words for the status in the first output line (docs/design.md section 5).
fn status_words(status: Status) -> &'static str {
    match status {
        Status::Draft => "draft, waiting for approval",
        _ => status.name(),
    }
}
