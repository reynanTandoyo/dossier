//! The lines of `brief --task`: what a session needs to know about one task.

use crate::blockfile::BlockFile;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::todo_bridge;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

/// The heading over the approved entries whose `task` field names the task.
const RECORDED: &str = "## Recorded for this task";
/// The heading over the approved decisions of the task's milestones.
const MILESTONES: &str = "## Decisions of its milestones";
/// The heading over the approved entries that match the task's tags and title words.
const RELATED: &str = "## Related";
/// The most entries `select::best` chooses for the Related heading.
const RELATED_LIMIT: usize = 10;
/// The heading over the task's file list (R-FILE-14).
const FILES: &str = "## Files";
/// The most lines of the file list (R-FILE-14).
const FILES_LINES: usize = 15;
/// The heading over the approved dead ends that match the task's tags and title words (R-CTX-16).
const DEAD_ENDS: &str = "## Dead ends";
/// The most dead ends `select::best` chooses for the Dead ends heading.
const DEAD_ENDS_LIMIT: usize = 5;
/// The heading over the approved entries whose code pointers lie in the files todo names for the
/// task (R-CODE-4).
const CODE: &str = "## Code";
/// The section of an entry that holds its pictures (format.md §16).
const PICTURES: &str = "Pictures";
/// What `brief --task` says when todo is not installed (R-CTX-5).
const NEEDS_TODO: &str = "brief --task needs todo";

/// The lines of `brief --task`: the task's title and goal lines from todo, the entries whose
/// `task` field names it, the decisions whose task is one of its milestones, the task's file list
/// (R-FILE-14), the approved dead ends that match it (R-CTX-16), the entries whose code pointers
/// lie in the files todo names for it (R-CODE-4), then select::best over the remaining approved
/// entries with the task's machine, analysis and software fields as tags and its title as words.
/// A decision or lesson printed with its text is followed by its pictures (R-ENTRY-17).
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
            out.extend(picture_lines(item));
        }
    }
    out.extend(files_lines(ctx, task));

    let query = Query {
        tags: about.tags(),
        words: select::words(&about.title),
    };
    let (dead_ends, mut pool): (Vec<Item>, Vec<Item>) = approved
        .into_iter()
        .filter(|item| !listed.contains(&item.entry.id()))
        .partition(is_dead_end);
    let found = select::best(&dead_ends, &query, DEAD_ENDS_LIMIT);
    if !found.is_empty() {
        out.push(DEAD_ENDS.to_string());
        for item in &found {
            out.push(summary(item));
            out.extend(section_line(item, "Observation"));
        }
    }
    listed.extend(found.iter().map(|item| item.entry.id()));
    pool.extend(
        dead_ends
            .into_iter()
            .filter(|item| !listed.contains(&item.entry.id())),
    );

    if let Some(paths) = &about.files {
        let code: Vec<(&Item, Vec<String>)> = pool
            .iter()
            .map(|item| (item, pointers_in(&item.entry, paths)))
            .filter(|(_, pointers)| !pointers.is_empty())
            .collect();
        if !code.is_empty() {
            out.push(CODE.to_string());
            for (item, pointers) in &code {
                out.push(summary(item));
                out.push(format!("  code: {}", pointers.join(", ")));
            }
        }
        listed.extend(code.iter().map(|(item, _)| item.entry.id()));
    }

    let pool: Vec<Item> = pool
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
    /// The paths of todo's `files` field; None when the data has no such field.
    files: Option<Vec<String>>,
}

