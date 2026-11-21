//! The `brief` command: what an LLM should know at the start of a session in this project
//! (R-CTX-1), or with `--task` what matters for one task (R-CTX-2, see `brief_task`). Only
//! approved entries are printed (R-CTX-4). The output stays within `brief_lines` lines and is
//! the same on every run. Nothing is changed.
//!
//! The project brief also prints the map of the code (R-CODE-1), the key values (R-VALUE-4), the
//! decisions proposed and waiting for a decider (R-CTX-16), and what rests on a wrong assumption
//! (R-CTX-16): the entries and file blocks marked `rests on <ID>`, under their own heading. With
//! `--on <date>` it prints what was in force on that day instead (R-LIFE-9); the proposals, the
//! drafts and what rests on a wrong assumption are then left out.

use crate::approval::{self, Mark};
use crate::blockfile::BlockFile;
use crate::brief_task;
use crate::cli::Global;
use crate::config::Tags;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::mapfile;
use crate::output::Output;
use crate::store::{self, Item};
use crate::time;
use crate::values;
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::{MAIN_SEPARATOR, Path};

/// The arguments of `brief`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// What matters for one todo task, such as T-042, instead of the whole project
    #[arg(long)]
    pub task: Option<String>,
    /// The day, YYYY-MM-DD: what was in force on that day instead of now (R-LIFE-9)
    #[arg(long, value_name = "date", conflicts_with = "task")]
    pub on: Option<String>,
}

/// The headings of the project brief, in the order they are printed.
const MAP: &str = "## Map";
const KEY_VALUES: &str = "## Key values";
const DECISIONS: &str = "## Decisions in force";
const ASSUMPTIONS: &str = "## Open assumptions";
const FINDINGS: &str = "## Findings";
const LESSONS: &str = "## Lessons";
const PROPOSALS: &str = "## Proposals waiting";
const RESTS: &str = "## Rests on a wrong assumption";

/// The most lines of the map the brief prints (R-CODE-1), and the most values (R-VALUE-4).
const MAX_MAP_LINES: usize = 25;
const MAX_VALUES: usize = 15;

/// The field that names the wrong assumptions an entry rests on (format.md §14), and the field
/// that gives the time an assumption was found wrong.
const RESTS_ON: &str = "rests-on";
const REFUTED: &str = "refuted";

/// The variable that is set when the command was started through ssh (R-REMOTE-12).
const SSH_VARIABLE: &str = "SSH_CONNECTION";

/// Runs `brief`: the context of the project, or with `--task` the context of one task. The
/// warnings of reading the dossier are passed on.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "brief")?;
    let remote = std::env::var(SSH_VARIABLE).is_ok_and(|value| !value.is_empty());
    match &args.task {
        Some(task) => for_task(&ctx, task, remote),
        None => {
            let day = args.on.as_deref().map(day_option).transpose()?;
            for_project(&ctx, remote, day.as_deref())
        }
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

/// `brief` for the project, fitted to `brief_lines`, for the day `day` when it is given. The
/// warnings of reading the dossier are passed on with their paths shown relative to the project
/// folder. A map or values file that cannot be read gives a warning and is left out.
fn for_project(ctx: &Ctx, remote: bool, day: Option<&str>) -> Result<Output> {
    let (items, warnings) = store::all(&ctx.dossier);
    let mut problems = Vec::new();
    let map_section = match mapfile::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => map_lines(&file),
        Err(error) => {
            problems.push(error.message);
            Vec::new()
        }
    };
    let value_section = match values::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => key_values(&file, day),
        Err(error) => {
            problems.push(error.message);
            Vec::new()
        }
    };
    let rests_section = match day {
        Some(_) => Vec::new(),
        None => {
            let (lines, found) = rests_lines(&ctx.dossier, &items);
            problems.extend(found);
            lines
        }
    };
    let mut brief = Brief::collect(
        ctx,
        &items,
        remote,
        day,
        map_section,
        value_section,
        rests_section,
    );
    let lines = brief.fit(ctx.cfg.brief_lines);
    let mut output = Output::with(lines.join("\n"), brief.data(&ctx.cfg.code));
    for warning in warnings.iter().chain(&problems) {
        output = output.warn(shown_warning(&ctx.root, warning));
    }
    Ok(output)
}

