//! The `decide` command: records a decision with its reason, and who made it. A person's
//! decision is approved at once; an LLM's needs `--said-by <person>` (create::create).
//!
//! Before it writes, `decide` looks for the approved decisions related to the new one: those
//! that share tags or title words with it, by word stems and rare words (select::top). An LLM
//! must say how the new decision stands to them: `--replaces <ID>` supersedes one of them in the
//! same step, `--besides` says that it stands next to them. Without either, the LLM's decision is
//! refused and the related decisions are listed. A person is only warned (R-CTX-13).

use crate::cli::Global;
use crate::config::Config;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::glossary;
use crate::names::{self, Who};
use crate::output::Output;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::tx::{self, Change, Tx};
use serde_json::json;
use std::path::Path;

/// The longest title made from the decision, in characters.
const TITLE_CHARS: usize = 80;

/// The most related decisions that are looked for and shown.
const RELATED_LIMIT: usize = 5;

/// The arguments of `decide`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The decision: what was decided
    pub decision: String,
    /// Why it was decided
    #[arg(long)]
    pub reason: String,
    /// What was considered and rejected, and why it was not chosen
    #[arg(long)]
    pub rejected: Option<String>,
    /// The title; by default the first 80 characters of the decision
    #[arg(long)]
    pub title: Option<String>,
    /// The todo task it came from, such as T-042
    #[arg(long)]
    pub task: Option<String>,
    /// The person who made the decision; needed when an LLM records it
    #[arg(long)]
    pub said_by: Option<String>,
    /// The approved decision this one replaces; it is superseded in the same step
    #[arg(long)]
    pub replaces: Option<String>,
    /// The new decision stands next to the related ones (an LLM states this instead of --replaces)
    #[arg(long)]
    pub besides: bool,
    /// The decision binds the whole project, not one milestone; it is written with `standing: yes`
    #[arg(long)]
    pub standing: bool,
    /// The machines the decision is about, comma-separated
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
    /// The topics it is about, from the project's list (dossier topics), comma-separated
    #[arg(long)]
    pub topic: Option<String>,
    /// The code it is about: paths, each optionally with ::name, comma-separated (src/pump or src/pump/seal.rs::leak_model)
    #[arg(long)]
    pub code: Option<String>,
}

/// Runs `decide`: looks for the related decisions, refuses an LLM's decision that does not say how
/// it stands to them, then creates the decision in one transaction and, with `--replaces`,
/// supersedes the old one in the same transaction; a proposal cannot replace one, which is refused
/// before anything is written. The output is `<ID>: added (<status>)`, or
/// `<ID>: proposed (...)` when the decision is only a proposal (see `announce`), and a second line
/// `<old ID>: superseded by <ID>` when one was replaced.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if args.replaces.is_some() && args.besides {
        return Err(Error::new(
            Code::Usage,
            "--replaces and --besides cannot be given together",
        ));
    }
    let ctx = Ctx::load(g, "decide")?;
    tx::write(&ctx, |tx| {
        let who = tx.ctx().writer()?.clone();
        let replaced = args
            .replaces
            .as_deref()
            .map(|id| store::get(&tx.ctx().dossier, id))
            .transpose()?;
        if let Some(old) = &replaced {
            check_replaceable(old)?;
            if let Some(author) = proposing_author(&tx.ctx().cfg, &who, args) {
                return Err(Error::new(
                    Code::Refused,
                    proposal_replaces_refusal(&author, &old.entry.id()),
                ));
            }
        }
        let mut new = new_decision(args);
        let related = find_related(
            &tx.ctx().dossier,
            &tx.ctx().cfg.code,
            &search_query(args, &new.title),
        );

        // An LLM names the person who decided it; without one, create refuses the decision
        // itself, so the related decisions are only asked about for a decision that could be
        // recorded.
        let llm_with_said_by = who.is_llm && non_empty(args.said_by.as_deref()).is_some();
        if llm_with_said_by && !related.is_empty() && replaced.is_none() && !args.besides {
            return Err(Error::new(Code::Refused, refusal(&related)));
        }

        let replaced_id = replaced.as_ref().map(|old| old.entry.id());
        let shown: Vec<(String, String)> = related
            .iter()
            .filter(|(id, _)| Some(id.as_str()) != replaced_id.as_deref())
            .cloned()
            .collect();

        if let Some(old) = &replaced {
            new.fields.push(("supersedes".to_string(), old.entry.id()));
        }
        let (id, text) = create::create(tx, &new)?;
        let entry = Entry::parse(&text)?;
        let status = entry
            .status()
            .ok_or_else(|| Error::new(Code::Parse, format!("{id} has no status")))?;

        let mut lines = vec![announce(&id, status, &entry)?];
        if let Some(old) = replaced {
            lines.push(close_replaced(tx, old, &id, &who));
        }
        let data = json!({
            "id": id,
            "status": status.name(),
            "replaces": replaced_id,
            "related": shown
                .iter()
                .map(|(related_id, title)| json!({"id": related_id, "title": title}))
                .collect::<Vec<_>>(),
        });
        let output = Output::with(lines.join("\n"), data);
        Ok(shown.iter().fold(output, |output, (related_id, title)| {
            output.warn(format!("related: {related_id} {title}"))
        }))
    })
}

