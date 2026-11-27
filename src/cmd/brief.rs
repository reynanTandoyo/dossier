//! The `brief` command: what an LLM should know at the start of a session in this project
//! (R-CTX-1), or with `--task` what matters for one task (R-CTX-2, see `brief_task`). Only
//! approved entries are printed (R-CTX-4). The output stays within `brief_lines` lines, or within
//! the `--budget` lines of one call (R-CTX-6), and is the same on every run. Nothing is changed.
//!
//! The project brief also prints the map of the code (R-CODE-1), the key values (R-VALUE-4), the
//! decisions proposed and waiting for a decider (R-CTX-16), and what rests on a wrong assumption
//! (R-CTX-16): the entries and file blocks marked `rests on <ID>`, under their own heading. With
//! `--on <date>` it prints what was in force on that day instead (R-LIFE-9); the proposals, the
//! drafts and what rests on a wrong assumption are then left out.
//!
//! In a worker session (DOSSIER_ROLE=worker, R-AGENT-6) the project brief is replaced by one line,
//! unless `--full` is given; `--task` and `--on` print as before.
//!
//! When the project names a team dossier (R-CTX-1), the team's approved lessons and how-tos whose
//! machine, analysis or software tags match the project's are printed under their own heading, at
//! most MAX_TEAM of them. The last line says how many entries were approved in the project or the
//! team since the person's marker, when there are any (R-READ-11): the marker is the one
//! `status --since last` writes, so a person who has never run it gets no news line. With `--on`
//! the team lines are those in force on that day, and the news line is left out.
//!
//! Every entry printed carries `[due for review]` when its review date has passed (R-LIFE-7,
//! R-CTX-4), and a lesson, how-to or tool changed by hand after its approval carries
//! `[changed after approval]` (R-LIFE-6): the same marks, in the same order, as `show` gives.
//!
//! The open investigations (R-INV-5) come after the open assumptions, one line each with its number
//! of attempts and its last attempt. With `--on` their state and attempts are those of now.
//!
//! The approved digests in force (R-CTX-15) come after the key values, under `## Digests`: each
//! one's heading and its Summary, indented. The decisions and findings that a digest sums up are
//! left out and counted after the digests, one line per digest that sums up any:
//! `<n> entries of <milestone> are summed up in <DIG-ID>: dossier show <DIG-ID>` (design.md section
//! 18). Open assumptions and decisions that stand (`standing: yes`) are never left out, and a digest
//! line is never cut to fit the limit.

use crate::approval::{self, Mark};
use crate::attempts;
use crate::blockfile::BlockFile;
use crate::brief_task;
use crate::cli::Global;
use crate::config::Tags;
use crate::ctx::Ctx;
use crate::digest;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::filter::{in_force_on, is_whole_date, newest_first};
use crate::journal::{self, Record};
use crate::mapfile;
use crate::output::Output;
use crate::role::{self, Role};
use crate::store::{self, Item};
use crate::time;
use crate::values;
use chrono::{DateTime, NaiveDate};
use serde_json::{Value, json};
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
    /// The most lines this call prints, at least 5, instead of `brief_lines` (R-CTX-6)
    #[arg(long, value_name = "lines")]
    pub budget: Option<usize>,
    /// Prints the project brief also in a worker session (R-AGENT-6)
    #[arg(long, conflicts_with = "task")]
    pub full: bool,
}

/// The headings of the project brief, in the order they are printed.
const MAP: &str = "## Map";
const KEY_VALUES: &str = "## Key values";
const DIGESTS: &str = "## Digests";
const DECISIONS: &str = "## Decisions in force";
const ASSUMPTIONS: &str = "## Open assumptions";
const INVESTIGATIONS: &str = "## Open investigations";
const FINDINGS: &str = "## Findings";
const LESSONS: &str = "## Lessons";
const TEAM: &str = "## Team lessons and how-tos";
const PROPOSALS: &str = "## Proposals waiting";
const RESTS: &str = "## Rests on a wrong assumption";

/// The most lines of the map the brief prints (R-CODE-1), and the most values (R-VALUE-4).
const MAX_MAP_LINES: usize = 25;
const MAX_VALUES: usize = 15;

