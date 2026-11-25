//! The `problem` command (docs/design.md section 15, R-INV-2): opens an investigation with state
//! `open`. An investigation is a fact of the project and is approved at once, whoever writes it
//! (R-INV-1). Before it writes, it names the approved investigations most like it, so that an
//! existing one is continued instead of started again.

use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::glossary::{self, Term};
use crate::output::Output;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::tx;
use serde_json::json;

/// The value of `kind` in dossier.toml of a team dossier.
const TEAM: &str = "team";

/// How many characters of the problem the default title takes.
const TITLE_CHARS: usize = 80;

/// How many similar investigations the output names at most.
const SIMILAR: usize = 5;

/// The arguments of `problem`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// What goes wrong, in one line
    pub problem: String,
    /// What is seen when it goes wrong, in one line
    #[arg(long)]
    pub observed: Option<String>,
    /// The title; the default is the first 80 characters of the problem
    #[arg(long)]
    pub title: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The machines it is about, comma-separated
    #[arg(long)]
    pub machine: Option<String>,
    /// The analyses it is about, comma-separated
    #[arg(long)]
    pub analysis: Option<String>,
    /// The software it is about, comma-separated
    #[arg(long)]
    pub software: Option<String>,
    /// Other tags, comma-separated
    #[arg(long)]
    pub tags: Option<String>,
}

/// One similar investigation as the output names it.
struct Similar {
    id: String,
    state: String,
    title: String,
}

/// Runs `problem`: refuses a team dossier, lists the similar investigations, then creates the
/// investigation in a transaction and says its ID.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "problem")?;
    if ctx.cfg.kind == TEAM {
        return Err(Error::new(
            Code::Refused,
            "investigations belong to a project, not to the team dossier; promote a solved one with dossier promote <ID> --as howto",
        ));
    }
    let problem = args.problem.trim();
    if problem.is_empty() {
        return Err(Error::new(Code::BadValue, "the problem must not be empty"));
    }
    check_line(problem, "the problem")?;
    let observed = trimmed(args.observed.as_deref());
    if let Some(observed) = observed {
        check_line(observed, "--observed")?;
    }

    let title = title_of(args, problem);
    let query = Query {
        tags: tag_values(args),
        words: select::words(&title),
    };
    let new = new_investigation(args, problem, observed, &title);
    tx::write(&ctx, |tx| {
        let similar = similar_to(tx.ctx(), &query);
        let (id, _text) = create::create(tx, &new)?;
        let mut lines = vec![format!("{id}: added (open)")];
        lines.extend(
            similar
                .iter()
                .map(|found| format!("similar: {} [{}] {}", found.id, found.state, found.title)),
        );
        let ids: Vec<&str> = similar.iter().map(|found| found.id.as_str()).collect();
        let data = json!({"id": id, "state": "open", "similar": ids});
        Ok(Output::with(lines.join("\n"), data))
    })
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

/// What `problem` asks to create: state open, the task and the tags the options give, and the
/// Problem section: the problem, then `Observed: <text>` after an empty line when it is given.
fn new_investigation(args: &Args, problem: &str, observed: Option<&str>, title: &str) -> New {
    let mut fields = vec![pair("state", "open")];
    if let Some(task) = trimmed(args.task.as_deref()) {
        fields.push(pair("task", &task.to_uppercase()));
    }
    for (key, value) in [
        ("machine", &args.machine),
        ("analysis", &args.analysis),
        ("software", &args.software),
        ("tags", &args.tags),
    ] {
        if let Some(value) = trimmed(value.as_deref()) {
            fields.push(pair(key, value));
        }
    }
    let body = match observed {
        Some(observed) => format!("{problem}\n\nObserved: {observed}"),
        None => problem.to_string(),
    };
    New {
        kind: Kind::Investigation,
        title: title.to_string(),
        fields,
        sections: vec![pair("Problem", &body)],
        said_by: None,
        draft: false,
    }
}

/// The title: `--title` when it is not blank, else the first 80 characters of the problem.
/// Line breaks become spaces.
fn title_of(args: &Args, problem: &str) -> String {
    match trimmed(args.title.as_deref()) {
        Some(title) => one_line(title),
        None => problem.chars().take(TITLE_CHARS).collect(),
    }
}

/// The approved investigations most like a new one, best first, at most `SIMILAR` of them
/// (select::top with the query's tags and title words, widened by the glossary's spellings).
fn similar_to(ctx: &Ctx, query: &Query) -> Vec<Similar> {
    let (items, _warnings) = store::all(&ctx.dossier);
    let investigations: Vec<Item> = items
        .into_iter()
        .filter(|item| {
            item.entry.kind() == Some(Kind::Investigation)
                && item.entry.status() == Some(Status::Approved)
        })
        .collect();
    let widened = select::widened(query, &glossary_terms(ctx));
    select::top(&investigations, &widened, SIMILAR)
        .into_iter()
        .map(|item| Similar {
            id: item.entry.id(),
            state: item.entry.field("state").unwrap_or_default(),
            title: item.entry.title(),
        })
        .collect()
}

/// The terms of the glossary of the dossier in file order (format.md section 20). A glossary that
/// cannot be read gives no terms, so the similar investigations are chosen without them.
fn glossary_terms(ctx: &Ctx) -> Vec<Term> {
    match glossary::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => glossary::terms(&file),
        Err(_) => Vec::new(),
    }
}

/// The values of the tag options, each comma-separated value trimmed; blank values are left out.
fn tag_values(args: &Args) -> Vec<String> {
    [&args.machine, &args.analysis, &args.software, &args.tags]
        .into_iter()
        .filter_map(|value| value.as_deref())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect()
}

/// The text with its ends trimmed, or None when it is not given or blank.
fn trimmed(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// The text on one line: each line trimmed, the non-empty lines joined with a space.
fn one_line(text: &str) -> String {
    text.split(['\r', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A field or section as a pair of owned strings.
fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}
