//! The `tool` command (docs/design.md section 12, D31, D37, format.md §22): `tool add` records a tool
//! the team uses, a script, a command, installed software or a git repository, with its name, its
//! type and, when known, its location. A script whose location is a file gets the checksum of that
//! file, which `check` and `show` compare later (R-USE-11). `tool update <ID>` records that
//! checksum again after an intended change to the script. A tool a person writes is approved at
//! once; one an LLM writes is a draft until a person approves it (approval::guarded).

use crate::approval;
use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::tx::{self, Change};
use chrono::NaiveDate;
use serde_json::json;
use std::io::Read;
use std::path::{Path, PathBuf};

/// The types of a tool (format.md §22).
const TYPES: [&str; 4] = ["script", "command", "software", "repo"];

/// The type whose location gets a checksum when the location is a file (format.md §22).
const SCRIPT: &str = "script";

/// The detail added to the history line when an LLM's change takes an approved tool back to draft
/// (the same words as `set` uses).
const BACK_TO_DRAFT: &str = "back to draft";

/// The arguments of `tool`: the subcommands `add` and `update`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `tool` does.
#[derive(clap::Subcommand, Debug)]
// The command line is parsed once per run, so the size of the largest variant does not matter
// (as in cli.rs).
#[allow(clippy::large_enum_variant)]
pub enum Action {
    /// Records a tool: a script, a command, installed software or a git repository
    Add(AddArgs),
    /// Records the checksum of a script's file again, after an intended change to the script
    Update(UpdateArgs),
}

/// The options of `tool update`.
#[derive(clap::Args, Debug)]
pub struct UpdateArgs {
    /// The ID of the tool, such as TOOL-001
    pub id: String,
}

/// The options of `tool add`.
#[derive(clap::Args, Debug)]
pub struct AddArgs {
    /// The name of the tool, such as "Mesh check script"
    #[arg(long)]
    pub title: String,
    /// The type of the tool: script, command, software or repo
    #[arg(long = "type", value_name = "type")]
    pub tool_type: String,
    /// Where the tool is: a path (a script inside the project folder or an absolute path), a
    /// command name or a URL; optional
    #[arg(long, value_name = "location")]
    pub location: Option<String>,
    /// The software the tool runs in or belongs to, such as ExampleSolver 2406
    #[arg(long)]
    pub software: Option<String>,
    /// The language of a script, such as Python
    #[arg(long)]
    pub language: Option<String>,
    /// A section, as Name=text, such as Purpose=Checks the mesh (repeatable)
    #[arg(long = "section", value_name = "Name=text")]
    pub sections: Vec<String>,
    /// A file with the entry body in Markdown: each line that starts with `## ` starts a section
    #[arg(long, value_name = "file", conflicts_with = "stdin")]
    pub from: Option<PathBuf>,
    /// Reads the entry body in Markdown from standard input
    #[arg(long)]
    pub stdin: bool,
    /// The date the tool should be checked again, as YYYY-MM-DD, such as 2027-06-01; none by
    /// default (R-LIFE-7)
    #[arg(long)]
    pub review: Option<String>,
    /// Tag: a comma-separated list of machines, such as axial compressor
    #[arg(long)]
    pub machine: Option<String>,
    /// Tag: a comma-separated list of analyses, such as steady RANS
    #[arg(long)]
    pub analysis: Option<String>,
    /// Tag: a comma-separated list of free tags
    #[arg(long)]
    pub tags: Option<String>,
}

/// Runs `tool`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match &args.action {
        Action::Add(add) => add_tool(add, g),
        Action::Update(update) => update_tool(update, g),
    }
}

