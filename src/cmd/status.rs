//! The `status` command: the state of the whole project for a person or an LLM who asks "where are
//! we" (R-CTX-9 to R-CTX-11, R-READ-8). The task parts are read from `todo` (its summary and its task
//! list); without `todo` they are one line, and a part whose todo call fails is left out. `--since <date>` prints what was recorded on or after the date, and `--since last` what is
//! new since the person last ran it (R-READ-10). Only a person's run moves that marker: the LLMs of
//! a person share the person's marker and never move it.

use crate::approval::{self, Mark};
use crate::atomic;
use crate::cli::Global;
use crate::config::Tags;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::journal::{self, Record};
use crate::names::{self, Who};
use crate::output::Output;
use crate::store::{self, Item};
use crate::{time, todo_bridge};
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The most findings the screen names.
const FINDINGS: usize = 5;

/// The most lines of the Tasks part: its heading and its counts line included.
const TASKS_LINES: usize = 20;

/// The most milestone lines of the Tasks part. The heading and the counts line take two of the
/// lines; a "more" line takes its place when there are more milestones than this.
const MILESTONE_LINES: usize = TASKS_LINES - 2;

/// The most tasks the Next part names.
const NEXT_TASKS: usize = 5;

/// The arguments of `status`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only what was recorded on or after this date, such as 2026-11-20; `last` means since the
    /// person's last run
    #[arg(long)]
    pub since: Option<String>,
    /// Only the todo milestone of this task, such as T-130, and the entries of its tasks
    #[arg(long)]
    pub milestone: Option<String>,
}

/// Runs `status`: the screen of the project, or with `--since` what was recorded since.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "status")?;
    let milestone = milestone_of(args.milestone.as_deref());
    let (all, mut warnings) = store::all(&ctx.dossier);
    let mut screen = match args.since.as_deref() {
        Some(text) => since_screen(&ctx, parse_since(text)?, &all, milestone.as_deref())?,
        None => project_screen(&ctx, &all, milestone.as_deref()),
    };
    if let Some(marker) = screen.marker.take() {
        if ctx.dry_run {
            screen.warnings.push(format!(
                "dry run: would write {}",
                shown_path(&ctx.root, &marker.path)
            ));
        } else {
            atomic::replace(&marker.path, format!("{}\n", marker.stamp).as_bytes(), None)?;
        }
    }
    warnings.append(&mut screen.warnings);
    fit(&mut screen.sections, ctx.cfg.status_lines);
    let mut output = Output::with(render(&screen.sections), screen.data);
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// What the screen is made of: its parts, the data of `--json`, warnings, and the marker of a
/// person's run that is to be written.
struct Screen {
    sections: Vec<Section>,
    data: Value,
    warnings: Vec<String>,
    marker: Option<Marker>,
}

/// The marker of a person's run: the file to write and the time to write into it.
struct Marker {
    path: PathBuf,
    stamp: String,
}

/// One part of the screen: an optional heading and its lines. A part that is cuttable gives up
/// its last lines when the screen is longer than the limit; `hidden` counts them, and the screen
/// then ends the part with `<m> more: <hint>`.
struct Section {
    heading: Option<String>,
    lines: Vec<String>,
    cuttable: bool,
    hidden: usize,
    hint: String,
}

impl Section {
    /// Lines without a heading that are never cut.
    fn plain(lines: Vec<String>) -> Section {
        Section {
            heading: None,
            lines,
            cuttable: false,
            hidden: 0,
            hint: String::new(),
        }
    }

    /// A heading with lines that are never cut.
    fn headed(heading: &str, lines: Vec<String>) -> Section {
        Section {
            heading: Some(heading.to_string()),
            lines,
            cuttable: false,
            hidden: 0,
            hint: String::new(),
        }
    }

    /// A heading with a list that may be cut. An empty list shows `empty` instead, and is never
    /// cut.
    fn list(heading: &str, lines: Vec<String>, empty: &str, hint: &str) -> Section {
        let cuttable = !lines.is_empty();
        let lines = if cuttable {
            lines
        } else {
            vec![empty.to_string()]
        };
        Section {
            heading: Some(heading.to_string()),
            lines,
            cuttable,
            hidden: 0,
            hint: hint.to_string(),
        }
    }

    /// How many lines the part takes on the screen.
    fn height(&self) -> usize {
        usize::from(self.heading.is_some()) + self.lines.len() + usize::from(self.hidden > 0)
    }
}

