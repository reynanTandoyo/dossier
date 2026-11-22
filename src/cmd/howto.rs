//! The `howto` command (docs/design.md section 12, R-ENTRY-8, R-WRITE-5, R-WRITE-6): writes a how-to,
//! how a software is used for a task. `--software` and `--title` are required; the sections are
//! Purpose, Steps, Settings, Traps and Tested with (format.md §22). A person's how-to is approved at
//! once; an LLM's is a draft until a person approves it (approval::guarded). With `--team` the
//! how-to goes into the team dossier, after the confidential check that create.rs makes.

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use chrono::NaiveDate;
use serde_json::json;
use std::io::Read;
use std::path::PathBuf;

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The arguments of `howto`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The software the how-to is for, such as ExampleSolver 2406
    #[arg(long)]
    pub software: String,
    /// The title of the how-to
    #[arg(long)]
    pub title: String,
    /// A section, as Name=text (repeatable): Purpose, Steps, Settings, Traps or Tested with
    #[arg(long = "section", value_name = "Name=text")]
    pub sections: Vec<String>,
    /// A file with the body in Markdown: each line that starts with `## ` starts a section
    #[arg(long, value_name = "file", conflicts_with = "stdin")]
    pub from: Option<PathBuf>,
    /// Reads the body in Markdown from standard input
    #[arg(long)]
    pub stdin: bool,
    /// The date the how-to should be checked again, as YYYY-MM-DD, such as 2027-06-01
    #[arg(long)]
    pub review: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// Tag: a comma-separated list of machines, such as axial compressor
    #[arg(long)]
    pub machine: Option<String>,
    /// Tag: a comma-separated list of analyses, such as steady RANS, speedline
    #[arg(long)]
    pub analysis: Option<String>,
    /// Tag: a comma-separated list of free tags
    #[arg(long)]
    pub tags: Option<String>,
}

/// Runs `howto`: checks the options, reads the body, and writes the how-to in one transaction.
/// The first output line is `<ID>: added (approved)` or `<ID>: added (draft, waiting for approval)`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    // Checked first, before the dossier is looked for, as `learn` checks its options.
    let software = software_of(&args.software)?;
    let title = title_of(&args.title)?;
    let review = review_of(args.review.as_deref())?;

    let ctx = Ctx::load(g, "howto")?;
    let mut sections = match body_text(args)? {
        Some(text) => body_sections(&text)?,
        None => Vec::new(),
    };
    sections.extend(section_pairs(&args.sections)?);
    let new = New {
        kind: Kind::Howto,
        title,
        fields: fields_of(args, &software, review.as_deref()),
        sections,
        said_by: None,
        draft: false,
    };
    let team = ctx.cfg.kind == TEAM;

    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = status_of(&text)?;
        Ok(Output::with(
            format!("{id}: added ({})", status_words(status)),
            json!({"id": id, "status": status.name(), "team": team}),
        ))
    })
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// The software named with `--software`, trimmed. A blank one is Code::MissingField: a how-to is
/// for a software. A line break is Code::BadValue (a field is one line).
fn software_of(software: &str) -> Result<String> {
    let software = software.trim();
    if software.is_empty() {
        return Err(Error::new(
            Code::MissingField,
            "--software is empty: name the software the how-to is for",
        ));
    }
    if software.contains(['\r', '\n']) {
        return Err(bad("the software is one line"));
    }
    Ok(software.to_string())
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

/// The `--review` option as YYYY-MM-DD, or None when it is not given. Code::BadValue when it is
/// not a date of that form.
fn review_of(text: Option<&str>) -> Result<Option<String>> {
    text.map(|text| {
        NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
            .map(|day| day.format("%Y-%m-%d").to_string())
            .map_err(|_| {
                bad(format!(
                    "--review needs a date such as 2027-06-01, not \"{text}\""
                ))
            })
    })
    .transpose()
}

/// The fields of the how-to: its software, then the review date, the task and the tags that were
/// given, in that order. Empty options are left out.
fn fields_of(args: &Args, software: &str, review: Option<&str>) -> Vec<(String, String)> {
    let mut fields = vec![("software".to_string(), software.to_string())];
    if let Some(review) = review {
        fields.push(("review".to_string(), review.to_string()));
    }
    if let Some(task) = given(args.task.as_deref()) {
        fields.push(("task".to_string(), task.to_uppercase()));
    }
    fields.extend(
        tag_fields(args)
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    );
    fields
}

/// The tag options that were given, as (field, value), without the empty ones.
fn tag_fields(args: &Args) -> Vec<(&'static str, String)> {
    [
        ("machine", args.machine.as_deref()),
        ("analysis", args.analysis.as_deref()),
        ("tags", args.tags.as_deref()),
    ]
    .into_iter()
    .filter_map(|(key, value)| given(value).map(|value| (key, value)))
    .collect()
}

/// The value of an option without its surrounding spaces; None when nothing is left of it.
fn given(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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