/// Runs `tool add`: checks the options, finds the file of a script's location, and writes the tool
/// in one transaction. The first line of the output is `<ID>: added (approved)` or
/// `<ID>: added (draft, waiting for approval)`. A script whose location is not a file gets a
/// warning and no checksum.
fn add_tool(args: &AddArgs, g: &Global) -> Result<Output> {
    // Checked first, before the dossier is looked for, as `learn` does: a wrong value is refused
    // the same way wherever the command is run.
    let title = title_of(&args.title)?;
    let tool_type = type_of(&args.tool_type)?;
    let review = review_of(args.review.as_deref())?;
    let location = location_of(args.location.as_deref())?;
    let mut sections = match body_text(args)? {
        Some(text) => body_sections(&text)?,
        None => Vec::new(),
    };
    sections.extend(section_pairs(&args.sections)?);

    let ctx = Ctx::load(g, "tool")?;
    let mut warnings: Vec<String> = Vec::new();
    let (location, checksum) = match location {
        Some(typed) if tool_type == SCRIPT => {
            let cwd = std::env::current_dir()?;
            let (stored, checksum) = script_file(&ctx, &cwd, &typed)?;
            if checksum.is_none() {
                warnings.push(format!(
                    "no checksum: {typed} is not a file inside the project folder or an absolute path to a file"
                ));
            }
            (Some(stored), checksum)
        }
        other => (other, None),
    };

    let mut fields = vec![("type".to_string(), tool_type.to_string())];
    let optional = [
        ("location", location.clone()),
        ("software", given(args.software.as_deref())),
        ("language", given(args.language.as_deref())),
        ("checksum", checksum.clone()),
        ("review", review),
        ("machine", given(args.machine.as_deref())),
        ("analysis", given(args.analysis.as_deref())),
        ("tags", given(args.tags.as_deref())),
    ];
    fields.extend(
        optional
            .into_iter()
            .filter_map(|(key, value)| value.map(|value| (key.to_string(), value))),
    );
    let new = New {
        kind: Kind::Tool,
        title,
        fields,
        sections,
        said_by: None,
        draft: false,
    };

    let output = tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = status_of(&text)?;
        Ok(Output::with(
            format!("{id}: added ({})", status_words(status)),
            json!({
                "id": id,
                "status": status.name(),
                "type": tool_type,
                "location": location,
                "checksum": checksum,
            }),
        ))
    })?;
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// Runs `tool update <ID>` (R-USE-11, format.md §22): records the checksum of the script's file
/// again after an intended change, so that `check`, `show` and `list` stop marking it. The entry
/// must be a script whose location is a file that exists: a relative location is taken from the
/// project folder, an absolute one as it is. The change is guarded as `set` guards it
/// (approval::guarded): an LLM's update of an approved tool takes it back to draft, and a person's
/// update records the approval again. The first output line is `<ID>: changed`, with `(back to
/// draft, waiting for approval)` after it when an LLM took the tool back to draft, or
/// `<ID>: no change` when the file already has the recorded checksum.
fn update_tool(args: &UpdateArgs, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "tool")?;
    let who = ctx.writer()?.clone();

    tx::write(&ctx, |tx| {
        let item = store::get(&tx.ctx().dossier, &args.id)?;
        let mut entry = item.entry.clone();
        let id = entry.id();
        if entry.kind() != Some(Kind::Tool) {
            return Err(Error::new(
                Code::Refused,
                format!("{id} is not a tool; only a tool has a checksum to record again"),
            ));
        }
        if entry.field("type").as_deref() != Some(SCRIPT) {
            return Err(Error::new(
                Code::Refused,
                format!("{id} is not a script; only a script has the checksum of its file"),
            ));
        }
        let Some(location) = entry.field("location").filter(|value| !value.is_empty()) else {
            return Err(Error::new(
                Code::MissingField,
                format!("{id} has no location, so its script file is not known"),
            ));
        };
        let path = script_path(&tx.ctx().root, &location);
        if !path.is_file() {
            return Err(Error::new(
                Code::NotFound,
                format!("the script file is not there: {location}"),
            ));
        }
        let checksum = filesum::checksum(&path)?;
        if entry.field("checksum").as_deref() == Some(checksum.as_str()) {
            return Ok(Output::with(
                format!("{id}: no change"),
                json!({
                    "id": id,
                    "status": status_name(&entry),
                    "location": location,
                    "checksum": checksum,
                }),
            ));
        }

        entry.set_field("checksum", &checksum);
        let approved_guarded =
            entry.status() == Some(Status::Approved) && entry.kind().is_some_and(approval::guarded);
        let mut detail = "checksum".to_string();
        let mut back_to_draft = false;
        if approved_guarded && who.is_llm {
            entry.set_field("status", Status::Draft.name());
            entry.remove_field("approved");
            entry.remove_field("valid-from");
            detail.push_str(", ");
            detail.push_str(BACK_TO_DRAFT);
            back_to_draft = true;
        }
        entry.add_history(tx.time(), &who, "changed", &detail);

        let text = entry.text();
        if approved_guarded && !tx.ctx().dry_run {
            let state = &tx.ctx().state;
            if who.is_llm {
                approval::forget(state, &id)?;
            } else {
                approval::record(state, &id, text.as_bytes())?;
            }
        }

        let message = if back_to_draft {
            format!("{id}: changed (back to draft, waiting for approval)")
        } else {
            format!("{id}: changed")
        };
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
        Ok(Output::with(
            message,
            json!({
                "id": id,
                "status": status,
                "location": location,
                "checksum": checksum,
            }),
        ))
    })
}

