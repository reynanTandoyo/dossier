//! The lines of `brief --task`: what a session needs to know about one task.

use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::{Code, Error, Result};
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::todo_bridge;
use serde_json::Value;
use std::collections::HashSet;

/// The heading over the approved entries whose `task` field names the task.
const RECORDED: &str = "## Recorded for this task";
/// The heading over the approved decisions of the task's milestones.
const MILESTONES: &str = "## Decisions of its milestones";
/// The heading over the approved entries that match the task's tags and title words.
const RELATED: &str = "## Related";
/// The most entries `select::best` chooses for the Related heading.
const RELATED_LIMIT: usize = 10;
/// What `brief --task` says when todo is not installed (R-CTX-5).
const NEEDS_TODO: &str = "brief --task needs todo";

/// The lines of `brief --task`: the task's title and goal lines from todo, the entries whose
/// `task` field names it, the decisions whose task is one of its milestones, then
/// select::best over the remaining approved entries with the task's machine, analysis and
/// software fields as tags and its title as words.
pub fn lines(ctx: &Ctx, task: &str, limit: usize) -> Result<Vec<String>> {
    let data = todo_bridge::json(&ctx.root, &["show", task]).map_err(|error| {
        if error.code == Code::TodoMissing {
            Error::new(Code::TodoMissing, NEEDS_TODO)
        } else {
            error
        }
    })?;
    let about = About::read(&data);
    let (items, _warnings) = store::all(&ctx.dossier);
    let approved: Vec<Item> = items
        .into_iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .collect();

    // An entry is listed once, under the first heading that takes it.
    let recorded: Vec<&Item> = approved.iter().filter(|item| task_is(item, task)).collect();
    let mut listed: HashSet<String> = recorded.iter().map(|item| item.entry.id()).collect();
    let decisions: Vec<&Item> = approved
        .iter()
        .filter(|item| item.entry.kind() == Some(Kind::Decision))
        .filter(|item| !listed.contains(&item.entry.id()))
        .filter(|item| {
            about
                .milestones
                .iter()
                .any(|milestone| task_is(item, milestone))
        })
        .collect();
    listed.extend(decisions.iter().map(|item| item.entry.id()));

    let mut out = vec![task_line(task, &about.title)];
    if !recorded.is_empty() {
        out.push(RECORDED.to_string());
        out.extend(recorded.iter().map(|item| summary(item)));
    }
    if !decisions.is_empty() {
        out.push(MILESTONES.to_string());
        for item in &decisions {
            out.push(summary(item));
            out.extend(indented(
                &item.entry.section("Decision").unwrap_or_default(),
            ));
        }
    }

    let query = Query {
        tags: about.tags(),
        words: select::words(&about.title),
    };
    let pool: Vec<Item> = approved
        .into_iter()
        .filter(|item| !listed.contains(&item.entry.id()))
        .collect();
    let related = select::best(&pool, &query, RELATED_LIMIT);
    if !related.is_empty() {
        out.push(RELATED.to_string());
        for item in related {
            out.push(summary(item));
            out.extend(lesson_lines(item));
        }
    }
    Ok(cut(out, limit))
}

/// What `brief --task` reads from `todo show <task> --json`. A missing value is empty.
struct About {
    title: String,
    machine: String,
    analysis: String,
    software: String,
    milestones: Vec<String>,
}

impl About {
    fn read(data: &Value) -> About {
        About {
            title: first_text([data.get("title"), inner(data, "title")]),
            machine: first_text([inner(data, "machine"), data.get("machine")]),
            analysis: first_text([inner(data, "analysis"), data.get("analysis")]),
            software: first_text([inner(data, "software"), data.get("software")]),
            milestones: milestone_ids(data),
        }
    }

    /// The machine, analysis and software values, each split at commas.
    fn tags(&self) -> Vec<String> {
        [&self.machine, &self.analysis, &self.software]
            .into_iter()
            .flat_map(|list| list.split(','))
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect()
    }
}

/// The value of `key` in the `fields` object of todo's data, when there is one.
fn inner<'a>(data: &'a Value, key: &str) -> Option<&'a Value> {
    data.get("fields")?.get(key)
}