/// The most team lessons and how-tos the brief names (R-CTX-1).
const MAX_TEAM: usize = 10;

/// The field that names the wrong assumptions an entry rests on (format.md §14), and the field
/// that gives the time an assumption was found wrong.
const RESTS_ON: &str = "rests-on";
const REFUTED: &str = "refuted";

/// The field that names the topics of an entry (R-TOPIC-1).
const TOPIC: &str = "topic";

/// The variable that is set when the command was started through ssh (R-REMOTE-12).
const SSH_VARIABLE: &str = "SSH_CONNECTION";

/// The fewest lines `--budget` accepts (R-CTX-6).
const MIN_BUDGET: usize = 5;

/// The one line a worker session gets instead of the project brief (R-AGENT-6).
const WORKER_LINE: &str = "dossier: worker session. The entries for your task are printed after todo claim; the project brief is left out.";

/// Runs `brief`: the context of the project, or with `--task` the context of one task. The line
/// limit is `--budget` when it is given, else `brief_lines`. The warnings of reading the dossier
/// are passed on.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if let Some(lines) = args.budget
        && lines < MIN_BUDGET
    {
        return Err(Error::new(
            Code::Usage,
            format!("--budget needs at least {MIN_BUDGET} lines, not {lines}"),
        ));
    }
    let ctx = Ctx::load(g, "brief")?;
    if args.task.is_none() && args.on.is_none() && !args.full && role::current() == Role::Worker {
        let data = json!({"code": ctx.cfg.code, "role": "worker", "lines": [WORKER_LINE]});
        return Ok(Output::with(WORKER_LINE, data));
    }
    let limit = args.budget.unwrap_or(ctx.cfg.brief_lines);
    let remote = std::env::var(SSH_VARIABLE).is_ok_and(|value| !value.is_empty());
    match &args.task {
        Some(task) => for_task(&ctx, task, remote, limit),
        None => {
            let day = args.on.as_deref().map(day_option).transpose()?;
            for_project(&ctx, remote, day.as_deref(), limit)
        }
    }
}

/// `brief --task`: the lines of `brief_task`, after the server line when the command came
/// through ssh. The server line counts in `limit`. The data also gives `items`: each entry whose
/// summary line is printed, with its reason (R-CTX-6). Both come from one selection, so they agree,
/// and a line the limit cuts takes its entry out of `items` too.
fn for_task(ctx: &Ctx, task: &str, remote: bool, limit: usize) -> Result<Output> {
    let mut lines = Vec::new();
    if remote {
        lines.push(server_line(ctx));
    }
    let room = limit.saturating_sub(lines.len());
    let selection = brief_task::select(ctx, task, room)?;
    lines.extend(selection.lines);
    let items: Vec<Value> = selection
        .chosen
        .into_iter()
        .map(|(id, reason)| json!({"id": id, "reason": reason}))
        .collect();
    let data = json!({"code": ctx.cfg.code, "task": task, "lines": lines, "items": items});
    Ok(Output::with(lines.join("\n"), data))
}

