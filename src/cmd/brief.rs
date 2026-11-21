//! The `brief` command: what an LLM should know at the start of a session in this project
//! (R-CTX-1), or with `--task` what matters for one task (R-CTX-2, see `brief_task`). Only
//! approved entries are printed (R-CTX-4). The output stays within `brief_lines` lines and is
//! the same on every run. Nothing is changed.

use crate::approval::{self, Mark};
use crate::brief_task;
use crate::cli::Global;
use crate::config::Tags;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::Result;
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};
use std::cmp::Ordering;
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `brief`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// What matters for one todo task, such as T-042, instead of the whole project
    #[arg(long)]
    pub task: Option<String>,
}

/// The headings of the project brief.
const DECISIONS: &str = "## Decisions in force";
const ASSUMPTIONS: &str = "## Open assumptions";
const FINDINGS: &str = "## Findings";
const LESSONS: &str = "## Lessons";

/// The variable that is set when the command was started through ssh (R-REMOTE-12).
const SSH_VARIABLE: &str = "SSH_CONNECTION";

/// Runs `brief`: the context of the project, or with `--task` the context of one task. The
/// warnings of reading the dossier are passed on.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "brief")?;
    let remote = std::env::var(SSH_VARIABLE).is_ok_and(|value| !value.is_empty());
    match &args.task {
        Some(task) => for_task(&ctx, task, remote),
        None => for_project(&ctx, remote),
    }
}

/// `brief --task`: the lines of `brief_task`, after the server line when the command came
/// through ssh. The server line counts in `brief_lines`.
fn for_task(ctx: &Ctx, task: &str, remote: bool) -> Result<Output> {
    let mut lines = Vec::new();
    if remote {
        lines.push(server_line(ctx));
    }
    let limit = ctx.cfg.brief_lines.saturating_sub(lines.len());
    lines.extend(brief_task::lines(ctx, task, limit)?);
    let data = json!({"code": ctx.cfg.code, "task": task, "lines": lines});
    Ok(Output::with(lines.join("\n"), data))
}

/// `brief` for the project, fitted to `brief_lines`. The warnings of reading the dossier are
/// passed on with their paths shown relative to the project folder.
fn for_project(ctx: &Ctx, remote: bool) -> Result<Output> {
    let (items, warnings) = store::all(&ctx.dossier);
    let mut brief = Brief::collect(ctx, &items, remote);
    let lines = brief.fit(ctx.cfg.brief_lines);
    let mut output = Output::with(lines.join("\n"), brief.data(&ctx.cfg.code));
    for warning in &warnings {
        output = output.warn(shown_warning(&ctx.root, warning));
    }
    Ok(output)
}

/// What the project brief holds before it is fitted to the line limit.
struct Brief {
    /// The project line (or the server line in remote mode) and the tags line.
    head: Vec<String>,
    /// The approved decisions, newest first.
    decisions: Vec<Decision>,
    /// The approved assumptions whose state is open, by ID.
    assumptions: Vec<Line>,
    /// The approved findings, by ID.
    findings: Vec<Line>,
    /// The approved lessons, by ID.
    lessons: Vec<Line>,
    /// The number of drafts waiting for approval.
    drafts: usize,
    /// The number of entries cut to fit the limit.
    left_out: usize,
}

/// One approved decision: its summary line, then the bodies of its Decision and Reason
/// sections. Shortened, only the summary line is printed.
struct Decision {
    id: String,
    summary: String,
    body: Vec<String>,
    shortened: bool,
}

/// One entry printed as one line, with its ID.
struct Line {
    id: String,
    text: String,
}

impl Brief {
    /// Reads the approved entries of the dossier into the brief, in the order it prints them.
    fn collect(ctx: &Ctx, items: &[Item], remote: bool) -> Brief {
        let mut decisions: Vec<&Item> = approved(items, Kind::Decision).collect();
        decisions.sort_by(|a, b| newest_first(&a.entry, &b.entry));
        Brief {
            head: head(ctx, remote),
            decisions: decisions.into_iter().map(decision_of).collect(),
            assumptions: approved(items, Kind::Assumption)
                .filter(|item| is_open(&item.entry))
                .map(|item| line(&item.entry, ""))
                .collect(),
            findings: approved(items, Kind::Finding)
                .map(|item| {
                    let mark = if is_dead_end(&item.entry) {
                        " (dead end)"
                    } else {
                        ""
                    };
                    line(&item.entry, mark)
                })
                .collect(),
            lessons: approved(items, Kind::Lesson)
                .map(|item| {
                    let changed = matches!(
                        approval::mark(&ctx.state, &item.entry.id(), &item.raw),
                        Mark::Changed
                    );
                    let mark = if changed {
                        " [changed after approval]"
                    } else {
                        ""
                    };
                    line(&item.entry, mark)
                })
                .collect(),
            drafts: items
                .iter()
                .filter(|item| item.entry.status() == Some(Status::Draft))
                .count(),
            left_out: 0,
        }
    }