/// The first string among `values`, trimmed; empty when none of them is a string.
fn first_text<'a>(values: impl IntoIterator<Item = Option<&'a Value>>) -> String {
    values
        .into_iter()
        .flatten()
        .find_map(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// The milestone IDs: the `id` of each item of the `milestones` list when that gives any, else
/// the string `milestone`. Empty when neither does.
fn milestone_ids(data: &Value) -> Vec<String> {
    let listed: Vec<String> = data
        .get("milestones")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    if !listed.is_empty() {
        return listed;
    }
    let single = first_text([data.get("milestone")]);
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}

/// Whether two IDs or names are the same, ignoring case and surrounding white space.
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Whether the entry's `task` field names `wanted`.
fn task_is(item: &Item, wanted: &str) -> bool {
    item.entry
        .field("task")
        .is_some_and(|value| same(&value, wanted))
}

/// The first line of the task: `Task <task> · <title>`.
fn task_line(task: &str, title: &str) -> String {
    if title.is_empty() {
        format!("Task {task}")
    } else {
        format!("Task {task} · {title}")
    }
}

/// One entry as one line: `<ID> · <kind> · <title>`.
fn summary(item: &Item) -> String {
    let kind = item.entry.kind().map_or("unknown", Kind::name);
    format!("{} · {kind} · {}", item.entry.id(), item.entry.title())
        .trim_end()
        .to_string()
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

/// Under a lesson in Related: the first line of its Statement and of its Valid for. A section
/// that is missing or empty gives no line; other kinds give none.
fn lesson_lines(item: &Item) -> Vec<String> {
    if item.entry.kind() != Some(Kind::Lesson) {
        return Vec::new();
    }
    ["Statement", "Valid for"]
        .into_iter()
        .filter_map(|name| {
            let body = item.entry.section(name)?;
            let first = body.lines().next()?;
            Some(format!("  {name}: {first}"))
        })
        .collect()
}

/// The first `limit` lines. When there are more, the last line kept is replaced by
/// `<m> more left out`, where m counts the lines left out, including the one whose place that
/// line takes. With a limit of 0 nothing is kept.
fn cut(lines: Vec<String>, limit: usize) -> Vec<String> {
    if lines.len() <= limit {
        return lines;
    }
    if limit == 0 {
        return Vec::new();
    }
    let left_out = lines.len() - (limit - 1);
    let mut kept: Vec<String> = lines.into_iter().take(limit - 1).collect();
    kept.push(format!("{left_out} more left out"));
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::entry::new::{Draft, render};
    use std::fs;
    use std::path::{Path, PathBuf};

    const CREATED: &str = "2026-11-20T09:00:00+01:00";

    /// A project folder with a DOSSIER folder inside: its four kind folders and dossier.toml.
    /// The folder's name has a space and a non-ASCII letter. Returns the temporary folder, which
    /// keeps the files alive, and the context built as a struct literal.
    fn project() -> (tempfile::TempDir, Ctx) {
        let dir = tempfile::Builder::new()
            .prefix("brief Prüf ")
            .tempdir()
            .expect("temporary folder");
        let root = dir.path().to_path_buf();
        let dossier = root.join("DOSSIER");
        for kind in Kind::all() {
            fs::create_dir_all(dossier.join(kind.folder())).expect("kind folder");
        }
        fs::write(dossier.join("dossier.toml"), config::default_text("P-001"))
            .expect("write the settings");
        let cfg = config::load(&dossier).expect("the settings load");
        let ctx = Ctx {
            state: dossier.join(".state"),
            root,
            dossier,
            cfg,
            who: None,
            json: false,
            dry_run: false,
            command: "brief".to_string(),
        };
        (dir, ctx)
    }

    /// Writes an entry file at the place the store finds it. Its status, created time and
    /// author come first, then `fields`; `sections` are the bodies, by name.
    fn write_entry(
        ctx: &Ctx,
        kind: Kind,
        id: &str,
        title: &str,
        status: &str,
        fields: &[(&str, &str)],
        sections: &[(&str, &str)],
    ) {
        let mut all_fields = vec![("status", status), ("created", CREATED), ("author", "anna")];
        all_fields.extend_from_slice(fields);
        let draft = Draft {
            kind,
            id: id.to_string(),
            title: title.to_string(),
            fields: all_fields
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            sections: sections
                .iter()
                .map(|(name, body)| (name.to_string(), body.to_string()))
                .collect(),
        };
        let path = store::path_for(&ctx.dossier, kind, id, title);
        fs::write(&path, render(&draft)).expect("write the entry");
    }

    /// The approved and draft entries the tests of `lines` read. The task T-132 has DEC-001 and
    /// FND-001 (the second written with a lower-case task); T-130, its milestone, has DEC-003
    /// (approved), DEC-004 (draft) and FND-004 (a finding, not a decision).
    fn write_entries(ctx: &Ctx) {
        write_entry(
            ctx,
            Kind::Decision,
            "DEC-001",
            "Run the speedline transient",
            "approved",
            &[("task", "T-132"), ("machine", "axial compressor")],
            &[
                (
                    "Decision",
                    "The last points of the speedline are run transient.",
                ),
                ("Reason", "Steady runs stall early."),
            ],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-001",
            "Steady runs stall early",
            "approved",
            &[("task", "t-132")],
            &[(
                "Observation",
                "The steady run stalled at the lowest mass flow.",
            )],
        );
        write_entry(
            ctx,
            Kind::Decision,
            "DEC-003",
            "Use transient runs on the speedline",
            "approved",
            &[("task", "T-130"), ("machine", "axial compressor")],
            &[
                ("Decision", "Run the last three speedline points transient."),
                (
                    "Reason",
                    "Steady runs converged where rotating stall was seen.",
                ),
            ],
        );
        write_entry(
            ctx,
            Kind::Decision,
            "DEC-004",
            "Draft milestone choice",
            "draft",
            &[("task", "T-130")],
            &[("Decision", "Never listed.")],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-004",
            "Milestone review notes",
            "approved",
            &[("task", "T-130")],
            &[("Observation", "Notes were kept.")],
        );
        write_entry(
            ctx,
            Kind::Lesson,
            "LES-001",
            "Steady mixing-plane runs overpredict stall margin",
            "approved",
            &[
                ("task", "T-042"),
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                ("confidence", "simulated"),
            ],
            &[
                (
                    "Statement",
                    "Near the stability limit, steady runs with a mixing-plane interface converged at mass flows\nwhere the transient run already showed rotating stall.",
                ),
                (
                    "Valid for",
                    "Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.",
                ),
            ],
        );
        write_entry(
            ctx,
            Kind::Lesson,
            "LES-002",
            "Draft lesson on the compressor",
            "draft",
            &[("task", "T-042"), ("machine", "axial compressor")],
            &[
                ("Statement", "Never listed."),
                ("Valid for", "Never listed."),
            ],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-002",
            "Speedline check on axial compressor",
            "approved",
            &[("task", "T-200"), ("machine", "axial compressor")],
            &[("Observation", "The check agreed with the rig.")],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-003",
            "Mesh independence reached",
            "approved",
            &[("task", "T-200"), ("machine", "radial turbine")],
            &[("Observation", "Grid counts were compared.")],
        );
    }

    /// Writes a stand-in for todo in `folder` that prints `line` and exits, and returns its path.
    #[cfg(windows)]
    fn stand_in(folder: &Path, name: &str, line: &str) -> PathBuf {
        let path = folder.join(format!("{name}.cmd"));
        fs::write(&path, format!("@echo off\r\necho {line}\r\n")).expect("the stand-in");
        path
    }

    /// Writes a stand-in for todo in `folder` that prints `line` and exits, and returns its path.
    #[cfg(not(windows))]
    fn stand_in(folder: &Path, name: &str, line: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = folder.join(name);
        fs::write(&path, format!("#!/bin/sh\nprintf '%s\\n' '{line}'\n")).expect("the stand-in");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
        path
    }

    /// todo's envelope for `show` with `data`, on one line.
    fn envelope(data: Value) -> String {
        serde_json::json!({
            "schema": "todo.show.v1",
            "ok": true,
            "data": data,
            "warnings": []
        })
        .to_string()
    }

    /// The data of the task T-132, as `todo show T-132 --json` gives it (invented values).
    fn speedline_task() -> Value {
        serde_json::json!({
            "id": "T-132",
            "title": "Speedline of the compressor",
            "status": "open",
            "milestone": "T-130",
            "milestones": [
                {"id": "T-130", "title": "Context", "goal": "An invented goal", "status": "open"},
                {"id": "T-001", "title": "Core", "goal": "An invented goal", "status": "open"}
            ],
            "analysis": "steady RANS",
            "fields": {"machine": "axial compressor"}
        })
    }

    #[test]
    fn lines_give_the_task_its_entries_its_milestones_and_the_related_entries() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        write_entries(&ctx);
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let all = lines(&ctx, "T-132", 100);
        let cut_short = lines(&ctx, "T-132", 5);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            all.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Recorded for this task",
                "DEC-001 · decision · Run the speedline transient",
                "FND-001 · finding · Steady runs stall early",
                "## Decisions of its milestones",
                "DEC-003 · decision · Use transient runs on the speedline",
                "  Run the last three speedline points transient.",
                "## Related",
                "FND-002 · finding · Speedline check on axial compressor",
                "LES-001 · lesson · Steady mixing-plane runs overpredict stall margin",
                "  Statement: Near the stability limit, steady runs with a mixing-plane interface converged at mass flows",
                "  Valid for: Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.",
            ]
        );
        // Twelve lines in all: five are kept, the last one counts the eight left out.
        assert_eq!(
            cut_short.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Recorded for this task",
                "DEC-001 · decision · Run the speedline transient",
                "FND-001 · finding · Steady runs stall early",
                "8 more left out",
            ]
        );
    }

    #[test]
    fn a_heading_with_no_line_is_left_out() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        write_entries(&ctx);
        let quiet = stand_in(
            dir.path(),
            "todo-quiet",
            &envelope(serde_json::json!({"id": "T-900", "title": "Quiet task"})),
        );
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &quiet) };

        let found = lines(&ctx, "T-900", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(found.expect("the lines"), vec!["Task T-900 · Quiet task"]);
    }

    #[test]
    fn without_todo_the_error_is_todo_missing() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        write_entries(&ctx);
        let missing = dir.path().join("no-such-todo");
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &missing) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        let error = found.expect_err("brief --task needs todo");
        assert_eq!(error.code, Code::TodoMissing);
        assert_eq!(error.message, "brief --task needs todo");
    }

    #[test]
    fn cut_keeps_the_limit_and_counts_the_lines_left_out() {
        let numbered = |count: usize| -> Vec<String> {
            (1..=count).map(|number| format!("line {number}")).collect()
        };
        assert_eq!(cut(numbered(3), 5), numbered(3));
        assert_eq!(cut(numbered(4), 4), numbered(4));
        assert_eq!(
            cut(numbered(4), 3),
            vec!["line 1", "line 2", "2 more left out"]
        );
        assert!(cut(numbered(4), 0).is_empty());
    }
}