/// The file of a script's location: an absolute path as it is, a relative one taken from the
/// project folder (format.md §22), as `check` reads it.
fn script_path(root: &Path, location: &str) -> PathBuf {
    let path = Path::new(location);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// The location as stored and the checksum of a script's location (D37, format.md §22). A location
/// that names a file inside the project folder (a relative path is taken from the current folder,
/// as `attach` takes paths) is stored relative to the project folder with `/`, which is how `check`
/// and `show` read it. An absolute path to a file is stored as typed. The checksum is that of the
/// file. A location that is not a file (a command name, a URL, a missing file) is stored as typed
/// with no checksum.
fn script_file(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<(String, Option<String>)> {
    if let Ok(stored) = paths::stored(&ctx.root, cwd, typed) {
        let path = ctx.root.join(paths::shown(&stored));
        if path.is_file() {
            let checksum = filesum::checksum(&path)?;
            return Ok((stored, Some(checksum)));
        }
    }
    let path = Path::new(typed);
    if path.is_absolute() && path.is_file() {
        let checksum = filesum::checksum(path)?;
        return Ok((typed.to_string(), Some(checksum)));
    }
    Ok((typed.to_string(), None))
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}

/// The title without its blank ends. A blank title is refused.
fn title_of(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        Err(bad("the title is empty: give the tool a name with --title"))
    } else {
        Ok(title.to_string())
    }
}

/// The `--type` option, in any case, as one of `TYPES`. Anything else is Code::BadValue.
fn type_of(text: &str) -> Result<&'static str> {
    let wanted = text.trim().to_lowercase();
    TYPES
        .into_iter()
        .find(|known| *known == wanted)
        .ok_or_else(|| {
            bad(format!(
                "--type must be script, command, software or repo, not \"{text}\""
            ))
        })
}

/// The `--location` option trimmed; a blank one is no location. A location is one line
/// (Code::BadValue otherwise).
fn location_of(text: Option<&str>) -> Result<Option<String>> {
    let Some(location) = given(text) else {
        return Ok(None);
    };
    if location.contains(['\r', '\n']) {
        return Err(bad("the location is one line"));
    }
    Ok(Some(location))
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

/// The value of an option without its surrounding spaces; None when nothing is left of it.
fn given(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// The text of `--from <file>` or of standard input with `--stdin`, or None when neither is
/// given. The file is read as UTF-8 from the current folder.
fn body_text(args: &AddArgs) -> Result<Option<String>> {
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

/// The name of the entry's status, or the empty text when it has none.
fn status_name(entry: &Entry) -> &'static str {
    entry.status().map_or("", Status::name)
}