/// The first output line of a new decision. A proposal (status `proposed`: its author is not in
/// `deciders`, R-WRITE-13) says who it is by and how a decider accepts it; any other decision says
/// its status, as `added (<status>)`.
fn announce(id: &str, status: Status, entry: &Entry) -> Result<String> {
    if status != Status::Proposed {
        return Ok(format!("{id}: added ({})", status.name()));
    }
    let author = entry
        .field("author")
        .ok_or_else(|| Error::new(Code::Parse, format!("{id} has no author")))?;
    Ok(format!(
        "{id}: proposed ({author} is not in deciders; a decider accepts it with: dossier accept {id})"
    ))
}

/// The entry that `decide` asks to create: the decision and its reason, the rejected options
/// when given, the tags, topic, code, task and standing mark when given, and the person who said it.
fn new_decision(args: &Args) -> New {
    let mut sections = vec![
        ("Decision".to_string(), args.decision.clone()),
        ("Reason".to_string(), args.reason.clone()),
    ];
    if let Some(rejected) = &args.rejected {
        sections.push(("Rejected".to_string(), rejected.clone()));
    }
    let tags = [
        ("machine", &args.machine),
        ("analysis", &args.analysis),
        ("software", &args.software),
        ("tags", &args.tags),
        ("topic", &args.topic),
    ];
    let mut fields: Vec<(String, String)> = tags
        .into_iter()
        .filter_map(|(key, value)| {
            non_empty(value.as_deref()).map(|value| (key.to_string(), value))
        })
        .collect();
    if let Some(code) = code_of(args) {
        fields.push(("code".to_string(), code));
    }
    if let Some(task) = non_empty(args.task.as_deref()) {
        fields.push(("task".to_string(), task.to_uppercase()));
    }
    if args.standing {
        fields.push(("standing".to_string(), "yes".to_string()));
    }
    let title = non_empty(args.title.as_deref()).unwrap_or_else(|| default_title(&args.decision));
    New {
        kind: Kind::Decision,
        title,
        fields,
        sections,
        said_by: args.said_by.clone(),
        draft: false,
    }
}

/// The value of the code field: each pointer of `--code` trimmed, with backslashes as slashes,
/// the empty ones left out, joined with `, `; None when no pointer is left.
fn code_of(args: &Args) -> Option<String> {
    let pointers: Vec<String> = args
        .code
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|pointer| pointer.trim().replace('\\', "/"))
        .filter(|pointer| !pointer.is_empty())
        .collect();
    (!pointers.is_empty()).then(|| pointers.join(", "))
}

/// What the new decision is related to: each value of the four tag options and of `--topic`,
/// split at commas, and the words of its title and of its decision.
fn search_query(args: &Args, title: &str) -> Query {
    let tags = [
        &args.machine,
        &args.analysis,
        &args.software,
        &args.tags,
        &args.topic,
    ]
    .into_iter()
    .filter_map(|value| value.as_deref())
    .flat_map(|value| value.split(','))
    .map(str::trim)
    .filter(|tag| !tag.is_empty())
    .map(str::to_string)
    .collect();
    let words = select::words(&format!("{title} {}", args.decision));
    Query { tags, words }
}