/// What the project brief holds before it is fitted to the line limit.
struct Brief {
    /// The project line (or the server line in remote mode) and the tags line.
    head: Vec<String>,
    /// The lines of the map, at most MAX_MAP_LINES; empty when there is no map or it was cut.
    map: Vec<String>,
    /// The values in force: key values first, then by name, at most MAX_VALUES.
    values: Vec<Line>,
    /// The decisions in force (approved, or with `--on` in force on the day), newest first.
    decisions: Vec<Decision>,
    /// The assumptions that are open, by ID.
    assumptions: Vec<Line>,
    /// The findings, by ID.
    findings: Vec<Line>,
    /// The lessons, by ID.
    lessons: Vec<Line>,
    /// The decisions proposed and waiting for a decider, by ID. Left out with `--on`.
    proposals: Vec<Line>,
    /// What rests on a wrong assumption (R-CTX-16): the approved entries, then the file blocks.
    /// Left out with `--on`.
    rests: Vec<Line>,
    /// The number of drafts waiting for approval. Left out with `--on`.
    drafts: usize,
    /// The number of entries cut to fit the limit (lessons, findings, decisions, assumptions).
    left_out: usize,
    /// The number of proposals cut to fit the limit.
    proposals_left_out: usize,
    /// The number of lines of what rests on a wrong assumption cut to fit the limit.
    rests_left_out: usize,
    /// The number of values cut to fit the limit.
    values_left_out: usize,
    /// Whether the map was cut to fit the limit.
    map_left_out: bool,
    /// The day of `--on`, as YYYY-MM-DD.
    day: Option<String>,
}

/// One decision: its summary line, then the bodies of its Decision and Reason sections. Shortened,
/// only the summary line is printed.
struct Decision {
    id: String,
    summary: String,
    body: Vec<String>,
    shortened: bool,
}

/// One entry or value printed as one line, with its ID or name.
struct Line {
    id: String,
    text: String,
}

/// Which wrong assumptions a line names in its `rests on` mark. With `--on` a mark names only the
/// assumptions that were found wrong on or before that day (their `refuted` date): on an earlier
/// day nobody knew yet that they were wrong.
struct Wrong {
    day: Option<String>,
    /// The day each assumption was found wrong, by ID.
    found: BTreeMap<String, String>,
}

impl Wrong {
    /// The assumptions of the dossier with their `refuted` day, for the brief of `day`.
    fn of(items: &[Item], day: Option<&str>) -> Wrong {
        let found = items
            .iter()
            .filter(|item| item.entry.kind() == Some(Kind::Assumption))
            .filter_map(|item| {
                let stamp = item.entry.field(REFUTED).unwrap_or_default();
                let date = time::date(stamp.trim());
                is_whole_date(date).then(|| (item.entry.id(), date.to_string()))
            })
            .collect();
        Wrong {
            day: day.map(str::to_string),
            found,
        }
    }

    /// The mark ` [rests on <IDs>]` of an entry, from its `rests-on` field (format.md §14), or
    /// nothing when no ID counts.
    fn mark(&self, entry: &Entry) -> String {
        let ids: Vec<String> = entry
            .list(RESTS_ON)
            .into_iter()
            .filter(|id| self.holds(id))
            .collect();
        rests_mark(&ids)
    }

    /// Whether an ID counts in the brief: every ID does for the present brief; for a day, the
    /// assumption must have been found wrong on or before it.
    fn holds(&self, id: &str) -> bool {
        match (&self.day, self.found.get(id)) {
            (None, _) => true,
            (Some(day), Some(found)) => found.as_str() <= day.as_str(),
            (Some(_), None) => false,
        }
    }
}