    /// The lines as they would be printed now: the head, then each heading with its lines (a
    /// heading with no lines is left out), the drafts line and the left-out line.
    fn lines(&self) -> Vec<String> {
        let mut lines = self.head.clone();
        let decisions: Vec<String> = self.decisions.iter().flat_map(Decision::lines).collect();
        push_section(&mut lines, DECISIONS, &decisions);
        push_section(&mut lines, ASSUMPTIONS, &texts(&self.assumptions));
        push_section(&mut lines, FINDINGS, &texts(&self.findings));
        push_section(&mut lines, LESSONS, &texts(&self.lessons));
        if self.drafts > 0 {
            lines.push(drafts_line(self.drafts));
        }
        if self.left_out > 0 {
            lines.push(left_out_line(self.left_out));
        }
        lines
    }

    /// Fits the brief to `limit` lines. First the decisions are shortened to their title lines,
    /// oldest first, until it fits. Then entries are cut in the order of `cut_one`, and the last
    /// line says how many were left out. A limit too small even for that keeps the first lines.
    fn fit(&mut self, limit: usize) -> Vec<String> {
        if self.lines().len() > limit {
            for index in (0..self.decisions.len()).rev() {
                self.decisions[index].shortened = true;
                if self.lines().len() <= limit {
                    break;
                }
            }
        }
        while self.lines().len() > limit {
            if !self.cut_one() {
                break;
            }
        }
        let mut lines = self.lines();
        lines.truncate(limit);
        lines
    }

    /// Cuts one entry: the last lesson, then the last finding, then the oldest decision, then
    /// the last open assumption. Returns false when nothing is left to cut. The `||` chain stops
    /// at the first list that had an entry, so only one entry is cut per call.
    fn cut_one(&mut self) -> bool {
        let cut = self.lessons.pop().is_some()
            || self.findings.pop().is_some()
            || self.decisions.pop().is_some()
            || self.assumptions.pop().is_some();
        if cut {
            self.left_out += 1;
        }
        cut
    }

    /// The data of `--json`: the IDs of the entries printed, and the number of drafts.
    fn data(&self, code: &str) -> Value {
        json!({
            "code": code,
            "decisions": self.decisions.iter().map(|decision| decision.id.as_str()).collect::<Vec<_>>(),
            "assumptions": ids(&self.assumptions),
            "findings": ids(&self.findings),
            "lessons": ids(&self.lessons),
            "drafts": self.drafts,
        })
    }
}

impl Decision {
    /// The lines the decision takes now.
    fn lines(&self) -> Vec<String> {
        let mut lines = vec![self.summary.clone()];
        if !self.shortened {
            lines.extend(self.body.iter().cloned());
        }
        lines
    }
}

/// The approved entries of one kind, in the order of `store::all` (by ID).
fn approved(items: &[Item], kind: Kind) -> impl Iterator<Item = &Item> {
    items.iter().filter(move |item| {
        item.entry.kind() == Some(kind) && item.entry.status() == Some(Status::Approved)
    })
}

/// The first two lines of the brief: the project line (the server line in remote mode), then
/// the tags line when there are tags.
fn head(ctx: &Ctx, remote: bool) -> Vec<String> {
    let project = if remote {
        server_line(ctx)
    } else {
        format!("Project {}", ctx.cfg.code)
    };
    let mut head = vec![project];
    head.extend(tags_line(&ctx.cfg.tags));
    head
}

/// `Project <code> on the server in <root>`, the first line in remote mode (R-REMOTE-12).
fn server_line(ctx: &Ctx) -> String {
    format!(
        "Project {} on the server in {}",
        ctx.cfg.code,
        ctx.root.display()
    )
}

/// `tags: machine <list>; analysis <list>; software <list>`, with each empty part left out, or
/// None when all three are empty.
fn tags_line(tags: &Tags) -> Option<String> {
    let parts: Vec<String> = [
        ("machine", &tags.machine),
        ("analysis", &tags.analysis),
        ("software", &tags.software),
    ]
    .into_iter()
    .filter_map(|(name, values)| {
        let values: Vec<&str> = values
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect();
        (!values.is_empty()).then(|| format!("{name} {}", values.join(", ")))
    })
    .collect();
    (!parts.is_empty()).then(|| format!("tags: {}", parts.join("; ")))
}