/// Cuts the longest cuttable part, one line at a time, until the screen has at most `limit`
/// lines. A screen that cannot be cut enough keeps its length.
fn fit(sections: &mut [Section], limit: usize) {
    while sections.iter().map(Section::height).sum::<usize>() > limit {
        let mut longest: Option<usize> = None;
        for (index, section) in sections.iter().enumerate() {
            let longer =
                longest.is_none_or(|best| section.lines.len() > sections[best].lines.len());
            if section.cuttable && !section.lines.is_empty() && longer {
                longest = Some(index);
            }
        }
        let Some(index) = longest else {
            break;
        };
        sections[index].lines.pop();
        sections[index].hidden += 1;
    }
}

/// The text of the screen: each heading, its lines, and the `more` line of a cut part.
fn render(sections: &[Section]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for section in sections {
        if let Some(heading) = &section.heading {
            lines.push(heading.clone());
        }
        lines.extend(section.lines.iter().cloned());
        if section.hidden > 0 {
            lines.push(format!("{} more: {}", section.hidden, section.hint));
        }
    }
    lines.join("\n")
}

/// The milestone given with `--milestone`, in upper case; None when none is given.
fn milestone_of(text: Option<&str>) -> Option<String> {
    text.map(|text| text.trim().to_uppercase())
        .filter(|text| !text.is_empty())
}

/// Whether an entry belongs to the milestone asked for (its `task` field equals the task), or
/// every entry does when no milestone is asked for.
fn in_scope(item: &Item, milestone: Option<&str>) -> bool {
    milestone.is_none_or(|wanted| {
        item.entry
            .field("task")
            .is_some_and(|task| task.eq_ignore_ascii_case(wanted))
    })
}

/// The screen without `--since`: the project and its tags, the tasks, who works on what and what
/// waits (from todo's list), the decisions in force, the newest findings, what is unsettled, and
/// what is next.
fn project_screen(ctx: &Ctx, all: &[Item], milestone: Option<&str>) -> Screen {
    let mut sections = vec![Section::plain(vec![format!("Project {}", ctx.cfg.code)])];
    if let Some(line) = tags_line(&ctx.cfg.tags) {
        sections.push(Section::plain(vec![line]));
    }
    let available = todo_bridge::available();
    let tasks = task_part(ctx, milestone, available, &mut sections);
    let live = live_part(ctx, milestone, available);
    let claimed: Vec<&Listed> = live
        .list
        .iter()
        .flatten()
        .filter(|task| task.status == "claimed")
        .collect();
    let blocked: Vec<&Listed> = live
        .list
        .iter()
        .flatten()
        .filter(|task| task.status == "blocked")
        .collect();
    let questions: Vec<&Listed> = live.questions.iter().flatten().collect();
    let next: Vec<&Listed> = live
        .list
        .iter()
        .flatten()
        .filter(|task| task.ready)
        .take(NEXT_TASKS)
        .collect();

    if !claimed.is_empty() {
        let lines = claimed
            .iter()
            .map(|task| format!("{} · {} ({})", task.id, task.title, task.owner))
            .collect();
        sections.push(Section::list("## Working now", lines, "none", "todo list"));
    }
    let waiting: Vec<String> = blocked
        .iter()
        .map(|task| format!("{} · {} (blocked)", task.id, task.title))
        .chain(
            questions
                .iter()
                .map(|task| format!("{} · {} (question)", task.id, task.title)),
        )
        .collect();
    if !waiting.is_empty() {
        sections.push(Section::list("## Waiting", waiting, "none", "todo list"));
    }

    let scope: Vec<&Item> = all
        .iter()
        .filter(|item| in_scope(item, milestone))
        .collect();
    let decisions = approved(&scope, Kind::Decision);
    let findings: Vec<&Item> = approved(&scope, Kind::Finding)
        .into_iter()
        .take(FINDINGS)
        .collect();
    let counts = count_unsettled(&scope, &ctx.state);

    sections.push(Section::list(
        "## Decisions in force",
        decisions.iter().copied().map(entry_line).collect(),
        "none",
        "dossier list",
    ));
    sections.push(Section::list(
        "## Newest findings",
        findings.iter().copied().map(entry_line).collect(),
        "none",
        "dossier list",
    ));
    sections.push(Section::headed("## Unsettled", unsettled_lines(&counts)));
    if !next.is_empty() {
        let lines = next
            .iter()
            .map(|task| format!("{} · {}", task.id, task.title))
            .collect();
        sections.push(Section::list("## Next", lines, "none", "todo list"));
    }

    // The JSON keys are null when the todo call behind them failed or todo is not installed.
    let working = if live.list.is_some() {
        json!(task_ids(&claimed))
    } else {
        Value::Null
    };
    let mut waiting_ids = task_ids(&blocked);
    waiting_ids.extend(task_ids(&questions));
    let waiting = if live.list.is_some() || live.questions.is_some() {
        json!(waiting_ids)
    } else {
        Value::Null
    };
    let next_ids = if live.list.is_some() {
        json!(task_ids(&next))
    } else {
        Value::Null
    };

    let data = json!({
        "code": ctx.cfg.code,
        "tasks": tasks,
        "working": working,
        "waiting": waiting,
        "next": next_ids,
        "decisions": ids(&decisions),
        "findings": ids(&findings),
        "unsettled": {
            "drafts": counts.drafts,
            "open_assumptions": counts.open_assumptions,
            "changed_lessons": counts.changed_lessons,
        },
    });
    Screen {
        sections,
        data,
        warnings: Vec::new(),
        marker: None,
    }
}

