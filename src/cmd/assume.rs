//! The `assume` command: records an assumption, something taken as true without proof. An
//! assumption is approved at once, whoever writes it, and stays `open` until someone confirms it
//! or finds it wrong (R-WRITE-14, R-ENTRY-13).

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx;
use serde_json::json;

/// The state of a new assumption: open until someone confirms it or finds it wrong.
const OPEN: &str = "open";

/// The number of characters of the assumption's text that a default title takes.
const TITLE_CHARS: usize = 80;

/// The arguments of `assume`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The assumption: what is taken as true without proof
    pub assumption: String,
    /// Why it had to be assumed
    #[arg(long)]
    pub reason: String,
    /// What rests on it: tasks, files or values
    #[arg(long)]
    pub affects: Option<String>,
    /// The title (default: the first 80 characters of the assumption)
    #[arg(long)]
    pub title: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// Machines, as a comma-separated list
    #[arg(long)]
    pub machine: Option<String>,
    /// Analysis types, as a comma-separated list
    #[arg(long)]
    pub analysis: Option<String>,
    /// Software, as a comma-separated list
    #[arg(long)]
    pub software: Option<String>,
    /// Other tags, as a comma-separated list
    #[arg(long)]
    pub tags: Option<String>,
    /// The topics it is about, from the project's list (dossier topics), comma-separated
    #[arg(long)]
    pub topic: Option<String>,
}

/// Runs `assume`: writes the assumption and prints its ID and its status.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "assume")?;
    let new = New {
        kind: Kind::Assumption,
        title: title_of(args)?,
        fields: fields(args),
        sections: sections(args),
        said_by: None,
        draft: false,
    };
    tx::write(&ctx, |tx| {
        let (id, text) = create::create(tx, &new)?;
        let status = Entry::parse(&text)?
            .status()
            .map_or("unknown", Status::name);
        Ok(Output::with(
            format!("{id}: added ({status}, {OPEN})"),
            json!({"id": id, "status": status}),
        ))
    })
}

/// The title given with `--title`, or the default made from the assumption. A blank title is
/// refused, because the entry's heading needs one.
fn title_of(args: &Args) -> Result<String> {
    match &args.title {
        Some(title) if title.trim().is_empty() => {
            Err(Error::new(Code::BadValue, "the title must not be empty"))
        }
        Some(title) => Ok(title.clone()),
        None => Ok(default_title(&args.assumption)),
    }
}

/// The first 80 characters of the text (characters, not bytes), with line breaks made into
/// spaces and the ends trimmed.
fn default_title(text: &str) -> String {
    let head: String = text.chars().take(TITLE_CHARS).collect();
    head.split(['\r', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The fields of the assumption: its state, then the task (upper-cased) and the tag lists and the
/// topic as given. `entry::new::render` leaves out a field whose value is empty.
fn fields(args: &Args) -> Vec<(String, String)> {
    let mut fields = vec![pair("state", OPEN)];
    if let Some(task) = &args.task {
        fields.push(pair("task", &task.to_uppercase()));
    }
    let tags = [
        ("machine", &args.machine),
        ("analysis", &args.analysis),
        ("software", &args.software),
        ("tags", &args.tags),
        ("topic", &args.topic),
    ];
    for (key, value) in tags {
        if let Some(value) = value {
            fields.push(pair(key, value));
        }
    }
    fields
}

/// The sections of the assumption: the assumption itself, its reason, and what rests on it when
/// given (format.md §8). The Confirmation section is written later, by confirm.
fn sections(args: &Args) -> Vec<(String, String)> {
    let mut sections = vec![
        pair("Assumption", &args.assumption),
        pair("Reason", &args.reason),
    ];
    if let Some(affects) = &args.affects {
        sections.push(pair("Affects", affects));
    }
    sections
}

/// A field or section as a pair of owned strings.
fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}