impl About {
    fn read(data: &Value) -> About {
        About {
            title: first_text([data.get("title"), inner(data, "title")]),
            machine: first_text([inner(data, "machine"), data.get("machine")]),
            analysis: first_text([inner(data, "analysis"), data.get("analysis")]),
            software: first_text([inner(data, "software"), data.get("software")]),
            milestones: milestone_ids(data),
            files: todo_files(data),
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

/// Under a lesson in Related: the first line of its Statement and of its Valid for, then one line
/// per picture (R-ENTRY-17). A section that is missing or empty gives no line; other kinds give
/// none.
fn lesson_lines(item: &Item) -> Vec<String> {
    if item.entry.kind() != Some(Kind::Lesson) {
        return Vec::new();
    }
    let mut lines: Vec<String> = ["Statement", "Valid for"]
        .into_iter()
        .filter_map(|name| section_line(item, name))
        .collect();
    lines.extend(picture_lines(item));
    lines
}

/// The first line of the section `name` of the entry, as `  <name>: <line>`. A missing or empty
/// section gives None.
fn section_line(item: &Item, name: &str) -> Option<String> {
    let body = item.entry.section(name)?;
    let first = body.lines().next()?;
    Some(format!("  {name}: {first}"))
}

/// One line per picture of the entry's Pictures section (R-ENTRY-17), as
/// `  picture: images/<ID>/<file> — <caption>`.
fn picture_lines(item: &Item) -> Vec<String> {
    item.entry
        .section(PICTURES)
        .unwrap_or_default()
        .lines()
        .filter_map(picture_line)
        .collect()
}

/// The picture line of one line `![<caption>](../images/<ID>/<file>)` (format.md §16). Other text
/// gives None.
fn picture_line(line: &str) -> Option<String> {
    let (caption, path) = line.strip_prefix("![")?.rsplit_once("](../")?;
    let path = path.strip_suffix(')')?;
    Some(format!("  picture: {path} — {caption}"))
}

/// The Files heading (R-FILE-14): the task's file list, descriptions and models first, then the
/// other roles, each in the order of the list. A detached block is left out. One line per file:
/// `<path> (<role>)`, with its marks in brackets. At most FILES_LINES lines; no heading when the
/// list has no file. A list that cannot be read gives the heading and one line that says so, and
/// brief --task goes on with its other parts.
fn files_lines(ctx: &Ctx, task: &str) -> Vec<String> {
    let (file, _bytes) = match filelist::load(&ctx.dossier, task) {
        Ok(loaded) => loaded,
        Err(error) => {
            return vec![
                FILES.to_string(),
                format!(
                    "file list cannot be read: {}; run dossier check",
                    error.message
                ),
            ];
        }
    };
    let mut keys: Vec<String> = file
        .keys()
        .into_iter()
        .filter(|key| !file.field(key, "detached").is_some_and(|at| !at.is_empty()))
        .collect();
    keys.sort_by_key(|key| u8::from(!main_role(&file, key)));
    let rows: Vec<String> = keys
        .iter()
        .map(|key| file_line(&ctx.root, &file, key))
        .collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let mut out = vec![FILES.to_string()];
    out.extend(cut(rows, FILES_LINES));
    out
}

/// Whether the block's role is `description` or `model`, ignoring case (R-FILE-14).
fn main_role(file: &BlockFile, key: &str) -> bool {
    let role = file
        .field(key, "role")
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    role == "description" || role == "model"
}

/// One file of the list as a line: `<path> (<role>)`, then its marks in brackets. A block without
/// a role has the path alone.
fn file_line(root: &Path, file: &BlockFile, key: &str) -> String {
    let role = file.field(key, "role").unwrap_or_default();
    let role = role.trim();
    let line = if role.is_empty() {
        key.to_string()
    } else {
        format!("{key} ({role})")
    };
    let marks = filelist::marks(root, file, key);
    if marks.is_empty() {
        line
    } else {
        format!("{line} [{}]", marks.join(", "))
    }
}

/// Whether the entry is a finding whose outcome is a dead end (format.md §5, §8).
fn is_dead_end(item: &Item) -> bool {
    item.entry.kind() == Some(Kind::Finding)
        && item
            .entry
            .field("outcome")
            .is_some_and(|outcome| same(&outcome, "dead end"))
}

/// The code pointers of the entry that lie in one of `paths` (R-CODE-4). A pointer is `path` or
/// `path::name` (format.md §14). Its path lies in one of `paths` when they are the same, or when
/// it is inside that folder.
fn pointers_in(entry: &Entry, paths: &[String]) -> Vec<String> {
    entry
        .list("code")
        .into_iter()
        .filter(|pointer| {
            let path = clean_path(
                pointer
                    .split_once("::")
                    .map_or(pointer.as_str(), |(path, _)| path),
            );
            paths.iter().any(|wanted| {
                path == wanted.as_str()
                    || path
                        .strip_prefix(wanted.as_str())
                        .is_some_and(|rest| rest.starts_with('/'))
            })
        })
        .collect()
}

/// A stored path as a list writes it: trimmed, with `/` for `\`, and without a leading `./` and a
/// trailing `/`.
fn clean_path(text: &str) -> String {
    text.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

/// The paths of the todo data's `files` field (todo 0.8 file scopes): a comma-separated string or
/// a list of strings, at the top of the data or under `fields`. None when there is no such field.
fn todo_files(data: &Value) -> Option<Vec<String>> {
    let value = [data.get("files"), inner(data, "files")]
        .into_iter()
        .flatten()
        .find(|value| value.is_string() || value.is_array())?;
    let items: Vec<&str> = match value {
        Value::String(text) => text.split(',').collect(),
        Value::Array(list) => list.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    Some(
        items
            .into_iter()
            .map(clean_path)
            .filter(|path| !path.is_empty())
            .collect(),
    )
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

    /// The file list of T-132 (invented): a result that is not on disk, a model and a description
    /// (both written to disk here) and a detached result.
    fn write_task_files(ctx: &Ctx) {
        fs::create_dir_all(ctx.dossier.join("files")).expect("the files folder");
        let list = "<!-- dossier-format: 1 -->\n# Files of T-132\n\n## results/extra.csv\n\n- role: result\n\n## data/speedline.sim\n\n- role: model\n\n## results/old.csv\n\n- role: result\n- detached: 2026-11-21T09:00:00+01:00 by anna\n\n## notes/setup.md\n\n- role: description\n";
        fs::write(ctx.dossier.join("files").join("T-132.md"), list).expect("the list of T-132");
        for path in ["data/speedline.sim", "notes/setup.md"] {
            let full = ctx.root.join(path);
            fs::create_dir_all(full.parent().expect("a folder")).expect("the folder");
            fs::write(full, "invented content").expect("the file");
        }
    }

    /// Entries for the test of the files, dead ends, pictures and code pointers of T-132 (invented).
    /// DEC-011 is a draft, so it is never listed.
    fn write_task_material(ctx: &Ctx) {
        write_entry(
            ctx,
            Kind::Decision,
            "DEC-010",
            "Use transient runs on the speedline",
            "approved",
            &[("task", "T-130"), ("machine", "axial compressor")],
            &[
                ("Decision", "Run the last three speedline points transient."),
                ("Reason", "Steady runs stall early."),
                (
                    "Pictures",
                    "![Speedline of stage 1 at design speed](../images/DEC-010/speedline-stage-1.png)",
                ),
            ],
        );
        write_entry(
            ctx,
            Kind::Decision,
            "DEC-011",
            "Run steady points as well",
            "draft",
            &[("task", "T-130")],
            &[
                ("Decision", "Never listed."),
                (
                    "Pictures",
                    "![Draft plot](../images/DEC-011/draft-plot.png)",
                ),
            ],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-010",
            "Higher seal pressure on the compressor did not help",
            "approved",
            &[
                ("task", "T-050"),
                ("machine", "axial compressor"),
                ("outcome", "dead end"),
            ],
            &[(
                "Observation",
                "The leak stayed at 0.2 kg/s with twice the seal pressure.",
            )],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-011",
            "Seal leak model checked against the rig",
            "approved",
            &[("task", "T-050"), ("code", "src/pump/seal.rs::leak_model")],
            &[(
                "Observation",
                "The model gives the leak of the rig within ten percent.",
            )],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-012",
            "Mesh independence reached",
            "approved",
            &[
                ("task", "T-050"),
                ("machine", "radial turbine"),
                ("code", "src/other/valve.rs"),
            ],
            &[("Observation", "Grid counts were compared.")],
        );
        write_entry(
            ctx,
            Kind::Finding,
            "FND-013",
            "Setup notes checked against rig",
            "approved",
            &[("task", "T-050"), ("code", "docs/setup.md")],
            &[("Observation", "The notes match the rig.")],
        );
        write_entry(
            ctx,
            Kind::Lesson,
            "LES-010",
            "Steady runs overpredict stall margin of the compressor",
            "approved",
            &[
                ("task", "T-042"),
                ("machine", "axial compressor"),
                ("confidence", "simulated"),
            ],
            &[
                (
                    "Statement",
                    "Steady mixing-plane runs converge at mass flows where rotating stall is seen.",
                ),
                ("Valid for", "Transonic front stages."),
                (
                    "Pictures",
                    "![Stall margin by method](../images/LES-010/stall-margin-by-method.svg)",
                ),
            ],
        );
    }

    #[test]
    fn brief_task_gives_the_files_dead_ends_pictures_and_code_of_the_task() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        write_task_files(&ctx);
        write_task_material(&ctx);
        let mut data = speedline_task();
        data["files"] = serde_json::json!("src/pump, docs/setup.md");
        let show = stand_in(dir.path(), "todo-show", &envelope(data));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Decisions of its milestones",
                "DEC-010 · decision · Use transient runs on the speedline",
                "  Run the last three speedline points transient.",
                "  picture: images/DEC-010/speedline-stage-1.png — Speedline of stage 1 at design speed",
                "## Files",
                "data/speedline.sim (model)",
                "notes/setup.md (description)",
                "results/extra.csv (result) [missing]",
                "## Dead ends",
                "FND-010 · finding · Higher seal pressure on the compressor did not help",
                "  Observation: The leak stayed at 0.2 kg/s with twice the seal pressure.",
                "## Code",
                "FND-011 · finding · Seal leak model checked against the rig",
                "  code: src/pump/seal.rs::leak_model",
                "FND-013 · finding · Setup notes checked against rig",
                "  code: docs/setup.md",
                "## Related",
                "LES-010 · lesson · Steady runs overpredict stall margin of the compressor",
                "  Statement: Steady mixing-plane runs converge at mass flows where rotating stall is seen.",
                "  Valid for: Transonic front stages.",
                "  picture: images/LES-010/stall-margin-by-method.svg — Stall margin by method",
            ]
        );
    }

    #[test]
    fn the_file_list_gives_at_most_fifteen_lines() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        fs::create_dir_all(ctx.dossier.join("files")).expect("the files folder");
        let mut list = String::from("<!-- dossier-format: 1 -->\n# Files of T-132\n\n");
        for number in 1..=17 {
            list.push_str(&format!(
                "## data/part-{number:02}.csv\n\n- role: result\n\n"
            ));
        }
        fs::write(ctx.dossier.join("files").join("T-132.md"), list).expect("the list of T-132");
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        // The task line, the heading, then fifteen file lines: fourteen files and the count of the
        // three left out.
        let found = found.expect("the lines");
        assert_eq!(found.len(), 17, "{found:?}");
        assert_eq!(found[1], "## Files");
        assert_eq!(found[2], "data/part-01.csv (result) [missing]");
        assert_eq!(found[15], "data/part-14.csv (result) [missing]");
        assert_eq!(found[16], "3 more left out");
    }

    #[test]
    fn a_broken_file_list_is_reported_and_the_other_parts_go_on() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        write_task_material(&ctx);
        fs::create_dir_all(ctx.dossier.join("files")).expect("the files folder");
        // A list with no heading line is broken (format.md §17).
        fs::write(
            ctx.dossier.join("files").join("T-132.md"),
            "<!-- dossier-format: 1 -->\nno heading in this file\n",
        )
        .expect("the broken list of T-132");
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("brief --task goes on"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Decisions of its milestones",
                "DEC-010 · decision · Use transient runs on the speedline",
                "  Run the last three speedline points transient.",
                "  picture: images/DEC-010/speedline-stage-1.png — Speedline of stage 1 at design speed",
                "## Files",
                "file list cannot be read: line 2: no heading; run dossier check",
                "## Dead ends",
                "FND-010 · finding · Higher seal pressure on the compressor did not help",
                "  Observation: The leak stayed at 0.2 kg/s with twice the seal pressure.",
                "## Related",
                "LES-010 · lesson · Steady runs overpredict stall margin of the compressor",
                "  Statement: Steady mixing-plane runs converge at mass flows where rotating stall is seen.",
                "  Valid for: Transonic front stages.",
                "  picture: images/LES-010/stall-margin-by-method.svg — Stall margin by method",
                "FND-011 · finding · Seal leak model checked against the rig",
                "FND-013 · finding · Setup notes checked against rig",
            ]
        );
    }

    #[test]
    fn at_most_five_dead_ends_are_listed_and_the_others_go_to_related() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        for number in 1..=6 {
            write_entry(
                &ctx,
                Kind::Finding,
                &format!("FND-{number:03}"),
                &format!("Compressor run variant {number} failed"),
                "approved",
                &[("outcome", "dead end"), ("machine", "axial compressor")],
                &[("Observation", "The run stalled.")],
            );
        }
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        let found = found.expect("the lines");
        let dead = found
            .iter()
            .position(|line| line == "## Dead ends")
            .expect("the Dead ends heading");
        let related = found
            .iter()
            .position(|line| line == "## Related")
            .expect("the Related heading");
        let listed = found[dead + 1..related]
            .iter()
            .filter(|line| line.starts_with("FND-"))
            .count();
        assert_eq!(listed, 5, "{found:?}");
        assert!(
            found[related..]
                .contains(&"FND-006 · finding · Compressor run variant 6 failed".to_string()),
            "{found:?}"
        );
    }

    #[test]
    fn the_todo_files_field_takes_a_string_or_a_list() {
        let text = serde_json::json!({"files": "src\\pump/, ./docs/setup.md ,"});
        assert_eq!(
            todo_files(&text),
            Some(vec!["src/pump".to_string(), "docs/setup.md".to_string()])
        );
        let list = serde_json::json!({"fields": {"files": ["data", " results\\run 1/ "]}});
        assert_eq!(
            todo_files(&list),
            Some(vec!["data".to_string(), "results/run 1".to_string()])
        );
        assert_eq!(todo_files(&serde_json::json!({"id": "T-132"})), None);
        assert_eq!(todo_files(&serde_json::json!({"files": 3})), None);
    }

    #[test]
    fn a_code_pointer_lies_in_a_path_when_it_is_the_path_or_inside_it() {
        let text = "<!-- dossier-format: 1 -->\n# FND-020 · Pointers\n\n- kind: finding\n- status: approved\n- code: src/pump/seal.rs::leak, src/pumps/x.rs, docs\\setup.md\n\n## Observation\n\nText.\n";
        let entry = Entry::parse(text).expect("the entry parses");
        let paths = ["src/pump".to_string(), "docs/setup.md".to_string()];
        assert_eq!(
            pointers_in(&entry, &paths),
            ["src/pump/seal.rs::leak", "docs\\setup.md"]
        );
        assert!(pointers_in(&entry, &["src/pum".to_string()]).is_empty());
    }

    #[test]
    fn a_picture_line_gives_the_path_and_the_caption() {
        assert_eq!(
            picture_line("![Contour, hot spot](../images/FND-004/contour-plot.png)").as_deref(),
            Some("  picture: images/FND-004/contour-plot.png — Contour, hot spot")
        );
        assert_eq!(picture_line("Some other text"), None);
    }
}
