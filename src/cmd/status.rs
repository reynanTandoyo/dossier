//! The `status` command: the state of the whole project for a person or an LLM who asks "where are
//! we" (R-CTX-9 to R-CTX-11, R-CTX-16, R-READ-8, R-EXCH-5). The task parts are read from `todo`
//! (its summary and its task list); without `todo` they are one line, and a part whose todo call
//! fails is left out. The map, the key values, the decisions in force, the open assumptions, what
//! rests on a wrong assumption, the findings, the last exchange and `--on` read the project's
//! files; a file that cannot be read is a warning and its part is left out. `--since <date>` prints
//! what was recorded on or after the date, and `--since last` what is new since the person last ran
//! it (R-READ-10). Only a person's run moves that marker: the LLMs of a person share the person's
//! marker and never move it, and `--keep` leaves it where it is; `--since <date>` never moves
//! it. The project's lines list entries only: the journal records that are not about an entry
//! (the settings, the values, the map, the exchange, the files; format.md §11) are left out. With
//! a team dossier, the project's lines are under `## Project <code>` and the team's news under
//! `## Team` (R-READ-10).
//!
//! `--on <date>` gives the decisions, findings, key values and last exchange as they were on that
//! day (R-LIFE-9), and leaves the map out. The tasks and the unsettled part (drafts, proposals,
//! open assumptions, marks) are as of now: a past day has no record of what waited on it.
//!
//! The decisions, findings, open assumptions and entries that rest on a wrong assumption carry
//! `[due for review]` when their review date has passed (R-LIFE-7). The unsettled part counts the
//! approved lessons, how-tos and tools changed by hand after their approval (approval::guarded,
//! R-LIFE-6); its JSON key stays `changed_lessons`.

use crate::approval::{self, Mark};
use crate::atomic;
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::config::Tags;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filter::Filter;
use crate::journal::{self, Record};
use crate::names::{self, Who};
use crate::output::Output;
use crate::store::{self, Item};
use crate::{exchange, filelist, mapfile, time, todo_bridge, values};
use chrono::{DateTime, FixedOffset, NaiveDate};
use clap::Parser;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The most findings the screen names.
const FINDINGS: usize = 5;

/// The most key values the screen names (R-VALUE-4): the key ones first, then by name.
const KEY_VALUES: usize = 15;

/// The most lines of the Tasks part: its heading and its counts line included.
const TASKS_LINES: usize = 20;

/// The most milestone lines of the Tasks part. The heading and the counts line take two of the
/// lines; a "more" line takes its place when there are more milestones than this.
const MILESTONE_LINES: usize = TASKS_LINES - 2;

/// The most tasks the Next part names.
const NEXT_TASKS: usize = 5;

/// The most lines of the Map part. When the map has more folders, the last of these lines says how
/// many are left out.
const MAP_LINES: usize = 10;

/// The most lines of the Team part of `--since` (R-READ-10, news_lines). When there is more news,
/// the last of these lines counts the news left out.
const TEAM_LINES: usize = 20;

/// The arguments of `status`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only what was recorded on or after this date, such as 2026-11-20; `last` means since the
    /// person's last run
    #[arg(long, conflicts_with = "on")]
    pub since: Option<String>,
    /// With `--since last`, leave the person's marker where it is, so that the same news shows
    /// again next time (R-READ-10); needs --since
    #[arg(long, requires = "since")]
    pub keep: bool,
    /// Only the todo milestone of this task, such as T-130, and the entries of its tasks
    #[arg(long)]
    pub milestone: Option<String>,
    /// The day, such as 2026-11-20: the decisions, findings, key values and last exchange as they
    /// were on it (R-LIFE-9); the tasks and the unsettled part are as of now
    #[arg(long, value_name = "date", conflicts_with = "since")]
    pub on: Option<String>,
}

/// Runs `status`: the screen of the project, or with `--since` what was recorded since.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "status")?;
    let milestone = milestone_of(args.milestone.as_deref());
    let day = args.on.as_deref().map(day_of).transpose()?;
    let (all, mut warnings) = store::all(&ctx.dossier);
    let mut screen = match args.since.as_deref() {
        Some(text) => since_screen(
            &ctx,
            parse_since(text)?,
            &all,
            milestone.as_deref(),
            args.keep,
        )?,
        None => project_screen(&ctx, &all, milestone.as_deref(), day.as_ref()),
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
/// then ends the part with `<m> more: <hint>`. A part with `first_cut` gives up its lines before
/// any other part does, so that the open assumptions and what rests on a wrong assumption are cut
/// before the decisions.
struct Section {
    heading: Option<String>,
    lines: Vec<String>,
    cuttable: bool,
    first_cut: bool,
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
            first_cut: false,
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
            first_cut: false,
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
            first_cut: false,
            hidden: 0,
            hint: hint.to_string(),
        }
    }

    /// How many lines the part takes on the screen.
    fn height(&self) -> usize {
        usize::from(self.heading.is_some()) + self.lines.len() + usize::from(self.hidden > 0)
    }
}