/// The Tasks part, from todo's summary. Without todo, or when its summary fails, one line says
/// so and the part is left out. Returns the summary for the JSON data (null when there is none).
fn task_part(
    ctx: &Ctx,
    milestone: Option<&str>,
    available: bool,
    sections: &mut Vec<Section>,
) -> Value {
    if !available {
        sections.push(Section::plain(vec![
            "tasks: todo is not installed".to_string(),
        ]));
        return Value::Null;
    }
    let mut summary = match todo_bridge::json(&ctx.root, &["summary"]) {
        Ok(summary) => summary,
        Err(error) => {
            sections.push(Section::plain(vec![format!(
                "tasks: todo summary failed: {}",
                error.message
            )]));
            return Value::Null;
        }
    };

    let milestones: Vec<Value> = summary
        .get("milestones")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter(|milestone_value| {
                    milestone.is_none_or(|wanted| {
                        text(milestone_value, "id").eq_ignore_ascii_case(wanted)
                    })
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let mut lines: Vec<String> = milestones.iter().map(milestone_line).collect();
    let mut hidden = 0;
    if lines.len() > MILESTONE_LINES {
        hidden = lines.len() - (MILESTONE_LINES - 1);
        lines.truncate(MILESTONE_LINES - 1);
    }
    let mut section = Section::list("## Tasks", lines, "none", "todo summary");
    section.hidden = hidden;
    sections.push(section);

    // The counts describe the whole project, so they are left out when one milestone is asked for.
    if milestone.is_none() {
        sections.push(Section::plain(vec![format!(
            "open {}, claimed {}, blocked {}, done {}",
            count(&summary, "open"),
            count(&summary, "claimed"),
            count(&summary, "blocked"),
            count(&summary, "done"),
        )]));
    } else if let Some(object) = summary.as_object_mut() {
        object.insert("milestones".to_string(), Value::Array(milestones));
        object.remove("counts");
    }
    summary
}

/// The project's tags as one line, `tags: machine <list>; analysis <list>; software <list>`. A kind
/// with no values is left out, and there is no line when all kinds are empty.
fn tags_line(tags: &Tags) -> Option<String> {
    let parts: Vec<String> = [
        ("machine", &tags.machine),
        ("analysis", &tags.analysis),
        ("software", &tags.software),
    ]
    .into_iter()
    .filter(|(_, values)| !values.is_empty())
    .map(|(name, values)| format!("{name} {}", values.join(", ")))
    .collect();
    (!parts.is_empty()).then(|| format!("tags: {}", parts.join("; ")))
}

/// A task of todo's list, as far as the screen shows it.
struct Listed {
    id: String,
    title: String,
    status: String,
    owner: String,
    ready: bool,
}

/// What the screen takes from todo's task list: the tasks, and the open questions. None for a
/// call that failed, or for both when todo is not available; the part that needs it is then
/// left out.
struct Live {
    list: Option<Vec<Listed>>,
    questions: Option<Vec<Listed>>,
}

/// Reads todo's task list and its open questions, when todo is available. A call that fails
/// gives None for its part, and no error.
fn live_part(ctx: &Ctx, milestone: Option<&str>, available: bool) -> Live {
    if !available {
        return Live {
            list: None,
            questions: None,
        };
    }
    Live {
        list: todo_bridge::json(&ctx.root, &["list"])
            .ok()
            .map(|data| listed(&data, milestone)),
        questions: todo_bridge::json(&ctx.root, &["list", "--kind", "question"])
            .ok()
            .map(|data| listed(&data, milestone)),
    }
}

/// The tasks of a todo list's data that belong to the milestone asked for, by their `milestone`
/// field (every task when no milestone is asked for). A task without that field belongs to no
/// milestone.
fn listed(data: &Value, milestone: Option<&str>) -> Vec<Listed> {
    data.get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
                .filter(|task| {
                    milestone
                        .is_none_or(|wanted| text(task, "milestone").eq_ignore_ascii_case(wanted))
                })
                .map(|task| Listed {
                    id: text(task, "id").to_string(),
                    title: text(task, "title").to_string(),
                    status: text(task, "status").to_string(),
                    owner: text(task, "owner").to_string(),
                    ready: task.get("ready").and_then(Value::as_bool) == Some(true),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The IDs of tasks, in their order.
fn task_ids(tasks: &[&Listed]) -> Vec<String> {
    tasks.iter().map(|task| task.id.clone()).collect()
}

/// One milestone of the Tasks part: `<id> · <title> · <done>/<total> done`.
fn milestone_line(milestone: &Value) -> String {
    format!(
        "{} · {} · {}/{} done",
        text(milestone, "id"),
        text(milestone, "title"),
        number(milestone.pointer("/progress/done")),
        number(milestone.pointer("/progress/total")),
    )
}

/// One count of todo's summary, or `?` when todo does not give it.
fn count(summary: &Value, key: &str) -> String {
    number(summary.get("counts").and_then(|counts| counts.get(key)))
}

/// A text field of a JSON object, or the empty text when it is missing.
fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

/// A whole number, or `?` when the value is missing or is not one.
fn number(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_u64)
        .map_or_else(|| "?".to_string(), |number| number.to_string())
}

/// The approved entries of `kind` in `scope`, newest first: by when they were approved, then by
/// number.
fn approved<'a>(scope: &[&'a Item], kind: Kind) -> Vec<&'a Item> {
    let mut found: Vec<&Item> = scope
        .iter()
        .copied()
        .filter(|item| {
            item.entry.kind() == Some(kind) && item.entry.status() == Some(Status::Approved)
        })
        .collect();
    found.sort_by(|a, b| {
        took_effect(b)
            .cmp(&took_effect(a))
            .then_with(|| number_of(&b.entry.id()).cmp(&number_of(&a.entry.id())))
    });
    found
}

/// When an entry took effect: the time it was approved (`valid-from`), or its creation when that
/// is missing.
fn approval_stamp(item: &Item) -> String {
    item.entry
        .field("valid-from")
        .or_else(|| item.entry.field("created"))
        .unwrap_or_default()
}

/// `approval_stamp` as a time, None when it is not one.
fn took_effect(item: &Item) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(&approval_stamp(item)).ok()
}