/// `brief` for the project, fitted to `limit` lines (`brief_lines`, or `--budget`), for the day
/// `day` when it is given. The warnings of reading the dossier are passed on with their paths shown
/// relative to the project folder. A map or values file that cannot be read gives a warning and is
/// left out.
fn for_project(ctx: &Ctx, remote: bool, day: Option<&str>, limit: usize) -> Result<Output> {
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
            let (lines, found) = rests_lines(&ctx.dossier, &ctx.state, &items);
            problems.extend(found);
            lines
        }
    };
    let team = team_context(ctx, &mut problems);
    let (team_lines, team_left_out, team_problems) = match &team {
        Some(team) => team_part(ctx, team, day),
        None => (Vec::new(), 0, Vec::new()),
    };
    problems.extend(team_problems);
    let news = match day {
        Some(_) => None,
        None => news_of(ctx, team.as_ref(), &mut problems),
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
    brief.team = team_lines;
    brief.team_left_out = team_left_out;
    brief.news = news;
    // The news line is the last line, so it takes one of the lines the limit allows.
    let news_line = brief.news.as_ref().and_then(News::line);
    let mut lines = brief.fit(if news_line.is_some() {
        limit.saturating_sub(1)
    } else {
        limit
    });
    if let Some(line) = news_line
        && limit > 0
    {
        lines.push(line);
    }
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
    /// The approved digests in force, newest first (R-CTX-15). Their lines are never cut.
    digests: Vec<Digest>,
    /// The digests in force that sum up at least one decision or finding, in the order of
    /// `digests` (R-CTX-15). Each is named by its line after the digests; the entries are not
    /// printed singly.
    summed: Vec<Summed>,
    /// The decisions in force (approved, or with `--on` in force on the day), newest first, without
    /// those a digest sums up.
    decisions: Vec<Decision>,
    /// The topics of the settings, in their order, each with the decisions in force that name it
    /// (R-TOPIC-5). Empty when the settings name no topic.
    topics: Vec<TopicCount>,
    /// The assumptions that are open, by ID.
    assumptions: Vec<Line>,
    /// The open investigations, newest first, by ID (R-INV-5).
    investigations: Vec<Line>,
    /// The findings, by ID, without those a digest sums up.
    findings: Vec<Line>,
    /// The lessons, by ID.
    lessons: Vec<Line>,
    /// The team's approved lessons and how-tos whose tags match the project's, by ID, at most
    /// MAX_TEAM (R-CTX-1). Set from the team dossier after `collect`.
    team: Vec<Line>,
    /// The decisions proposed and waiting for a decider, by ID. Left out with `--on`.
    proposals: Vec<Line>,
    /// What rests on a wrong assumption (R-CTX-16): the approved entries, then the file blocks.
    /// Left out with `--on`.
    rests: Vec<Line>,
    /// The number of drafts waiting for approval. Left out with `--on`.
    drafts: usize,
    /// The news of the project and the team since the person's marker (R-READ-11). Set after
    /// `collect`; None with `--on`, and when no person is named or the person has no marker yet.
    news: Option<News>,
    /// The number of entries cut to fit the limit (lessons, findings, investigations, decisions,
    /// assumptions).
    left_out: usize,
    /// The number of team lines left out: those beyond MAX_TEAM, or cut to fit the limit.
    team_left_out: usize,
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
        let digests = digest::in_force(items, day);
        // The decisions in force, those a digest sums up included: the topics count them all.
        let in_force = shown(items, Kind::Decision, day);
        let mut decisions = in_force.clone();
        decisions.sort_by(|a, b| newest_first(&a.entry, &b.entry));
        let (decisions, summed_decisions) = split_covered(decisions, &digests);
        let (findings, summed_findings) = split_covered(shown(items, Kind::Finding, day), &digests);
        let summed: Vec<&Item> = summed_decisions
            .into_iter()
            .chain(summed_findings)
            .collect();
        Brief {
            head: head(ctx, remote, &in_force),
            topics: topic_counts(&ctx.cfg.topics, &in_force),
            map: map_section,
            values: value_section,
            digests: digests
                .iter()
                .map(|digest| digest_of(digest, &ctx.state))
                .collect(),
            summed: summed_per_digest(&digests, &summed),
            decisions: decisions
                .into_iter()
                .map(|item| decision_of(item, &ctx.state, &wrong))
                .collect(),
            assumptions: shown(items, Kind::Assumption, day)
                .into_iter()
                .filter(|item| is_open(&item.entry, day))
                .map(|item| line(&item.entry, &review_marks(&ctx.state, item), &wrong))
                .collect(),
            investigations: investigations_of(items, &ctx.state, day, &wrong),
            findings: findings
                .into_iter()
                .map(|item| {
                    let dead_end = if is_dead_end(&item.entry) {
                        " (dead end)"
                    } else {
                        ""
                    };
                    let suffix = format!("{dead_end}{}", review_marks(&ctx.state, item));
                    line(&item.entry, &suffix, &wrong)
                })
                .collect(),
            lessons: shown(items, Kind::Lesson, day)
                .into_iter()
                .map(|item| line(&item.entry, &review_marks(&ctx.state, item), &wrong))
                .collect(),
            proposals: if day.is_some() {
                Vec::new()
            } else {
                proposals_of(items, &ctx.state, &wrong)
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
            team: Vec::new(),
            news: None,
            left_out: 0,
            team_left_out: 0,
            proposals_left_out: 0,
            rests_left_out: 0,
            values_left_out: 0,
            map_left_out: false,
            day: day.map(str::to_string),
        }
    }

    /// The lines of the digests in force: each digest's heading and summary, then one line per
    /// digest that sums up entries, saying how many and where to see them.
    fn digest_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.digests.iter().flat_map(Digest::lines).collect();
        lines.extend(self.summed.iter().map(summed_text));
        lines
    }

    /// The lines as they would be printed now: the head, then each heading with its lines (a
    /// heading with no lines is left out), the drafts line and the left-out lines.
    fn lines(&self) -> Vec<String> {
        let day = self.day.as_deref();
        let mut lines = self.head.clone();
        push_section(&mut lines, MAP, &self.map);
        push_section(&mut lines, KEY_VALUES, &texts(&self.values));
        push_section(&mut lines, DIGESTS, &self.digest_lines());
        let decisions: Vec<String> = self.decisions.iter().flat_map(Decision::lines).collect();
        push_section(&mut lines, DECISIONS, &decisions);
        push_section(&mut lines, ASSUMPTIONS, &texts(&self.assumptions));
        push_section(&mut lines, INVESTIGATIONS, &texts(&self.investigations));
        push_section(&mut lines, FINDINGS, &texts(&self.findings));
        push_section(&mut lines, LESSONS, &texts(&self.lessons));
        push_section(&mut lines, TEAM, &texts(&self.team));
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
        if self.team_left_out > 0 {
            lines.push(left_out_text(
                self.team_left_out,
                "team entry",
                "team entries",
                "dossier list --team",
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
    /// team line, the last open investigation, the last proposal, the last line that rests on a
    /// wrong assumption, the last value, the whole map, the oldest decision, the last open
    /// assumption. Returns false when nothing is left to cut. Each step stops at the first list
    /// that had something, so only one thing is cut per call. The digests in force and the lines
    /// about them are not on the list: they are never cut (R-CTX-15).
    fn cut_one(&mut self) -> bool {
        if self.lessons.pop().is_some() || self.findings.pop().is_some() {
            self.left_out += 1;
        } else if self.team.pop().is_some() {
            self.team_left_out += 1;
        } else if self.investigations.pop().is_some() {
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
    /// printed, the digests in force, the number of entries they sum up, the team's IDs, the news,
    /// and the number of drafts. A file block that rests on a wrong assumption is named by its task
    /// and path, as `<task> <path>`.
    fn data(&self, code: &str) -> Value {
        json!({
            "code": code,
            "on": self.day,
            "map": self.map,
            "values": ids(&self.values),
            "digests": self.digests.iter().map(|digest| digest.id.as_str()).collect::<Vec<_>>(),
            "summed_up": self.summed.iter().map(|summed| summed.count).sum::<usize>(),
            "decisions": self.decisions.iter().map(|decision| decision.id.as_str()).collect::<Vec<_>>(),
            "topics": self.topics.iter().map(|topic| json!({"name": topic.name, "decisions": topic.decisions})).collect::<Vec<_>>(),
            "assumptions": ids(&self.assumptions),
            "investigations": ids(&self.investigations),
            "findings": ids(&self.findings),
            "lessons": ids(&self.lessons),
            "team": ids(&self.team),
            "proposals": ids(&self.proposals),
            "rests": ids(&self.rests),
            "drafts": self.drafts,
            "news": self.news.as_ref().map(News::data),
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

/// One digest in force and the number of decisions and findings it sums up that the brief left out
/// (R-CTX-15). Its line after the digests names the milestone and the digest.
struct Summed {
    /// The ID of the digest.
    digest: String,
    /// The milestone the digest names, as its heading shows it.
    milestone: String,
    /// The number of entries it covers.
    count: usize,
}

/// One topic of the settings and the number of decisions in force that name it (R-TOPIC-5).
struct TopicCount {
    name: String,
    decisions: usize,
}

/// One digest in force (R-CTX-15) as printed: its heading `<DIG-ID> · <milestone> · <title>` with
/// its review marks, then its Summary lines, indented by two spaces.
struct Digest {
    id: String,
    heading: String,
    summary: Vec<String>,
}

impl Digest {
    /// The lines the digest takes: the heading, then the summary.
    fn lines(&self) -> Vec<String> {
        let mut lines = vec![self.heading.clone()];
        lines.extend(self.summary.iter().cloned());
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
            Some(day) => in_force_on(&item.entry, day),
            None => item.entry.status() == Some(Status::Approved),
        })
        .collect()
}

/// Splits the entries into those no digest covers and those a digest sums up (R-CTX-15), each in the
/// order given. `digests` are the digests in force, as `digest::in_force` gives them.
fn split_covered<'a>(items: Vec<&'a Item>, digests: &[&Item]) -> (Vec<&'a Item>, Vec<&'a Item>) {
    items
        .into_iter()
        .partition(|item| digest::covering(digests, &item.entry).is_none())
}

/// The digest as printed: `<ID> · <milestone> · <title>` with its review marks, and its Summary
/// indented by two spaces. A digest with no milestone shows `-` for it, and one with no title
/// leaves the title out.
fn digest_of(item: &Item, state: &Path) -> Digest {
    let entry = &item.entry;
    let mut parts = vec![entry.id(), milestone_of(entry)];
    let title = entry.title();
    if !title.is_empty() {
        parts.push(title);
    }
    Digest {
        id: entry.id(),
        heading: format!("{}{}", parts.join(" · "), review_marks(state, item)),
        summary: entry
            .section("Summary")
            .map_or_else(Vec::new, |summary| indented(&summary)),
    }
}

/// The milestone of a digest as its lines show it: its `milestone` field, or `-` when it has none.
fn milestone_of(entry: &Entry) -> String {
    entry
        .field("milestone")
        .filter(|milestone| !milestone.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

/// The digests in force that sum up at least one of `summed` (the decisions and findings the
/// digests cover, as `split_covered` gives them), in the order of `digests`, each with the number of
/// those it covers. An entry counts for the digest that covers it (`digest::covering`).
fn summed_per_digest(digests: &[&Item], summed: &[&Item]) -> Vec<Summed> {
    digests
        .iter()
        .filter_map(|digest| {
            let id = digest.entry.id();
            let count = summed
                .iter()
                .filter(|item| {
                    digest::covering(digests, &item.entry)
                        .is_some_and(|covering| covering.entry.id() == id)
                })
                .count();
            (count > 0).then(|| Summed {
                digest: id,
                milestone: milestone_of(&digest.entry),
                count,
            })
        })
        .collect()
}

/// The decisions waiting for a decider (status `proposed`), in the order of `store::all`, each as
/// `<ID> · <title> (<author>)`, then its review marks. `state` is the state folder of the dossier.
fn proposals_of(items: &[Item], state: &Path, wrong: &Wrong) -> Vec<Line> {
    items
        .iter()
        .filter(|item| {
            item.entry.kind() == Some(Kind::Decision)
                && item.entry.status() == Some(Status::Proposed)
        })
        .map(|item| {
            let suffix = format!(" ({}){}", author(&item.entry), review_marks(state, item));
            line(&item.entry, &suffix, wrong)
        })
        .collect()
}

/// The open investigations, newest first, each as `<ID> · <title>[ marks] — <n> attempts, last:
/// <summary of its last attempt>`, or `— no attempt yet` (R-INV-5). `state` is the state folder of
/// the dossier and `day` is as in `shown`.
fn investigations_of(items: &[Item], state: &Path, day: Option<&str>, wrong: &Wrong) -> Vec<Line> {
    let mut open: Vec<&Item> = shown(items, Kind::Investigation, day)
        .into_iter()
        .filter(|item| is_open_investigation(&item.entry))
        .collect();
    open.sort_by(|a, b| newest_first(&a.entry, &b.entry));
    open.into_iter()
        .map(|item| {
            let suffix = format!(
                "{}{}",
                review_marks(state, item),
                attempts_text(&item.entry)
            );
            line(&item.entry, &suffix, wrong)
        })
        .collect()
}

/// Whether an investigation is open now: its `state` is `open`.
fn is_open_investigation(entry: &Entry) -> bool {
    entry
        .field("state")
        .is_some_and(|state| state.eq_ignore_ascii_case("open"))
}

/// The attempts part of an investigation's line: ` — 2 attempts, last: 2. failed: <text>`, or
/// ` — no attempt yet` when it has none.
fn attempts_text(entry: &Entry) -> String {
    let attempts = attempts::of(entry);
    match attempts.last() {
        None => " — no attempt yet".to_string(),
        Some(last) => {
            let count = attempts.len();
            let noun = if count == 1 { "attempt" } else { "attempts" };
            format!(" — {count} {noun}, last: {}", attempts::summary(last))
        }
    }
}

/// What rests on a wrong assumption, as the brief of now prints it (R-CTX-16): the approved entries
/// whose `rests-on` field names an ID, in the order of `store::all`, then the blocks of the file
/// lists that are not detached and whose `rests-on` field names an ID, the tasks in order and the
/// blocks in the order of their list. Each is one line. A list that cannot be read gives its
/// problem and no lines.
fn rests_lines(dossier: &Path, state: &Path, items: &[Item]) -> (Vec<Line>, Vec<String>) {
    let mut lines: Vec<Line> = items
        .iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter_map(|item| {
            let assumptions = item.entry.list(RESTS_ON);
            (!assumptions.is_empty()).then(|| Line {
                id: item.entry.id(),
                text: format!(
                    "{}{}{}",
                    entry_text(&item.entry),
                    review_marks(state, item),
                    rests_mark(&assumptions)
                ),
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

/// The first lines of the brief: the project line (the server line in remote mode), then the
/// tags line when there are tags, then the topics line when the settings name topics. `decisions`
/// are the decisions in force, as `shown` gives them.
fn head(ctx: &Ctx, remote: bool, decisions: &[&Item]) -> Vec<String> {
    let project = if remote {
        server_line(ctx)
    } else {
        format!("Project {}", ctx.cfg.code)
    };
    let mut head = vec![project];
    head.extend(tags_line(&ctx.cfg.tags));
    head.extend(topics_line(&topic_counts(&ctx.cfg.topics, decisions)));
    head
}

/// The topics of the settings, in their order, each with the decisions that name it (R-TOPIC-5).
fn topic_counts(topics: &[String], decisions: &[&Item]) -> Vec<TopicCount> {
    topics
        .iter()
        .map(|topic| TopicCount {
            name: topic.clone(),
            decisions: decisions
                .iter()
                .filter(|item| names(&item.entry, topic))
                .count(),
        })
        .collect()
}

/// Whether the entry names the topic, ignoring case: one of the values of its `topic` field.
fn names(entry: &Entry, topic: &str) -> bool {
    let wanted = topic.to_lowercase();
    entry
        .list(TOPIC)
        .iter()
        .any(|named| named.to_lowercase() == wanted)
}

/// `Topics: pumps (2), rig (0) · dossier list --topic <name>`, or None when there are no topics.
fn topics_line(topics: &[TopicCount]) -> Option<String> {
    if topics.is_empty() {
        return None;
    }
    let parts: Vec<String> = topics
        .iter()
        .map(|topic| format!("{} ({})", topic.name, topic.decisions))
        .collect();
    Some(format!(
        "Topics: {} · dossier list --topic <name>",
        parts.join(", ")
    ))
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

/// The decision as printed: `<ID> · <title> (<author>, <date>)`, with the topics it names between
/// the ID and the title, its review marks and its rests-on mark, and its bodies. `state` is the
/// state folder of the dossier.
fn decision_of(item: &Item, state: &Path, wrong: &Wrong) -> Decision {
    let entry = &item.entry;
    let date = entry
        .field("created")
        .map(|stamp| time::date(&stamp).to_string())
        .filter(|date| !date.is_empty())
        .unwrap_or_else(|| "-".to_string());
    Decision {
        id: entry.id(),
        summary: format!(
            "{} ({}, {date}){}{}",
            decision_text(entry),
            author(entry),
            review_marks(state, item),
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

/// The review marks of an entry shown by the brief, in the order `show` gives them (R-READ-1):
/// `[due for review]` when its review date has passed (R-LIFE-7), then `[changed after approval]`
/// for an approved lesson, how-to or tool (approval::guarded) whose text differs from its recorded
/// checksum (R-LIFE-6). `state` is the state folder of the dossier the entry is in.
fn review_marks(state: &Path, item: &Item) -> String {
    let entry = &item.entry;
    let changed = entry.kind().is_some_and(approval::guarded)
        && entry.status() == Some(Status::Approved)
        && matches!(approval::mark(state, &entry.id(), &item.raw), Mark::Changed);
    let changed_mark = if changed {
        " [changed after approval]"
    } else {
        ""
    };
    format!("{}{changed_mark}", due_mark(entry))
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

/// The heading of an entry: `<ID> · <title>`, or the ID alone when there is no title.
fn entry_text(entry: &Entry) -> String {
    let title = entry.title();
    if title.is_empty() {
        entry.id()
    } else {
        format!("{} · {title}", entry.id())
    }
}

/// The heading of a decision: `entry_text`, with the topics the entry names in brackets after the
/// ID, as `<ID> · [<topic>, <topic>] <title>` (R-TOPIC-1). The topics are as written in the entry.
/// A decision that names no topic has the heading of `entry_text`.
fn decision_text(entry: &Entry) -> String {
    let topics = entry.list(TOPIC);
    if topics.is_empty() {
        return entry_text(entry);
    }
    let named = format!("[{}]", topics.join(", "));
    let title = entry.title();
    if title.is_empty() {
        format!("{} · {named}", entry.id())
    } else {
        format!("{} · {named} {title}", entry.id())
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

/// `1 entry of <milestone> is summed up in <DIG-ID>: dossier show <DIG-ID>`, or `<n> entries of
/// <milestone> are summed up in <DIG-ID>: dossier show <DIG-ID>` (design.md section 18).
fn summed_text(summed: &Summed) -> String {
    let (count, milestone, id) = (summed.count, &summed.milestone, &summed.digest);
    let what = if count == 1 {
        format!("1 entry of {milestone} is")
    } else {
        format!("{count} entries of {milestone} are")
    };
    format!("{what} summed up in {id}: dossier show {id}")
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

/// The team dossier of the project as a context (R-PLACE-8), when the project names one that can
/// be reached. A team dossier that cannot be read is a problem, and the brief has no team part.
fn team_context(ctx: &Ctx, problems: &mut Vec<String>) -> Option<Ctx> {
    // No team dossier named, or not reachable (then Ctx::load left `team` empty).
    ctx.team.as_ref()?;
    match ctx.team_ctx() {
        Ok(team) => Some(team),
        Err(error) => {
            problems.push(error.message);
            None
        }
    }
}

/// The team part of the brief (R-CTX-1): the approved lessons and how-tos of the team whose
/// machine, analysis or software tags match the project's, in the order of `store::all`; with `day`
/// the ones in force on it. At most MAX_TEAM are returned, with the number of matching ones beyond
/// that. The problems of reading the team dossier come back with their paths shown relative to the
/// team folder.
fn team_part(ctx: &Ctx, team: &Ctx, day: Option<&str>) -> (Vec<Line>, usize, Vec<String>) {
    let (items, warnings) = store::all(&team.dossier);
    let matching: Vec<&Item> = items
        .iter()
        .filter(|item| matches!(item.entry.kind(), Some(Kind::Lesson | Kind::Howto)))
        .filter(|item| match day {
            Some(day) => in_force_on(&item.entry, day),
            None => item.entry.status() == Some(Status::Approved),
        })
        .filter(|item| matches_tags(&item.entry, &ctx.cfg.tags))
        .collect();
    let left_out = matching.len().saturating_sub(MAX_TEAM);
    let lines = matching
        .into_iter()
        .take(MAX_TEAM)
        .map(|item| Line {
            text: format!(
                "team:{}{}",
                entry_text(&item.entry),
                review_marks(&team.state, item)
            ),
            id: item.entry.id(),
        })
        .collect();
    let problems = warnings
        .iter()
        .map(|warning| format!("team: {}", shown_warning(&team.root, warning)))
        .collect();
    (lines, left_out, problems)
}

/// Whether an entry shares a machine, analysis or software tag with the project, ignoring case.
fn matches_tags(entry: &Entry, tags: &Tags) -> bool {
    shares(entry, "machine", &tags.machine)
        || shares(entry, "analysis", &tags.analysis)
        || shares(entry, "software", &tags.software)
}

/// Whether one of the comma-separated values of `key` equals one of `tags`, ignoring case.
fn shares(entry: &Entry, key: &str, tags: &[String]) -> bool {
    entry
        .list(key)
        .iter()
        .filter(|value| !value.is_empty())
        .any(|value| {
            tags.iter()
                .any(|tag| tag.trim().eq_ignore_ascii_case(value))
        })
}

/// The news of the brief (R-READ-11): how many entries were approved in the project or in the team
/// after the person's marker, and the day of the marker.
struct News {
    count: usize,
    since: String,
}

impl News {
    /// `1 new entry since <date>: dossier status --since last`, or `<n> new entries since <date>:
    /// ...`. None when there is no news.
    fn line(&self) -> Option<String> {
        if self.count == 0 {
            return None;
        }
        let what = if self.count == 1 {
            "1 new entry".to_string()
        } else {
            format!("{} new entries", self.count)
        };
        Some(format!(
            "{what} since {}: dossier status --since last",
            self.since
        ))
    }

    /// The news as data: the count, and the day of the marker.
    fn data(&self) -> Value {
        json!({"count": self.count, "since": self.since})
    }
}

/// The news of the project (R-READ-11): the approvals in the journal of the project and of its
/// team after the person's marker, `.state/seen-<person>.txt`, which `status --since last` writes.
/// None when no person is named, and when the person has no marker yet: then nothing is new since
/// a run, and the first `status --since last` starts the count.
fn news_of(ctx: &Ctx, team: Option<&Ctx>, problems: &mut Vec<String>) -> Option<News> {
    let who = ctx.who.as_ref()?;
    let path = ctx
        .state
        .join(format!("seen-{}.txt", who.person.to_lowercase()));
    let marker = marker_of(&ctx.root, &path, problems)?;
    let mut count = approvals_since(&ctx.state, &marker, problems);
    if let Some(team) = team {
        count += approvals_since(&team.state, &marker, problems);
    }
    Some(News {
        count,
        since: time::date(&marker).to_string(),
    })
}

/// The time in a person's marker file. None when there is no file. A file that holds no time, or
/// cannot be read, is a problem, and then no news is shown.
fn marker_of(root: &Path, path: &Path, problems: &mut Vec<String>) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let stamp = text.trim();
            if time::valid(stamp) {
                Some(stamp.to_string())
            } else {
                problems.push(format!(
                    "{} does not hold a time; no news is shown",
                    shown_path(root, path)
                ));
                None
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            problems.push(format!(
                "{}: {error}; no news is shown",
                shown_path(root, path)
            ));
            None
        }
    }
}

/// The approvals in the journal of `state` made after the time `after`. A journal that cannot be
/// read is a problem and counts nothing.
fn approvals_since(state: &Path, after: &str, problems: &mut Vec<String>) -> usize {
    match journal::read(state) {
        Ok(records) => records
            .iter()
            .filter(|record| is_approval(record))
            .filter(|record| later(&record.time, after))
            .count(),
        Err(error) => {
            problems.push(error.message);
            0
        }
    }
}

/// Whether a journal record is an approval: an `approved` or `accepted` event, or a `created` event
/// whose detail is `approved` (create.rs).
fn is_approval(record: &Record) -> bool {
    match record.event.as_str() {
        "approved" | "accepted" => true,
        "created" => record.detail == "approved",
        _ => false,
    }
}

/// Whether the time `stamp` is after `last`. False when either is not a time.
fn later(stamp: &str, last: &str) -> bool {
    match (
        DateTime::parse_from_rfc3339(stamp),
        DateTime::parse_from_rfc3339(last),
    ) {
        (Ok(stamp), Ok(last)) => stamp > last,
        _ => false,
    }
}

/// A path as people see it: relative to the project folder, with `/` as the separator.
fn shown_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.to_string_lossy().replace(MAIN_SEPARATOR, "/")
}