/// The approved decisions most related to the query, best first, as (ID, title). The query is
/// widened by the project's glossary (`code` names it when it is missing); a glossary that cannot
/// be read adds no terms. The store's warnings about files it could not read are not shown here.
fn find_related(dossier: &Path, code: &str, query: &Query) -> Vec<(String, String)> {
    let (items, _warnings) = store::all(dossier);
    let approved: Vec<Item> = items
        .into_iter()
        .filter(|item| item.entry.kind() == Some(Kind::Decision))
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .collect();
    let terms = glossary::load(dossier, code)
        .map(|(file, _)| glossary::terms(&file))
        .unwrap_or_default();
    let widened = select::widened(query, &terms);
    select::top(&approved, &widened, RELATED_LIMIT)
        .into_iter()
        .map(|item| (item.entry.id(), item.entry.title()))
        .collect()
}

/// What `--replaces` needs of the old entry: a decision that is approved. Otherwise
/// Code::Refused, naming what it is.
fn check_replaceable(old: &Item) -> Result<()> {
    let id = old.entry.id();
    if old.entry.kind() != Some(Kind::Decision) {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{id} is not a decision (kind: {}); --replaces names an approved decision",
                old.entry.kind().map_or("none", Kind::name)
            ),
        ));
    }
    if old.entry.status() != Some(Status::Approved) {
        return Err(Error::new(
            Code::Refused,
            format!(
                "{id} is not an approved decision (status: {}); only an approved decision can be replaced",
                old.entry.status().map_or("none", Status::name)
            ),
        ));
    }
    Ok(())
}

/// The person a new decision is by when it would be a proposal: its author is not in `deciders`
/// (R-WRITE-13, names::may_decide). That is the writer, or for an LLM the person named with
/// `--said-by`, written as the settings write the name. None when the decision would be approved,
/// or when no such person is in the settings: create refuses those cases itself.
fn proposing_author(cfg: &Config, who: &Who, args: &Args) -> Option<String> {
    let name = if who.is_llm {
        non_empty(args.said_by.as_deref())?
    } else {
        who.person.clone()
    };
    let person = cfg
        .people
        .iter()
        .find(|person| person.name.to_lowercase() == name.to_lowercase())?;
    (!names::may_decide(cfg, &person.name)).then(|| person.name.clone())
}

/// The refusal of `--replaces` by a proposal. A proposal is not in force, so the old decision
/// stays in force until a decider accepts the proposal; then `supersede` links the two.
fn proposal_replaces_refusal(author: &str, old_id: &str) -> String {
    format!(
        "{author} is not in deciders: a proposal cannot replace {old_id}; propose it, and once it is accepted run: dossier supersede {old_id} --by <new ID>"
    )
}

/// The old decision is closed in the transaction, as `supersede` closes it: status superseded,
/// valid-until, superseded-by and a history line. Returns its output line.
fn close_replaced(tx: &mut Tx, old: Item, new_id: &str, who: &Who) -> String {
    let old_id = old.entry.id();
    let time = tx.time().to_string();
    let mut closed = old.entry.clone();
    closed.set_field("status", Status::Superseded.name());
    closed.set_field("valid-until", &time);
    closed.set_field("superseded-by", new_id);
    let detail = format!("by {new_id}");
    closed.add_history(&time, who, "superseded", &detail);
    let text = closed.text();
    tx.put(
        old.path.clone(),
        Some(&old),
        text,
        Change {
            id: old_id.clone(),
            event: "superseded".to_string(),
            detail,
        },
    );
    format!("{old_id}: superseded by {new_id}")
}

/// The refusal of an LLM's decision that stands next to related ones: the related decisions,
/// then what the LLM must state.
fn refusal(related: &[(String, String)]) -> String {
    let mut lines = vec!["related decisions in force:".to_string()];
    lines.extend(related.iter().map(|(id, title)| format!("{id} {title}")));
    lines.push("state --replaces <ID> or --besides".to_string());
    lines.join("\n")
}

/// The value without its surrounding spaces, or None when nothing is left of it.
fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// The first 80 characters of the decision (characters, not bytes), with line breaks turned into
/// spaces and the ends trimmed.
fn default_title(decision: &str) -> String {
    decision
        .chars()
        .take(TITLE_CHARS)
        .map(|c| if matches!(c, '\r' | '\n') { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}