/// The decision as printed: `<ID> · <title> (<author>, <date>)`, with its bodies.
fn decision_of(item: &Item) -> Decision {
    let entry = &item.entry;
    let author = entry
        .field("author")
        .filter(|author| !author.is_empty())
        .unwrap_or_else(|| "-".to_string());
    let date = entry
        .field("created")
        .map(|stamp| time::date(&stamp).to_string())
        .filter(|date| !date.is_empty())
        .unwrap_or_else(|| "-".to_string());
    Decision {
        id: entry.id(),
        summary: format!("{} ({author}, {date})", entry_text(entry)),
        body: bodies(entry),
        shortened: false,
    }
}

/// The bodies of the Decision and Reason sections, each line indented by two spaces. A section
/// that is missing or empty gives no lines.
fn bodies(entry: &Entry) -> Vec<String> {
    ["Decision", "Reason"]
        .into_iter()
        .filter_map(|name| entry.section(name))
        .flat_map(|body| indented(&body))
        .collect()
}

/// Each line of `body` indented by two spaces. A blank line stays blank.
fn indented(body: &str) -> Vec<String> {
    body.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("  {line}")
            }
        })
        .collect()
}

/// An entry as one line with `suffix` after its heading.
fn line(entry: &Entry, suffix: &str) -> Line {
    Line {
        id: entry.id(),
        text: format!("{}{suffix}", entry_text(entry)),
    }
}

/// The heading of an entry: `<ID> · <title>`, or the ID alone when there is no title.
fn entry_text(entry: &Entry) -> String {
    let title = entry.title();
    if title.is_empty() {
        entry.id()
    } else {
        format!("{} · {title}", entry.id())
    }
}

/// Whether an assumption's `state` is `open`.
fn is_open(entry: &Entry) -> bool {
    entry
        .field("state")
        .is_some_and(|state| state.eq_ignore_ascii_case("open"))
}

/// Whether a finding's `outcome` is `dead end`.
fn is_dead_end(entry: &Entry) -> bool {
    entry
        .field("outcome")
        .is_some_and(|outcome| outcome.eq_ignore_ascii_case("dead end"))
}

/// The order of decisions: newest `created` first, then the higher number first, then the ID
/// in reverse. An entry with no valid `created` comes last.
fn newest_first(a: &Entry, b: &Entry) -> Ordering {
    created_at(b)
        .cmp(&created_at(a))
        .then_with(|| number(&b.id()).cmp(&number(&a.id())))
        .then_with(|| b.id().cmp(&a.id()))
}

/// The `created` field as a point in time, when it is a time stamp.
fn created_at(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry
        .field("created")
        .filter(|stamp| time::valid(stamp))
        .and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// The number of an ID: 12 for DEC-012, 0 when there is none.
fn number(id: &str) -> u64 {
    id.rsplit('-')
        .next()
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

/// The text of each line.
fn texts(lines: &[Line]) -> Vec<String> {
    lines.iter().map(|line| line.text.clone()).collect()
}

/// The IDs of each line.
fn ids(lines: &[Line]) -> Vec<&str> {
    lines.iter().map(|line| line.id.as_str()).collect()
}

/// Adds a heading and its lines, when there are any lines.
fn push_section(lines: &mut Vec<String>, heading: &str, body: &[String]) {
    if !body.is_empty() {
        lines.push(heading.to_string());
        lines.extend(body.iter().cloned());
    }
}

/// `1 draft is waiting: dossier drafts`, or `<n> drafts are waiting: dossier drafts`.
fn drafts_line(n: usize) -> String {
    if n == 1 {
        "1 draft is waiting: dossier drafts".to_string()
    } else {
        format!("{n} drafts are waiting: dossier drafts")
    }
}

/// `1 entry left out: dossier list`, or `<m> entries left out: dossier list`.
fn left_out_line(n: usize) -> String {
    if n == 1 {
        "1 entry left out: dossier list".to_string()
    } else {
        format!("{n} entries left out: dossier list")
    }
}

/// A warning of `store::all` (`<path>: <message>`) with its path shown relative to the project
/// folder, with `/` between the parts. A warning that does not start in the project folder is
/// returned as it is.
fn shown_warning(root: &Path, warning: &str) -> String {
    let root_text = root.display().to_string();
    let Some(rest) = warning.strip_prefix(root_text.as_str()) else {
        return warning.to_string();
    };
    let rest = rest.strip_prefix(['\\', '/']).unwrap_or(rest);
    match rest.split_once(": ") {
        Some((path, message)) => format!("{}: {message}", path.replace(MAIN_SEPARATOR, "/")),
        None => warning.to_string(),
    }
}