/// The number of an ID: 12 for DEC-012, and 0 when there is none.
fn number_of(id: &str) -> u64 {
    let digits: String = id
        .split_once('-')
        .map_or("", |(_, rest)| rest)
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or(0)
}

/// One entry as `<ID> · <title> (<author>, <date>)`.
fn entry_line(item: &Item) -> String {
    let stamp = approval_stamp(item);
    format!(
        "{} · {} ({}, {})",
        item.entry.id(),
        item.entry.title(),
        item.entry.field("author").unwrap_or_default(),
        time::date(&stamp),
    )
}

/// The IDs of the entries, in their order.
fn ids(items: &[&Item]) -> Vec<String> {
    items.iter().map(|item| item.entry.id()).collect()
}

/// What is waiting for a person to settle.
#[derive(Default)]
struct Unsettled {
    drafts: usize,
    open_assumptions: usize,
    changed_lessons: usize,
}

/// Counts the drafts, the approved assumptions that are still open, and the approved lessons
/// whose file was changed by hand after their approval.
fn count_unsettled(scope: &[&Item], state: &Path) -> Unsettled {
    let mut counts = Unsettled::default();
    for item in scope {
        let status = item.entry.status();
        if status == Some(Status::Draft) {
            counts.drafts += 1;
        }
        if status != Some(Status::Approved) {
            continue;
        }
        match item.entry.kind() {
            Some(Kind::Assumption) if item.entry.field("state").as_deref() == Some("open") => {
                counts.open_assumptions += 1;
            }
            Some(Kind::Lesson)
                if matches!(
                    approval::mark(state, &item.entry.id(), &item.raw),
                    Mark::Changed
                ) =>
            {
                counts.changed_lessons += 1;
            }
            _ => {}
        }
    }
    counts
}