impl Brief {
    /// Reads the dossier into the brief, in the order it prints it. With `day`, the entries and
    /// values in force on that day; without it, the approved entries and the values now.
    fn collect(
        ctx: &Ctx,
        items: &[Item],
        remote: bool,
        day: Option<&str>,
        map_section: Vec<String>,
        value_section: Vec<Line>,
        rests_section: Vec<Line>,
    ) -> Brief {
        let wrong = Wrong::of(items, day);
        let mut decisions = shown(items, Kind::Decision, day);
        decisions.sort_by(|a, b| newest_first(&a.entry, &b.entry));
        Brief {
            head: head(ctx, remote),
            map: map_section,
            values: value_section,
            decisions: decisions
                .into_iter()
                .map(|item| decision_of(item, &wrong))
                .collect(),
            assumptions: shown(items, Kind::Assumption, day)
                .into_iter()
                .filter(|item| is_open(&item.entry, day))
                .map(|item| line(&item.entry, "", &wrong))
                .collect(),
            findings: shown(items, Kind::Finding, day)
                .into_iter()
                .map(|item| {
                    let mark = if is_dead_end(&item.entry) {
                        " (dead end)"
                    } else {
                        ""
                    };
                    line(&item.entry, mark, &wrong)
                })
                .collect(),
            lessons: shown(items, Kind::Lesson, day)
                .into_iter()
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
                    line(&item.entry, mark, &wrong)
                })
                .collect(),
            proposals: if day.is_some() {
                Vec::new()
            } else {
                proposals_of(items, &wrong)
            },
            rests: rests_section,
            drafts: if day.is_some() {
                0
            } else {
                items
                    .iter()
                    .filter(|item| item.entry.status() == Some(Status::Draft))
                    .count()
            },
            left_out: 0,
            proposals_left_out: 0,
            rests_left_out: 0,
            values_left_out: 0,
            map_left_out: false,
            day: day.map(str::to_string),
        }
    }

    /// The lines as they would be printed now: the head, then each heading with its lines (a
    /// heading with no lines is left out), the drafts line and the left-out lines.
    fn lines(&self) -> Vec<String> {
        let day = self.day.as_deref();
        let mut lines = self.head.clone();
        push_section(&mut lines, MAP, &self.map);
        push_section(&mut lines, KEY_VALUES, &texts(&self.values));
        let decisions: Vec<String> = self.decisions.iter().flat_map(Decision::lines).collect();
        push_section(&mut lines, DECISIONS, &decisions);
        push_section(&mut lines, ASSUMPTIONS, &texts(&self.assumptions));
        push_section(&mut lines, FINDINGS, &texts(&self.findings));
        push_section(&mut lines, LESSONS, &texts(&self.lessons));
        push_section(&mut lines, PROPOSALS, &texts(&self.proposals));
        push_section(&mut lines, RESTS, &texts(&self.rests));
        if self.drafts > 0 {
            lines.push(drafts_line(self.drafts));
        }
        if self.left_out > 0 {
            lines.push(left_out_text(
                self.left_out,
                "entry",
                "entries",
                &command_text("list", day),
            ));
        }
        if self.proposals_left_out > 0 {
            lines.push(left_out_text(
                self.proposals_left_out,
                "proposal",
                "proposals",
                "dossier list --status proposed",
            ));
        }
        if self.rests_left_out > 0 {
            lines.push(left_out_text(
                self.rests_left_out,
                "line resting on a wrong assumption",
                "lines resting on a wrong assumption",
                "dossier list and dossier files",
            ));
        }
        if self.values_left_out > 0 {
            lines.push(left_out_text(
                self.values_left_out,
                "value",
                "values",
                &command_text("values", day),
            ));
        }
        if self.map_left_out {
            lines.push("the map is left out: dossier map".to_string());
        }
        lines
    }

    /// Fits the brief to `limit` lines. First the decisions are shortened to their title lines,
    /// oldest first, until it fits. Then things are cut in the order of `cut_one`, and the
    /// left-out lines say what was left out. A limit too small even for that keeps the first lines.
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

    /// Cuts one thing to fit the limit, in this order: the last lesson, the last finding, the last
    /// proposal, the last line that rests on a wrong assumption, the last value, the whole map, the
    /// oldest decision, the last open assumption. Returns false when nothing is left to cut. Each
    /// step stops at the first list that had something, so only one thing is cut per call.
    fn cut_one(&mut self) -> bool {
        if self.lessons.pop().is_some() || self.findings.pop().is_some() {
            self.left_out += 1;
        } else if self.proposals.pop().is_some() {
            self.proposals_left_out += 1;
        } else if self.rests.pop().is_some() {
            self.rests_left_out += 1;
        } else if self.values.pop().is_some() {
            self.values_left_out += 1;
        } else if !self.map.is_empty() {
            self.map.clear();
            self.map_left_out = true;
        } else if self.decisions.pop().is_some() || self.assumptions.pop().is_some() {
            self.left_out += 1;
        } else {
            return false;
        }
        true
    }

    /// The data of `--json`: the day, the map lines, and the IDs (values: names) of the entries
    /// printed, and the number of drafts. A file block that rests on a wrong assumption is named
    /// by its task and path, as `<task> <path>`.
    fn data(&self, code: &str) -> Value {
        json!({
            "code": code,
            "on": self.day,
            "map": self.map,
            "values": ids(&self.values),
            "decisions": self.decisions.iter().map(|decision| decision.id.as_str()).collect::<Vec<_>>(),
            "assumptions": ids(&self.assumptions),
            "findings": ids(&self.findings),
            "lessons": ids(&self.lessons),
            "proposals": ids(&self.proposals),
            "rests": ids(&self.rests),
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

/// The entries of one kind that the brief shows, in the order of `store::all`: the approved ones,
/// or with `day` the ones in force on that day (R-LIFE-9).
fn shown<'a>(items: &'a [Item], kind: Kind, day: Option<&str>) -> Vec<&'a Item> {
    items
        .iter()
        .filter(|item| item.entry.kind() == Some(kind))
        .filter(|item| match day {
            Some(day) => in_force(&item.entry, day),
            None => item.entry.status() == Some(Status::Approved),
        })
        .collect()
}

/// The decisions waiting for a decider (status `proposed`), in the order of `store::all`, each as
/// `<ID> · <title> (<author>)`.
fn proposals_of(items: &[Item], wrong: &Wrong) -> Vec<Line> {
    items
        .iter()
        .filter(|item| {
            item.entry.kind() == Some(Kind::Decision)
                && item.entry.status() == Some(Status::Proposed)
        })
        .map(|item| line(&item.entry, &format!(" ({})", author(&item.entry)), wrong))
        .collect()
}

/// What rests on a wrong assumption, as the brief of now prints it (R-CTX-16): the approved entries
/// whose `rests-on` field names an ID, in the order of `store::all`, then the blocks of the file
/// lists that are not detached and whose `rests-on` field names an ID, the tasks in order and the
/// blocks in the order of their list. Each is one line. A list that cannot be read gives its
/// problem and no lines.
fn rests_lines(dossier: &Path, items: &[Item]) -> (Vec<Line>, Vec<String>) {
    let mut lines: Vec<Line> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter_map(|item| {
            let assumptions = item.entry.list(RESTS_ON);
            (!assumptions.is_empty()).then(|| Line {
                id: item.entry.id(),
                text: format!("{}{}", entry_text(&item.entry), rests_mark(&assumptions)),
            })
        })
        .collect();
    let mut problems = Vec::new();
    for task in filelist::tasks(dossier) {
        let (list, _) = match filelist::load(dossier, &task) {
            Ok(loaded) => loaded,
            Err(error) => {
                problems.push(error.message);
                continue;
            }
        };
        for key in list.keys() {
            let detached = list
                .field(&key, "detached")
                .is_some_and(|at| !at.is_empty());
            let assumptions = ids_of(&list.field(&key, RESTS_ON).unwrap_or_default());
            if !detached && !assumptions.is_empty() {
                let id = format!("{task} {key}");
                lines.push(Line {
                    text: format!("{id}{}", rests_mark(&assumptions)),
                    id,
                });
            }
        }
    }
    (lines, problems)
}

/// The mark ` [rests on <IDs>]` of IDs, joined with `, `; nothing when there are none.
fn rests_mark(assumptions: &[String]) -> String {
    if assumptions.is_empty() {
        String::new()
    } else {
        format!(" [rests on {}]", assumptions.join(", "))
    }
}

/// The IDs of a comma-separated field, trimmed, without the empty ones.
fn ids_of(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
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

/// The decision as printed: `<ID> · <title> (<author>, <date>)`, with its rests-on mark, and its
/// bodies.
fn decision_of(item: &Item, wrong: &Wrong) -> Decision {
    let entry = &item.entry;
    let date = entry
        .field("created")
        .map(|stamp| time::date(&stamp).to_string())
        .filter(|date| !date.is_empty())
        .unwrap_or_else(|| "-".to_string());
    Decision {
        id: entry.id(),
        summary: format!(
            "{} ({}, {date}){}",
            entry_text(entry),
            author(entry),
            wrong.mark(entry)
        ),
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

/// An entry as one line: its heading, then `suffix`, then its rests-on mark.
fn line(entry: &Entry, suffix: &str, wrong: &Wrong) -> Line {
    Line {
        id: entry.id(),
        text: format!("{}{suffix}{}", entry_text(entry), wrong.mark(entry)),
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

/// The author of an entry, or `-` when it has none.
fn author(entry: &Entry) -> String {
    entry
        .field("author")
        .filter(|author| !author.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

/// Whether an assumption is open. Now: its `state` is `open`. On a day (`--on`): it was not yet
/// confirmed on that day; a refuted one is open until its `valid-until` day, which `in_force`
/// has already checked.
fn is_open(entry: &Entry, day: Option<&str>) -> bool {
    match day {
        Some(day) => {
            let confirmed = entry.field("confirmed").unwrap_or_default();
            let date = time::date(confirmed.trim());
            !is_whole_date(date) || date > day
        }
        None => entry
            .field("state")
            .is_some_and(|state| state.eq_ignore_ascii_case("open")),
    }
}

/// Whether a finding's `outcome` is `dead end`.
fn is_dead_end(entry: &Entry) -> bool {
    entry
        .field("outcome")
        .is_some_and(|outcome| outcome.eq_ignore_ascii_case("dead end"))
}

/// Whether an entry was in force on the day `day` (YYYY-MM-DD), by the rule of filter.rs for
/// `--on` (R-LIFE-9): its valid-from date is on or before the day, and its valid-until is empty or
/// after the day. An entry with no valid-from was never in force.
fn in_force(entry: &Entry, day: &str) -> bool {
    let from = entry.field("valid-from").unwrap_or_default();
    let from = time::date(from.trim());
    if !is_whole_date(from) || from > day {
        return false;
    }
    let until = entry.field("valid-until").unwrap_or_default();
    let until = until.trim();
    if until.is_empty() {
        return true;
    }
    let end = time::date(until);
    is_whole_date(end) && end > day
}

/// A whole date as format.md §7 writes its day, `2026-11-15`: ten characters, digits, with a dash
/// at the fifth and the eighth place.
fn is_whole_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
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

/// The map lines to print (`mapfile::lines`), at most MAX_MAP_LINES. When there are more, the last
/// line says how many folders are left out and where to see them.
fn map_lines(file: &BlockFile) -> Vec<String> {
    let mut lines = mapfile::lines(file);
    if lines.len() > MAX_MAP_LINES {
        let more = lines.len() - (MAX_MAP_LINES - 1);
        lines.truncate(MAX_MAP_LINES - 1);
        lines.push(format!("{more} more folders in the map: dossier map"));
    }
    lines
}

/// The values the brief prints: the values in force, key values first and then by name, at most
/// MAX_VALUES, each as `<name> = <value>[ <unit>]`. With `day`, the values in force on that day.
fn key_values(file: &BlockFile, day: Option<&str>) -> Vec<Line> {
    let mut all = values::all(file);
    all.sort_by(|a, b| b.key.cmp(&a.key).then_with(|| a.name.cmp(&b.name)));
    all.iter()
        .filter_map(|value| {
            value_text(file, value, day).map(|text| Line {
                id: value.name.clone(),
                text: format!("{} = {text}", value.name),
            })
        })
        .take(MAX_VALUES)
        .collect()
}

/// The value as printed, `<value>[ <unit>]`: the value in force now, or with `day` the value in
/// force at the end of that day (`values::on`). None when no value is in force.
fn value_text(file: &BlockFile, value: &values::Value, day: Option<&str>) -> Option<String> {
    match day {
        Some(day) => values::on(file, &value.name, day),
        None => (!value.value.is_empty()).then(|| match &value.unit {
            Some(unit) => format!("{} {unit}", value.value),
            None => value.value.clone(),
        }),
    }
}

/// The `--on` day as YYYY-MM-DD. Code::BadValue when it is not a date of that form.
fn day_option(text: &str) -> Result<String> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
        .map(|day| day.format("%Y-%m-%d").to_string())
        .map_err(|_| {
            Error::new(
                Code::BadValue,
                format!("--on needs a date such as 2026-11-21, not \"{text}\""),
            )
        })
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

/// `dossier <command>`, with ` --on <day>` after it when the brief is for a day.
fn command_text(command: &str, day: Option<&str>) -> String {
    match day {
        Some(day) => format!("dossier {command} --on {day}"),
        None => format!("dossier {command}"),
    }
}

/// `1 <one> left out: <command>`, or `<n> <many> left out: <command>`.
fn left_out_text(n: usize, one: &str, many: &str, command: &str) -> String {
    if n == 1 {
        format!("1 {one} left out: {command}")
    } else {
        format!("{n} {many} left out: {command}")
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