/// Cuts one line at a time, until the screen has at most `limit` lines: the next part is chosen by
/// `next_to_cut`. A screen that cannot be cut enough keeps its length.
fn fit(sections: &mut [Section], limit: usize) {
    while sections.iter().map(Section::height).sum::<usize>() > limit {
        let Some(index) = next_to_cut(sections) else {
            break;
        };
        sections[index].lines.pop();
        sections[index].hidden += 1;
    }
}

/// The part to cut next: the longest cuttable part with `first_cut` that still has lines, or when
/// there is none, the longest cuttable part that still has lines. Among parts of equal length the
/// first one is cut.
fn next_to_cut(sections: &[Section]) -> Option<usize> {
    longest_cuttable(sections, true).or_else(|| longest_cuttable(sections, false))
}

/// The index of the longest cuttable part that still has lines, of the parts with `first_cut`
/// only when `first_only` is set. Among parts of equal length the first one.
fn longest_cuttable(sections: &[Section], first_only: bool) -> Option<usize> {
    let mut longest: Option<usize> = None;
    for (index, section) in sections.iter().enumerate() {
        let eligible =
            section.cuttable && !section.lines.is_empty() && (section.first_cut || !first_only);
        let longer = longest.is_none_or(|best| section.lines.len() > sections[best].lines.len());
        if eligible && longer {
            longest = Some(index);
        }
    }
    longest
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

/// The first line of the screen: the project code. On a server (`SSH_CONNECTION` is set and not
/// empty, R-REMOTE-12) it also names the folder of the project there, as it is.
fn project_line(ctx: &Ctx) -> String {
    let on_server = std::env::var_os("SSH_CONNECTION").is_some_and(|value| !value.is_empty());
    if on_server {
        format!(
            "Project {} on the server in {}",
            ctx.cfg.code,
            ctx.root.display()
        )
    } else {
        format!("Project {}", ctx.cfg.code)
    }
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

/// The screen without `--since`: the project and its tags, the map (not with a day), the key
/// values, the tasks, who works on what and what waits (from todo's list), the decisions in force,
/// the open assumptions, what rests on a wrong assumption, the newest findings, what is unsettled,
/// the last exchange, and what is next. With a day (`--on`) the decisions, findings, key values and
/// exchange are those of that day.
fn project_screen(ctx: &Ctx, all: &[Item], milestone: Option<&str>, day: Option<&Day>) -> Screen {
    let mut warnings = Vec::new();
    let mut sections = vec![Section::plain(vec![project_line(ctx)])];
    if let Some(line) = tags_line(&ctx.cfg.tags) {
        sections.push(Section::plain(vec![line]));
    }
    if let Some(day) = day {
        sections.push(Section::plain(vec![format!(
            "as of {} for decisions, findings, values and exchange; the rest is now",
            day.date
        )]));
    }
    let (map_section, map_data) = match day {
        Some(_) => (None, Vec::new()),
        None => map_part(ctx, &mut warnings),
    };
    if let Some(section) = map_section {
        sections.push(section);
    }
    let shown = key_values_part(ctx, day, &mut warnings);
    if !shown.is_empty() {
        let lines = shown.iter().map(Shown::line).collect();
        sections.push(Section::list(
            "## Key values",
            lines,
            "none",
            "dossier values",
        ));
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
    let decisions = in_force(&scope, Kind::Decision, day);
    let findings: Vec<&Item> = in_force(&scope, Kind::Finding, day)
        .into_iter()
        .take(FINDINGS)
        .collect();
    let mut counts = count_unsettled(&scope, &ctx.state);
    let proposals: Vec<&Item> = scope
        .iter()
        .copied()
        .filter(|item| item.entry.status() == Some(Status::Proposed))
        .collect();
    let rests_entries = rests_on_entries(&scope);
    let rests_files = rests_on_files(ctx, milestone, &mut warnings);
    let open_assumptions: Vec<&Item> = scope
        .iter()
        .copied()
        .filter(|item| is_open_assumption(item))
        .collect();
    let resting = resting_lines(&rests_entries, &rests_files);
    counts.proposals = proposals.len();
    counts.resting = resting.len();
    let (received, delivered) = last_exchange(ctx, day, &mut warnings);

    sections.push(Section::list(
        "## Decisions in force",
        decisions.iter().copied().map(entry_line).collect(),
        "none",
        "dossier list",
    ));
    if !open_assumptions.is_empty() {
        let mut section = Section::list(
            "## Open assumptions",
            open_assumptions
                .iter()
                .copied()
                .map(|item| format!("{}{}", heading_of(item), due_mark(&item.entry)))
                .collect(),
            "none",
            "dossier list --kind assumption",
        );
        section.first_cut = true;
        sections.push(section);
    }
    if !resting.is_empty() {
        let mut section = Section::list(
            "## Rests on a wrong assumption",
            resting.iter().map(|(_, text)| text.clone()).collect(),
            "none",
            "dossier list and dossier files",
        );
        section.first_cut = true;
        sections.push(section);
    }
    sections.push(Section::list(
        "## Newest findings",
        findings.iter().copied().map(entry_line).collect(),
        "none",
        "dossier list",
    ));
    sections.push(Section::headed("## Unsettled", unsettled_lines(&counts)));
    if received.is_some() || delivered.is_some() {
        sections.push(Section::headed(
            "## Exchange",
            exchange_lines(received.as_ref(), delivered.as_ref()),
        ));
    }
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

    let values_data: Vec<Value> = shown.iter().map(Shown::data).collect();
    let rests_data = rests_on_json(&rests_entries, &rests_files);
    let resting_ids: Vec<String> = resting.iter().map(|(id, _)| id.clone()).collect();
    let data = json!({
        "code": ctx.cfg.code,
        "on": day.map(|day| day.date.as_str()),
        "tasks": tasks,
        "working": working,
        "waiting": waiting,
        "next": next_ids,
        "map": map_data,
        "values": values_data,
        "decisions": ids(&decisions),
        "open_assumptions": ids(&open_assumptions),
        "findings": ids(&findings),
        "proposals": ids(&proposals),
        "rests_on": rests_data,
        "rests": resting_ids,
        "exchange": {
            "received": received.as_ref().map(Exchanged::data),
            "delivered": delivered.as_ref().map(Exchanged::data),
        },
        "unsettled": {
            "drafts": counts.drafts,
            "open_assumptions": counts.open_assumptions,
            "changed_lessons": counts.changed,
        },
    });
    Screen {
        sections,
        data,
        warnings,
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

/// The Map part (R-CODE-1): the described folders as `mapfile::lines`, at most MAP_LINES lines.
/// When there are more, the last line says how many are left out, and the part is cut like the
/// other lists. Returns the part (None when the map has no folder, or cannot be read: then a
/// warning says so) and the lines as data, the same lines as the part with the `more` line.
fn map_part(ctx: &Ctx, warnings: &mut Vec<String>) -> (Option<Section>, Vec<String>) {
    let file = match mapfile::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => file,
        Err(error) => {
            warnings.push(error.message);
            return (None, Vec::new());
        }
    };
    let mut lines = mapfile::lines(&file);
    if lines.is_empty() {
        return (None, Vec::new());
    }
    let mut hidden = 0;
    if lines.len() > MAP_LINES {
        hidden = lines.len() - (MAP_LINES - 1);
        lines.truncate(MAP_LINES - 1);
    }
    let mut data = lines.clone();
    if hidden > 0 {
        data.push(format!("{hidden} more: dossier map"));
    }
    let mut section = Section::list("## Map", lines, "none", "dossier map");
    section.hidden = hidden;
    (Some(section), data)
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

/// Reads todo's task list and its open questions, when todo is available. With a milestone, both
/// calls name it and todo keeps the tasks below it. A call that fails gives None for its part,
/// and no error.
fn live_part(ctx: &Ctx, milestone: Option<&str>, available: bool) -> Live {
    if !available {
        return Live {
            list: None,
            questions: None,
        };
    }
    let list_args: Vec<&str> = match milestone {
        Some(id) => vec!["list", "--milestone", id],
        None => vec!["list"],
    };
    let question_args: Vec<&str> = match milestone {
        Some(id) => vec!["list", "--kind", "question", "--milestone", id],
        None => vec!["list", "--kind", "question"],
    };
    Live {
        list: todo_bridge::json(&ctx.root, &list_args)
            .ok()
            .map(|data| listed(&data)),
        questions: todo_bridge::json(&ctx.root, &question_args)
            .ok()
            .map(|data| listed(&data)),
    }
}

/// The tasks of a todo list's data, in the order todo gives them.
fn listed(data: &Value) -> Vec<Listed> {
    data.get("tasks")
        .and_then(Value::as_array)
        .map(|tasks| {
            tasks
                .iter()
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

/// The entries of `kind` in `scope` in force: the approved ones, or with a day the ones that `--on`
/// keeps on it (the filter rule of R-LIFE-9). Newest first: by when they took effect, then by
/// number.
fn in_force<'a>(scope: &[&'a Item], kind: Kind, day: Option<&Day>) -> Vec<&'a Item> {
    let mut found: Vec<&Item> = scope
        .iter()
        .copied()
        .filter(|item| {
            item.entry.kind() == Some(kind)
                && match day {
                    Some(day) => day.filter.matches(item),
                    None => item.entry.status() == Some(Status::Approved),
                }
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

/// One entry as `<ID> · <title> (<author>, <date>)`, then its due-for-review mark.
fn entry_line(item: &Item) -> String {
    let stamp = approval_stamp(item);
    format!(
        "{} · {} ({}, {}){}",
        item.entry.id(),
        item.entry.title(),
        item.entry.field("author").unwrap_or_default(),
        time::date(&stamp),
        due_mark(&item.entry),
    )
}

/// One entry as its heading: `<ID> · <title>`, or the ID alone when it has no title.
fn heading_of(item: &Item) -> String {
    let title = item.entry.title();
    if title.is_empty() {
        item.entry.id()
    } else {
        format!("{} · {title}", item.entry.id())
    }
}

/// ` [due for review]` when the `review` date of an entry (format.md §23) is before today
/// (R-LIFE-7), with today the day of the run, as `show` compares it. A review value that is not a
/// date, or no review date, gives nothing.
fn due_mark(entry: &Entry) -> &'static str {
    let now = time::now();
    let today = NaiveDate::parse_from_str(time::date(&now), "%Y-%m-%d").ok();
    let review = entry
        .field("review")
        .and_then(|text| NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok());
    if matches!((review, today), (Some(review), Some(today)) if review < today) {
        " [due for review]"
    } else {
        ""
    }
}

/// The IDs of the entries, in their order.
fn ids(items: &[&Item]) -> Vec<String> {
    items.iter().map(|item| item.entry.id()).collect()
}

/// What is waiting for a person to settle. `proposals` and `resting` are counted by the caller.
#[derive(Default)]
struct Unsettled {
    drafts: usize,
    open_assumptions: usize,
    /// The approved lessons, how-tos and tools changed by hand (approval::guarded).
    changed: usize,
    proposals: usize,
    resting: usize,
}

/// Whether an entry is an approved assumption whose `state` is `open`: the open assumptions the
/// screen lists, and the ones it counts.
fn is_open_assumption(item: &Item) -> bool {
    item.entry.kind() == Some(Kind::Assumption)
        && item.entry.status() == Some(Status::Approved)
        && item.entry.field("state").as_deref() == Some("open")
}

/// Counts the drafts, the approved assumptions that are still open, and the approved lessons,
/// how-tos and tools (approval::guarded) whose file was changed by hand after their approval.
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
            Some(Kind::Assumption) if is_open_assumption(item) => {
                counts.open_assumptions += 1;
            }
            Some(kind)
                if approval::guarded(kind)
                    && matches!(
                        approval::mark(state, &item.entry.id(), &item.raw),
                        Mark::Changed
                    ) =>
            {
                counts.changed += 1;
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
        lines.push(counted(counts.drafts, "draft waiting", "drafts waiting"));
    }
    if counts.open_assumptions > 0 {
        lines.push(counted(
            counts.open_assumptions,
            "open assumption",
            "open assumptions",
        ));
    }
    if counts.changed > 0 {
        lines.push(counted(
            counts.changed,
            "approved lesson, how-to or tool changed by hand",
            "approved lessons, how-tos or tools changed by hand",
        ));
    }
    if counts.proposals > 0 {
        lines.push(counted(
            counts.proposals,
            "proposal waiting",
            "proposals waiting",
        ));
    }
    if counts.resting > 0 {
        lines.push(counted(
            counts.resting,
            "entry or file rests on a wrong assumption",
            "entries and files rest on a wrong assumption",
        ));
    }
    if lines.is_empty() {
        lines.push("nothing waiting".to_string());
    }
    lines
}

/// `<n> <one>` for 1, `<n> <many>` otherwise.
fn counted(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// The day of `--on`: its date as YYYY-MM-DD, and the filter that keeps the entries in force on
/// it (the filter rule of R-LIFE-9, built by the filter module).
struct Day {
    date: String,
    filter: Filter,
}

/// The option `--on` alone, parsed by its own parser so that the filter module builds the filter.
#[derive(clap::Parser)]
struct OnOnly {
    #[command(flatten)]
    filter: Filter,
}

/// Reads `--on`: a date such as 2026-11-21. Code::BadValue for any other text.
fn day_of(text: &str) -> Result<Day> {
    let bad = || {
        Error::new(
            Code::BadValue,
            format!("--on takes a date such as 2026-11-21, not {text:?}"),
        )
    };
    let date = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
        .map_err(|_| bad())?
        .format("%Y-%m-%d")
        .to_string();
    let filter = OnOnly::try_parse_from(["status", "--on", date.as_str()])
        .map_err(|_| bad())?
        .filter;
    Ok(Day { date, filter })
}

/// A value of the screen with the fields `values` shows (format.md §17).
struct Shown {
    name: String,
    value: String,
    unit: Option<String>,
    source: Option<String>,
    valid_from: Option<String>,
    key: bool,
    proposed: Option<String>,
}

impl Shown {
    /// `<name>  <value>[ <unit>]  (<source>, <date>)`, then `  [proposed: <text>]` when a change
    /// waits: the line the `values` command writes. A missing source or date is left out.
    fn line(&self) -> String {
        let number = match &self.unit {
            Some(unit) => format!("{} {unit}", self.value),
            None => self.value.clone(),
        };
        let from = self.valid_from.as_deref().map(time::date);
        let facts: Vec<&str> = [self.source.as_deref(), from]
            .into_iter()
            .flatten()
            .filter(|text| !text.is_empty())
            .collect();
        let mut line = format!("{}  {number}", self.name);
        if !facts.is_empty() {
            line.push_str(&format!("  ({})", facts.join(", ")));
        }
        if let Some(proposed) = &self.proposed {
            line.push_str(&format!("  [proposed: {proposed}]"));
        }
        line
    }

    /// The value as data, with the full time it came into force.
    fn data(&self) -> Value {
        json!({
            "name": self.name,
            "value": self.value,
            "unit": self.unit,
            "source": self.source,
            "valid_from": self.valid_from,
            "key": self.key,
            "proposed": self.proposed,
        })
    }
}

/// The key values of the screen: the values in force (on the day when there is one), the key ones
/// first and then by name, at most KEY_VALUES. A values file that cannot be read is a warning, and
/// no value is shown.
fn key_values_part(ctx: &Ctx, day: Option<&Day>, warnings: &mut Vec<String>) -> Vec<Shown> {
    let file = match values::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => file,
        Err(error) => {
            warnings.push(error.message);
            return Vec::new();
        }
    };
    let on = day.map(|day| day.date.as_str());
    let mut shown: Vec<Shown> = values::all(&file)
        .iter()
        .filter_map(|value| match on {
            Some(date) => shown_on(&file, value, date),
            None => shown_now(value),
        })
        .collect();
    shown.sort_by(|a, b| b.key.cmp(&a.key).then_with(|| a.name.cmp(&b.name)));
    shown.truncate(KEY_VALUES);
    shown
}

/// A value in force now. A block with only a proposal is not in force and is left out.
fn shown_now(value: &values::Value) -> Option<Shown> {
    (!value.value.is_empty()).then(|| Shown {
        name: value.name.clone(),
        value: value.value.clone(),
        unit: value.unit.clone(),
        source: value.source.clone(),
        valid_from: value.valid_from.clone(),
        key: value.key,
        proposed: value.proposed.clone(),
    })
}

/// A value in force at the end of `day` (YYYY-MM-DD), as `values --on` shows it: the value and unit
/// `values::on` gives, the time of the `set` line in force, and the source only when no later `set`
/// or `changed` line could have changed it (format.md §17). A past day has no record of a
/// proposal, so none is shown.
fn shown_on(file: &BlockFile, value: &values::Value, day: &str) -> Option<Shown> {
    let text = values::on(file, &value.name, day)?;
    let (number, unit) = values::split(&text, value.unit.is_none()).unwrap_or((text.clone(), None));
    let history = history_of(file, &value.name);
    let valid_from = history
        .iter()
        .rfind(|(stamp, event)| event == "set" && time::date(stamp) <= day)
        .map(|(stamp, _)| stamp.clone());
    let source_kept = history.iter().all(|(stamp, event)| {
        !matches!(event.as_str(), "set" | "changed") || time::date(stamp) <= day
    });
    Some(Shown {
        name: value.name.clone(),
        value: number,
        unit,
        source: value.source.clone().filter(|_| source_kept),
        valid_from,
        key: value.key,
        proposed: None,
    })
}

/// The time and the event of each history line of a block (format.md §6): the line is
/// `<time> <who>[ via <llm>]: <event>[ — <detail>]`. Lines without those parts are left out.
fn history_of(file: &BlockFile, key: &str) -> Vec<(String, String)> {
    file.history(key)
        .iter()
        .filter_map(|line| {
            let (stamp, rest) = line.split_once(' ')?;
            let (_who, rest) = rest.split_once(": ")?;
            let event = rest.split(' ').next()?;
            Some((stamp.to_string(), event.to_string()))
        })
        .collect()
}

/// A record of the exchange as the screen shows it: its key, its date, and who it was from or to.
struct Exchanged {
    id: String,
    date: String,
    who: String,
}

impl Exchanged {
    /// The record as data.
    fn data(&self) -> Value {
        json!({"id": self.id, "date": self.date, "who": self.who})
    }
}

/// The last received and the last delivered record (exchange.md), on or before the day when there
/// is one. A file that cannot be read is a warning, and neither record is shown.
fn last_exchange(
    ctx: &Ctx,
    day: Option<&Day>,
    warnings: &mut Vec<String>,
) -> (Option<Exchanged>, Option<Exchanged>) {
    let file = match exchange::load(&ctx.dossier, &ctx.cfg.code) {
        Ok((file, _)) => file,
        Err(error) => {
            warnings.push(error.message);
            return (None, None);
        }
    };
    let on = day.map(|day| day.date.as_str());
    (
        last_record(&file, "RCV", "from", on),
        last_record(&file, "DLV", "to", on),
    )
}

/// The newest record with the prefix RCV or DLV, by date and then by number. `who_field` is `from`
/// or `to`. With a day, a record dated after it, or with no date, is left out.
fn last_record(
    file: &BlockFile,
    prefix: &str,
    who_field: &str,
    on: Option<&str>,
) -> Option<Exchanged> {
    file.keys()
        .into_iter()
        .filter_map(|key| {
            let number = number_after(&key, prefix)?;
            let date = file.field(&key, "date").unwrap_or_default();
            if on.is_some_and(|day| date.is_empty() || date.as_str() > day) {
                return None;
            }
            let who = file.field(&key, who_field).unwrap_or_default();
            Some((number, Exchanged { id: key, date, who }))
        })
        .max_by(|(a_number, a), (b_number, b)| a.date.cmp(&b.date).then(a_number.cmp(b_number)))
        .map(|(_, record)| record)
}

/// The number of a key `<prefix>-<digits>`: 2 for RCV-002 and the prefix RCV. None for any other
/// key.
fn number_after(key: &str, prefix: &str) -> Option<u64> {
    let digits = key.strip_prefix(prefix)?.strip_prefix('-')?;
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| digits.parse().unwrap_or(u64::MAX))
}

/// The lines of the Exchange part: the last received and the last delivered record, or that there
/// is none of that kind.
fn exchange_lines(received: Option<&Exchanged>, delivered: Option<&Exchanged>) -> Vec<String> {
    vec![
        match received {
            Some(record) => format!("received {} {} from {}", record.date, record.id, record.who),
            None => "nothing received".to_string(),
        },
        match delivered {
            Some(record) => format!("delivered {} {} to {}", record.date, record.id, record.who),
            None => "nothing delivered".to_string(),
        },
    ]
}

/// The entries of `scope` that rest on a wrong assumption (format.md §14), each with its
/// assumptions: the `rests-on` field names one, and the entry is in use (see `in_use`). Drafts and
/// proposals count, as the Unsettled part counts them.
fn rests_on_entries<'a>(scope: &[&'a Item]) -> Vec<(&'a Item, Vec<String>)> {
    scope
        .iter()
        .copied()
        .filter(|item| in_use(item.entry.status()))
        .map(|item| (item, item.entry.list("rests-on")))
        .filter(|(_, assumptions)| !assumptions.is_empty())
        .collect()
}

/// Whether an entry with this status is in use, so that a wrong assumption it rests on counts
/// (R-CTX-16): every status but rejected, declined, superseded and retired.
fn in_use(status: Option<Status>) -> bool {
    !matches!(
        status,
        Some(Status::Rejected | Status::Declined | Status::Superseded | Status::Retired)
    )
}

/// The blocks of the file lists that rest on a wrong assumption and are not detached (format.md
/// §17): the task, the stored path and the assumptions. With a milestone only its task's list is
/// read. A list that cannot be read is a warning.
fn rests_on_files(
    ctx: &Ctx,
    milestone: Option<&str>,
    warnings: &mut Vec<String>,
) -> Vec<(String, String, Vec<String>)> {
    let mut found = Vec::new();
    for task in filelist::tasks(&ctx.dossier)
        .into_iter()
        .filter(|task| milestone.is_none_or(|wanted| task.eq_ignore_ascii_case(wanted)))
    {
        let (list, _) = match filelist::load(&ctx.dossier, &task) {
            Ok(loaded) => loaded,
            Err(error) => {
                let path = filelist::path(&ctx.dossier, &task);
                warnings.push(format!(
                    "{}: {}",
                    shown_path(&ctx.root, &path),
                    error.message
                ));
                continue;
            }
        };
        for key in list.keys() {
            let detached = list
                .field(&key, "detached")
                .is_some_and(|at| !at.is_empty());
            let assumptions = comma_list(&list.field(&key, "rests-on").unwrap_or_default());
            if !detached && !assumptions.is_empty() {
                found.push((task.clone(), key, assumptions));
            }
        }
    }
    found
}

/// A comma-separated list as its parts, trimmed, without the empty parts.
fn comma_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// What rests on a wrong assumption as the screen lists it (R-CTX-16): each entry of `entries` (see
/// `rests_on_entries`) as `<ID> · <title>`, its status mark and `[rests on <IDs>]`, then each file
/// block of `files` (task, path, assumptions) as `<task> <path> [rests on <IDs>]`. Each line comes
/// with the ID it is listed by.
fn resting_lines(
    entries: &[(&Item, Vec<String>)],
    files: &[(String, String, Vec<String>)],
) -> Vec<(String, String)> {
    let entries = entries.iter().map(|(item, assumptions)| {
        (
            item.entry.id(),
            format!(
                "{}{}{}{}",
                heading_of(item),
                status_mark(item),
                due_mark(&item.entry),
                rests_mark(assumptions)
            ),
        )
    });
    let blocks = files.iter().map(|(task, path, assumptions)| {
        let id = format!("{task} {path}");
        (id.clone(), format!("{id}{}", rests_mark(assumptions)))
    });
    entries.chain(blocks).collect()
}

/// The status mark of an entry in the list of what rests on a wrong assumption: `[draft]` or
/// `[proposed]` with its leading space, and nothing for an approved entry.
fn status_mark(item: &Item) -> &'static str {
    match item.entry.status() {
        Some(Status::Draft) => " [draft]",
        Some(Status::Proposed) => " [proposed]",
        _ => "",
    }
}

/// The mark ` [rests on <IDs>]` of IDs, joined with `, `; nothing when there are none.
fn rests_mark(assumptions: &[String]) -> String {
    if assumptions.is_empty() {
        String::new()
    } else {
        format!(" [rests on {}]", assumptions.join(", "))
    }
}

/// The entries and blocks that rest on a wrong assumption, as data: `kind` is `entry` with its
/// `id`, or `file` with its `task` and `path`; each with the assumptions it rests on.
fn rests_on_json(
    entries: &[(&Item, Vec<String>)],
    files: &[(String, String, Vec<String>)],
) -> Value {
    let mut all: Vec<Value> = entries
        .iter()
        .map(|(item, assumptions)| {
            json!({"kind": "entry", "id": item.entry.id(), "rests_on": assumptions})
        })
        .collect();
    all.extend(files.iter().map(|(task, path, assumptions)| {
        json!({"kind": "file", "task": task, "path": path, "rests_on": assumptions})
    }));
    Value::Array(all)
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

/// The screen with `--since`: the journal records of entries (`is_entry_record`) that `bound`
/// keeps, newest first, with the title of each entry. With `last`, the person's marker is read,
/// and written when a person runs the command without `keep`; an LLM's run never writes it. With a
/// team dossier the team's news follows the project's lines (`team_news`).
fn since_screen(
    ctx: &Ctx,
    since: Since,
    all: &[Item],
    milestone: Option<&str>,
    keep: bool,
) -> Result<Screen> {
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
            if !who.is_llm && !keep {
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
        .filter(is_entry_record)
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
    let team = team_news(ctx, &bound, milestone, &mut warnings);
    let team_events = match &team {
        Some(news) => serde_json::to_value(&news.records)
            .map_err(|error| Error::new(Code::Io, error.to_string()))?,
        None => Value::Null,
    };
    let mut sections = vec![Section::plain(vec![project_line(ctx)])];
    match team {
        Some(news) => {
            sections.push(Section::headed(&heading, Vec::new()));
            sections.push(Section::list(
                &format!("## Project {}", ctx.cfg.code),
                lines,
                "none",
                "dossier log",
            ));
            sections.push(news.section);
        }
        None => sections.push(Section::list(&heading, lines, "none", "dossier log")),
    }
    Ok(Screen {
        sections,
        data: json!({"since": bound.value(), "events": events_json, "team_events": team_events}),
        warnings,
        marker,
    })
}

/// Whether a journal record is about an entry (format.md §11): its ID is a kind's prefix, a dash
/// and a number. The settings IDs (DOSSIER, SETTINGS, AGENTS, HOOKS, SKILL) and the IDs of the
/// values, the map, the exchange and the files are not entries, so the project's lines of
/// `--since` leave their records out.
fn is_entry_record(record: &Record) -> bool {
    let id = record.id.to_uppercase();
    Kind::all()
        .iter()
        .any(|kind| number_after(&id, kind.prefix()).is_some())
}

/// The Team part of `--since` (R-READ-10): its section, and the records it lists, for the data.
struct TeamNews {
    section: Section,
    records: Vec<Record>,
}

/// Whether a team journal record is news (R-READ-10): an entry approved, superseded, retired or
/// accepted, or created approved. Drafts, rejections and changes to fields are not news.
fn is_team_news(record: &Record) -> bool {
    match record.event.as_str() {
        "approved" | "superseded" | "retired" | "accepted" => true,
        "created" => record.detail == "approved",
        _ => false,
    }
}

/// The Team part of `--since` (R-READ-10): the team dossier's journal records that `bound` keeps
/// and that are news (`is_team_news`), newest first, each with the title of its team entry and the
/// milestone rule of the project's lines. At most TEAM_LINES lines; the last one counts the news
/// left out. None when the project has no team dossier. A team dossier that cannot be reached, or
/// whose journal cannot be read, is a warning and its part is left out.
fn team_news(
    ctx: &Ctx,
    bound: &Bound,
    milestone: Option<&str>,
    warnings: &mut Vec<String>,
) -> Option<TeamNews> {
    let stored = ctx.cfg.team.as_deref()?;
    if ctx.team.is_none() {
        warnings.push(format!(
            "the team dossier {stored} cannot be reached; its news is left out"
        ));
        return None;
    }
    let team = match ctx.team_ctx() {
        Ok(team) => team,
        Err(error) => {
            warnings.push(error.message);
            return None;
        }
    };
    let (items, item_warnings) = store::all(&team.dossier);
    warnings.extend(item_warnings);
    let records = match journal::read(&team.state) {
        Ok(records) => records,
        Err(error) => {
            warnings.push(error.message);
            return None;
        }
    };
    let index: HashMap<String, &Item> = items.iter().map(|item| (item.entry.id(), item)).collect();
    let mut news: Vec<Record> = records
        .into_iter()
        .filter(|record| is_team_news(record) && bound.keeps(&record.time))
        .filter(|record| {
            index
                .get(&record.id.to_uppercase())
                .map_or(milestone.is_none(), |item| in_scope(item, milestone))
        })
        .collect();
    news.reverse();

    let mut lines: Vec<String> = news
        .iter()
        .map(|record| event_line(record, &index))
        .collect();
    let mut hidden = 0;
    if lines.len() > TEAM_LINES {
        hidden = lines.len() - (TEAM_LINES - 1);
        lines.truncate(TEAM_LINES - 1);
    }
    let mut section = Section::list("## Team", lines, "none", "dossier log --team");
    section.hidden = hidden;
    Some(TeamNews {
        section,
        records: news,
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
            first_cut: false,
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
    fn only_records_of_entry_ids_are_entry_records() {
        let record = |id: &str| Record {
            time: "2026-11-21T10:00:00+01:00".to_string(),
            name: "anna".to_string(),
            person: "anna".to_string(),
            llm: false,
            command: "status".to_string(),
            id: id.to_string(),
            event: "created".to_string(),
            detail: String::new(),
        };
        for id in [
            "DEC-012", "asm-001", "FND-1000", "LES-003", "USE-001", "HOW-002", "TOOL-004",
        ] {
            assert!(is_entry_record(&record(id)), "{id}");
        }
        for id in [
            "DOSSIER",
            "SETTINGS",
            "AGENTS",
            "HOOKS",
            "SKILL",
            "VALUES",
            "MAP",
            "EXCHANGE",
            "FILES-T-042",
            "RCV-001",
            "NT-999",
            "DEC",
            "DEC-",
            "DECX-1",
        ] {
            assert!(!is_entry_record(&record(id)), "{id}");
        }
    }

    #[test]
    fn shown_path_is_relative_to_the_project_with_slashes() {
        let root = Path::new("projects").join("Projekt Äpfel");
        let path = root.join("DOSSIER").join(".state").join("seen-anna.txt");
        assert_eq!(shown_path(&root, &path), "DOSSIER/.state/seen-anna.txt");
    }
}