/// The lines under `## Unsettled`: one per count that is not zero.
fn unsettled_lines(counts: &Unsettled) -> Vec<String> {
    let mut lines = Vec::new();
    if counts.drafts > 0 {
        lines.push(format!("{} drafts waiting", counts.drafts));
    }
    if counts.open_assumptions > 0 {
        lines.push(format!("{} open assumptions", counts.open_assumptions));
    }
    if counts.changed_lessons > 0 {
        lines.push(format!(
            "{} approved lessons changed by hand",
            counts.changed_lessons
        ));
    }
    if lines.is_empty() {
        lines.push("nothing waiting".to_string());
    }
    lines
}

/// What `--since` asks for, as typed.
enum Since {
    /// Everything on or after this date (YYYY-MM-DD).
    Date(String),
    /// Everything since the person's last run.
    Last,
}

/// Which journal records `--since` keeps.
enum Bound {
    /// Every record: the person has never run `status --since last`.
    Everything,
    /// The records on or after this date (YYYY-MM-DD).
    Date(String),
    /// The records after this time: the person's last run.
    After(String),
}

impl Bound {
    /// Whether a record made at `stamp` is kept.
    fn keeps(&self, stamp: &str) -> bool {
        match self {
            Bound::Everything => true,
            Bound::Date(date) => time::date(stamp) >= date.as_str(),
            Bound::After(last) => {
                match (
                    DateTime::parse_from_rfc3339(stamp),
                    DateTime::parse_from_rfc3339(last),
                ) {
                    (Ok(stamp), Ok(last)) => stamp > last,
                    _ => false,
                }
            }
        }
    }

    /// The `since` value of the JSON data.
    fn value(&self) -> Value {
        match self {
            Bound::Everything => Value::Null,
            Bound::Date(date) => json!(date),
            Bound::After(last) => json!(last),
        }
    }
}

/// Reads `--since`: `last`, or a date in the form 2026-11-20.
fn parse_since(text: &str) -> Result<Since> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("last") {
        return Ok(Since::Last);
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .map(|date| Since::Date(date.format("%Y-%m-%d").to_string()))
        .map_err(|_| {
            Error::new(
                Code::BadValue,
                format!("--since takes a date such as 2026-11-20, or last, not {text:?}"),
            )
        })
}

/// The screen with `--since`: the journal records that `bound` keeps, newest first, with the
/// title of each entry. With `last`, the person's marker is read, and written when a person runs
/// the command.
fn since_screen(ctx: &Ctx, since: Since, all: &[Item], milestone: Option<&str>) -> Result<Screen> {
    let mut warnings = Vec::new();
    let mut marker = None;
    let (heading, bound) = match since {
        Since::Date(date) => (format!("## Since {date}"), Bound::Date(date)),
        Since::Last => {
            let who = ctx.writer()?;
            let path = ctx
                .state
                .join(format!("seen-{}.txt", who.person.to_lowercase()));
            let last = read_marker(&ctx.root, &path, &mut warnings);
            if !who.is_llm {
                marker = Some(Marker {
                    path,
                    stamp: time::now(),
                });
            }
            match last {
                Some(stamp) => (format!("## Since {stamp}"), Bound::After(stamp)),
                None => (
                    "## Since the start (no earlier run)".to_string(),
                    Bound::Everything,
                ),
            }
        }
    };

    let index: HashMap<String, &Item> = all.iter().map(|item| (item.entry.id(), item)).collect();
    let mut events: Vec<Record> = journal::read(&ctx.state)?
        .into_iter()
        .filter(|record| bound.keeps(&record.time))
        .filter(|record| {
            index
                .get(&record.id.to_uppercase())
                .map_or(milestone.is_none(), |item| in_scope(item, milestone))
        })
        .collect();
    events.reverse();

    let lines: Vec<String> = events
        .iter()
        .map(|record| event_line(record, &index))
        .collect();
    let events_json =
        serde_json::to_value(&events).map_err(|error| Error::new(Code::Io, error.to_string()))?;
    Ok(Screen {
        sections: vec![
            Section::plain(vec![format!("Project {}", ctx.cfg.code)]),
            Section::list(&heading, lines, "none", "dossier log"),
        ],
        data: json!({"since": bound.value(), "events": events_json}),
        warnings,
        marker,
    })
}

/// One journal record as `log` prints it (T-118): time, who, the LLM, event, ID and title.
fn event_line(record: &Record, index: &HashMap<String, &Item>) -> String {
    let who = Who {
        name: record.name.clone(),
        person: record.person.clone(),
        is_llm: record.llm,
    };
    let title = index
        .get(&record.id.to_uppercase())
        .map_or_else(|| "(removed)".to_string(), |item| item.entry.title());
    format!(
        "{}  {}  {}  {}  {}",
        record.time,
        names::label(&who),
        record.event,
        record.id,
        title
    )
}

/// The time in a person's marker file. None when there is no file, or when it does not hold a
/// time; then a warning says what is shown instead.
fn read_marker(root: &Path, path: &Path, warnings: &mut Vec<String>) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let stamp = text.trim();
            if time::valid(stamp) {
                Some(stamp.to_string())
            } else {
                warnings.push(format!(
                    "{} does not hold a time; showing everything",
                    shown_path(root, path)
                ));
                None
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            warnings.push(format!(
                "{}: {error}; showing everything",
                shown_path(root, path)
            ));
            None
        }
    }
}

/// A path as people see it: relative to the project folder, with `/` as the separator.
fn shown_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A section with `count` lines, each named after its index.
    fn section(heading: &str, count: usize, cuttable: bool) -> Section {
        Section {
            heading: Some(heading.to_string()),
            lines: (0..count).map(|index| format!("line {index}")).collect(),
            cuttable,
            hidden: 0,
            hint: "dossier list".to_string(),
        }
    }

    #[test]
    fn fit_cuts_the_longest_part_and_says_how_many_are_left_out() {
        let mut sections = vec![section("## A", 2, true), section("## B", 5, true)];
        fit(&mut sections, 8);
        assert_eq!(sections[0].hidden, 0);
        assert_eq!(sections[1].hidden, 2);
        let text = render(&sections);
        assert_eq!(
            text,
            "## A\nline 0\nline 1\n## B\nline 0\nline 1\nline 2\n2 more: dossier list"
        );
        assert_eq!(text.lines().count(), 8);
    }

    #[test]
    fn fit_leaves_a_screen_alone_when_it_fits_and_keeps_parts_that_cannot_be_cut() {
        let mut sections = vec![section("## Fixed", 4, false)];
        fit(&mut sections, 2);
        assert_eq!(sections[0].hidden, 0);
        assert_eq!(sections[0].lines.len(), 4);

        let mut sections = vec![section("## Short", 1, true)];
        fit(&mut sections, 10);
        assert_eq!(sections[0].hidden, 0);
    }

    #[test]
    fn parse_since_takes_a_date_or_the_word_last_only() {
        assert!(matches!(parse_since("last"), Ok(Since::Last)));
        assert!(matches!(parse_since(" LAST "), Ok(Since::Last)));
        match parse_since("2026-1-5") {
            Ok(Since::Date(date)) => assert_eq!(date, "2026-01-05"),
            _ => panic!("a date is read"),
        }
        let error = parse_since("yesterday").err().expect("refused");
        assert_eq!(error.code, Code::BadValue);
    }

    #[test]
    fn bound_keeps_records_on_or_after_a_date_and_after_a_time() {
        let date = Bound::Date("2026-11-21".to_string());
        assert!(!date.keeps("2026-11-20T23:59:59+01:00"));
        assert!(date.keeps("2026-11-21T00:00:00+01:00"));

        let after = Bound::After("2026-11-21T09:00:00+01:00".to_string());
        assert!(!after.keeps("2026-11-21T09:00:00+01:00"));
        assert!(after.keeps("2026-11-21T09:00:01+01:00"));
        // The same instant in UTC is the same moment.
        assert!(!after.keeps("2026-11-21T08:00:00+00:00"));
        assert!(Bound::Everything.keeps("not a time"));
    }

    #[test]
    fn number_of_reads_the_digits_after_the_prefix() {
        assert_eq!(number_of("DEC-012"), 12);
        assert_eq!(number_of("FND-1000"), 1000);
        assert_eq!(number_of("DEC"), 0);
    }

    #[test]
    fn shown_path_is_relative_to_the_project_with_slashes() {
        let root = Path::new("projects").join("Projekt Äpfel");
        let path = root.join("DOSSIER").join(".state").join("seen-anna.txt");
        assert_eq!(shown_path(&root, &path), "DOSSIER/.state/seen-anna.txt");
    }
}
