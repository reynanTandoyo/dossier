//! The lines of `brief --task`: what a session needs to know about one task.

use crate::blockfile::BlockFile;
use crate::config;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::glossary::{self, Term};
use crate::select::{self, Query};
use crate::store::{self, Item};
use crate::teamfiles;
use crate::todo_bridge;
use serde_json::Value;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
/// The heading over the glossary terms whose term or other spelling appears in the task's title,
/// fields or notes (R-TERM-2).
const TERMS: &str = "## Terms";
/// The most lines of the Terms heading; the last one counts the lines left out (R-TERM-2).
const TERMS_LINES: usize = 10;
/// The heading over the last usage records of the task's software and analysis (R-CTX-2).
const EARLIER_USES: &str = "## Earlier uses";
/// The most usage records the Earlier uses heading gives (R-CTX-2).
const USES_LIMIT: usize = 3;

/// The lines of `brief --task`: the task's title and goal lines from todo, the glossary terms that
/// appear in the task (R-TERM-2, the team's marked `team:`), the entries whose `task` field names
/// it, the decisions whose task is one of its milestones, the task's file list (R-FILE-14), the
/// last usage records of its software and analysis in the team's usage index (R-CTX-2), the
/// approved dead ends that match it (R-CTX-16), the entries whose code pointers lie in the files
/// todo names for it (R-CODE-4), then select::best over the remaining approved entries and the
/// team's approved lessons and how-tos (R-CTX-2, marked `team:`), except a copy of a project entry
/// that is listed (R-TEAM-4), with the task's machine, analysis and software fields as tags and
/// its title as words. A decision or lesson printed with its text is followed by its pictures
/// (R-ENTRY-17).
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
    out.extend(terms_lines(ctx, &about));
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
    out.extend(uses_lines(ctx, &about));

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

    let mut pool: Vec<Item> = pool
        .into_iter()
        .filter(|item| !listed.contains(&item.entry.id()))
        .collect();
    // The team's entries join after that check: their IDs can be the same as a project's, so
    // they are told apart by path, not by ID.
    let team = team_items(ctx);
    let team_paths: HashSet<PathBuf> = team.iter().map(|item| item.path.clone()).collect();
    pool.extend(team);
    // A team copy of a listed project entry is left out (R-TEAM-4). Leaving it out can bring
    // another entry into the choice, so the choice is made again without it.
    let related = loop {
        let picked = select::best(&pool, &query, RELATED_LIMIT);
        let left_out = copies_left_out(&picked, &team_paths, &listed, &ctx.cfg.code);
        if left_out.is_empty() {
            break picked;
        }
        pool.retain(|item| !left_out.contains(&item.path));
    };
    if !related.is_empty() {
        out.push(RELATED.to_string());
        for item in related {
            let line = summary(item);
            if team_paths.contains(&item.path) {
                out.push(format!("team:{line}"));
            } else {
                out.push(line);
            }
            out.extend(lesson_lines(item));
        }
    }
    Ok(cut(out, limit))
}

/// The approved lessons and how-tos of the team dossier of the project (R-CTX-2), in the order of
/// `store::all`. A project with no team dossier gives none; tools are left out.
fn team_items(ctx: &Ctx) -> Vec<Item> {
    let Some(team) = &ctx.team else {
        return Vec::new();
    };
    let (items, _warnings) = store::all(team);
    items
        .into_iter()
        .filter(|item| item.entry.status() == Some(Status::Approved))
        .filter(|item| matches!(item.entry.kind(), Some(Kind::Lesson | Kind::Howto)))
        .collect()
}

/// The ID of the project entry that a team entry was copied from: the part after the colon of its
/// `source` field (`<project code>:<ID>`, format.md §23), when the part before the colon is `code`,
/// ignoring case. None when there is no such field or it names another project.
fn promoted_from(item: &Item, code: &str) -> Option<String> {
    let source = item.entry.field("source")?;
    let (from, id) = source.rsplit_once(':')?;
    same(from, code).then(|| id.trim().to_string())
}

/// The team entries among `picked` that are copies of a project entry which is listed: one listed
/// under an earlier heading (`listed`), or one of the project entries picked (R-TEAM-4). Their
/// paths; the project's own entry stays.
fn copies_left_out(
    picked: &[&Item],
    team_paths: &HashSet<PathBuf>,
    listed: &HashSet<String>,
    code: &str,
) -> Vec<PathBuf> {
    let shown: HashSet<String> = picked
        .iter()
        .filter(|item| !team_paths.contains(&item.path))
        .map(|item| item.entry.id())
        .collect();
    picked
        .iter()
        .filter(|item| team_paths.contains(&item.path))
        .filter(|item| {
            promoted_from(item, code).is_some_and(|id| listed.contains(&id) || shown.contains(&id))
        })
        .map(|item| item.path.clone())
        .collect()
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
    /// The text that glossary terms are looked for in (R-TERM-2): the title, the values of the
    /// fields and the text of the notes.
    texts: Vec<String>,
}

impl About {
    fn read(data: &Value) -> About {
        let title = first_text([data.get("title"), inner(data, "title")]);
        let texts = task_texts(data, &title);
        About {
            title,
            machine: first_text([inner(data, "machine"), data.get("machine")]),
            analysis: first_text([inner(data, "analysis"), data.get("analysis")]),
            software: first_text([inner(data, "software"), data.get("software")]),
            milestones: milestone_ids(data),
            files: todo_files(data),
            texts,
        }
    }

    /// The machine, analysis and software values, each split at commas.
    fn tags(&self) -> Vec<String> {
        [&self.machine, &self.analysis, &self.software]
            .into_iter()
            .flat_map(|list| list_of(list))
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

/// The Terms heading (R-TERM-2): the terms of the project's glossary and of the team's whose term
/// or other spelling is in the task's title, fields or notes. The project's come first, each
/// glossary in its file order; the team's are marked `team:`. A glossary that cannot be read gives
/// a line that says so, placed before the terms. A team whose settings cannot be read adds nothing.
/// At most TERMS_LINES lines; no heading when there is nothing to say.
fn terms_lines(ctx: &Ctx, about: &About) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let mut found: Vec<String> = Vec::new();
    match glossary_terms(&ctx.dossier, &ctx.cfg.code) {
        Ok(terms) => found.extend(matching(&terms, &about.texts).into_iter().map(term_line)),
        Err(error) => problems.push(format!("glossary cannot be read: {}", error.message)),
    }
    if let Some(team) = &ctx.team
        && let Some(code) = team_code(team)
    {
        match glossary_terms(team, &code) {
            Ok(terms) => found.extend(
                matching(&terms, &about.texts)
                    .into_iter()
                    .map(|term| format!("team:{}", term_line(term))),
            ),
            Err(error) => problems.push(format!(
                "the team's glossary cannot be read: {}",
                error.message
            )),
        }
    }
    let rows: Vec<String> = problems.into_iter().chain(found).collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let mut out = vec![TERMS.to_string()];
    out.extend(cut(rows, TERMS_LINES));
    out
}

/// The terms of the glossary of `dossier` in file order (format.md §20). `code` names the file
/// when it is missing, and a missing file has no terms.
fn glossary_terms(dossier: &Path, code: &str) -> Result<Vec<Term>> {
    let (file, _) = glossary::load(dossier, code)?;
    Ok(glossary::terms(&file))
}

/// The code of a team dossier, from its settings. None when they cannot be read, so that a team
/// dossier that cannot be read adds nothing (as its lessons and how-tos do).
fn team_code(team: &Path) -> Option<String> {
    config::load(team).ok().map(|cfg| cfg.code)
}

/// The terms whose term or one of whose other spellings is a run of whole words in one of
/// `texts`, ignoring case, in the order of `terms`. So "compress" does not match "compressor", and
/// "Massenstrom" matches the term "mass flow" with that spelling.
fn matching<'a>(terms: &'a [Term], texts: &[String]) -> Vec<&'a Term> {
    let texts: Vec<Vec<String>> = texts.iter().map(|text| words_of(text)).collect();
    terms
        .iter()
        .filter(|term| {
            std::iter::once(term.term.as_str())
                .chain(term.also.iter().map(String::as_str))
                .map(words_of)
                .any(|spelling| texts.iter().any(|words| has_run(words, &spelling)))
        })
        .collect()
}

/// One term as a line, in the form of `glossary`: `<term> (<symbol>, <unit>): <meaning>`, then
/// `  [also: <spellings>]`. The parts a term does not have are left out.
fn term_line(term: &Term) -> String {
    let mut line = term.term.clone();
    let facts: Vec<&str> = [term.symbol.as_deref(), term.unit.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !facts.is_empty() {
        line.push_str(&format!(" ({})", facts.join(", ")));
    }
    if !term.meaning.is_empty() {
        line.push_str(&format!(": {}", term.meaning));
    }
    if !term.also.is_empty() {
        line.push_str(&format!("  [also: {}]", term.also.join(", ")));
    }
    line
}

/// The words of `text` in lower case, split at every character that is not a letter or a digit,
/// in their order: "m-dot" is "m" and "dot".
fn words_of(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

/// Whether the words of `needle` follow each other, in order, among `words`. An empty `needle` is
/// never found.
fn has_run(words: &[String], needle: &[String]) -> bool {
    !needle.is_empty() && words.windows(needle.len()).any(|window| window == needle)
}

/// The text that glossary terms are looked for in (R-TERM-2): the title, each value of the fields
/// object, and the text of each note. Notes are read from `notes` at the top of the data or under
/// `fields`, as a list or as one note.
fn task_texts(data: &Value, title: &str) -> Vec<String> {
    let mut texts = vec![title.to_string()];
    if let Some(Value::Object(fields)) = data.get("fields") {
        texts.extend(fields.values().flat_map(text_values));
    }
    let notes = [data.get("notes"), inner(data, "notes")]
        .into_iter()
        .flatten()
        .find(|notes| !notes.is_null());
    match notes {
        Some(Value::Array(items)) => texts.extend(items.iter().flat_map(note_texts)),
        Some(note) => texts.extend(note_texts(note)),
        None => {}
    }
    texts
}

/// The text of one note: the `text` of a note that is an object, else the note itself.
fn note_texts(note: &Value) -> Vec<String> {
    match note {
        Value::Object(_) => note.get("text").map_or_else(Vec::new, text_values),
        other => text_values(other),
    }
}

/// The text of a value as plain strings: a string, a number or a boolean as its text, and a list as
/// the text of each of its items. An object and null give none.
fn text_values(value: &Value) -> Vec<String> {
    match value {
        Value::String(text) => vec![text.clone()],
        Value::Number(_) | Value::Bool(_) => vec![value.to_string()],
        Value::Array(items) => items.iter().flat_map(text_values).collect(),
        Value::Null | Value::Object(_) => Vec::new(),
    }
}

/// The Earlier uses heading (R-CTX-2): the last USES_LIMIT usage records of the team's usage index
/// whose software is one of the task's software and whose analysis shares a word with the task's
/// analysis (any analysis when the task names none), newest date first. Of two records on one
/// date, the one written later comes first. No heading without a team dossier whose settings can
/// be read, without software in the task, or without a matching record. A usage index that cannot
/// be read gives the heading and one line that says so.
fn uses_lines(ctx: &Ctx, about: &About) -> Vec<String> {
    let Some(team) = &ctx.team else {
        return Vec::new();
    };
    let Some(code) = team_code(team) else {
        return Vec::new();
    };
    let software = list_of(&about.software);
    if software.is_empty() {
        return Vec::new();
    }
    let analysis = list_of(&about.analysis);
    let file = match teamfiles::usage(team, &code) {
        Ok((file, _)) => file,
        Err(error) => {
            return vec![
                EARLIER_USES.to_string(),
                format!("the team's usage index cannot be read: {}", error.message),
            ];
        }
    };
    let value = |key: &str, name: &str| file.field(key, name).unwrap_or_default();
    // Later blocks first: the sort below is stable, so records of one date keep this order.
    let mut keys: Vec<String> = file.keys().into_iter().rev().collect();
    keys.retain(|key| {
        names_any(&software, &value(key, "software"))
            && (analysis.is_empty() || names_any(&analysis, &value(key, "analysis")))
    });
    keys.sort_by_key(|key| Reverse(value(key, "date")));
    if keys.is_empty() {
        return Vec::new();
    }
    let mut out = vec![EARLIER_USES.to_string()];
    out.extend(keys.iter().take(USES_LIMIT).map(|key| use_line(&file, key)));
    out
}

/// Whether one of the words of the comma-separated `value` is one of `wanted`, ignoring case.
fn names_any(wanted: &[String], value: &str) -> bool {
    list_of(value)
        .iter()
        .any(|word| wanted.iter().any(|name| same(word, name)))
}

/// One usage record as a line: its date, its key, its software and version, its analysis, its
/// machine, its how-to and its person, joined with ` · `. A part the block does not have is left
/// out.
fn use_line(file: &BlockFile, key: &str) -> String {
    let value = |name: &str| file.field(key, name).unwrap_or_default().trim().to_string();
    let software = [value("software"), value("version")]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<String>>()
        .join(" ");
    let howto = value("howto");
    let howto = if howto.is_empty() {
        String::new()
    } else {
        format!("how-to {howto}")
    };
    [
        value("date"),
        key.to_string(),
        software,
        value("analysis"),
        value("machine"),
        howto,
        value("person"),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<String>>()
    .join(" · ")
}

/// The values of a comma-separated list, trimmed, without the empty ones.
fn list_of(list: &str) -> Vec<String> {
    list.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
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
            team: None,
            from_document: None,
            project: None,
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

    /// A team dossier in `TEAM/DOSSIER` beside the project's folder: kind = "team" and the folders
    /// of a team. Returns a context of it, for writing its entries; the project context's `team`
    /// names its path.
    fn team_of(project: &Ctx) -> Ctx {
        let root = project.root.join("TEAM");
        let dossier = root.join("DOSSIER");
        for folder in ["lessons", "howtos", "tools"] {
            fs::create_dir_all(dossier.join(folder)).expect("a team folder");
        }
        let text = format!("kind = \"team\"\n{}", config::default_text("TEAM"));
        fs::write(dossier.join("dossier.toml"), text).expect("the team's settings");
        Ctx {
            state: dossier.join(".state"),
            cfg: config::load(&dossier).expect("the team's settings load"),
            root,
            dossier,
            who: None,
            json: false,
            dry_run: false,
            command: "brief".to_string(),
            team: None,
            from_document: None,
            project: None,
        }
    }

    #[test]
    fn related_gives_the_team_lessons_and_how_tos_marked_team() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        // The project's LES-010 and the team's LES-010 have the same ID. Scores: the project's 8
        // (title and machine), the team lesson's 6 (machine and analysis), the how-to's 5 (title).
        write_entry(
            &ctx,
            Kind::Lesson,
            "LES-010",
            "Compressor stall notes",
            "approved",
            &[
                ("machine", "axial compressor"),
                ("confidence", "experience"),
            ],
            &[
                ("Statement", "Stall came early at low mass flow."),
                ("Valid for", "Front stages only."),
            ],
        );
        write_entry(
            &team,
            Kind::Lesson,
            "LES-010",
            "Steady runs overpredict stall margin",
            "approved",
            &[
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                ("confidence", "simulated"),
            ],
            &[
                (
                    "Statement",
                    "Steady runs converge where rotating stall is seen.",
                ),
                ("Valid for", "Transonic front stages."),
            ],
        );
        write_entry(
            &team,
            Kind::Howto,
            "HOW-001",
            "Set up speedline runs",
            "approved",
            &[("software", "ExampleSolver")],
            &[
                ("Purpose", "Runs one point of a curve."),
                ("Steps", "Set mass flow and wait."),
            ],
        );
        // Left out although each would match: a draft lesson, a retired how-to and a tool.
        write_entry(
            &team,
            Kind::Lesson,
            "LES-011",
            "Speedline lesson draft",
            "draft",
            &[("machine", "axial compressor")],
            &[
                ("Statement", "Never listed."),
                ("Valid for", "Never listed."),
            ],
        );
        write_entry(
            &team,
            Kind::Howto,
            "HOW-002",
            "Old speedline setup",
            "retired",
            &[("software", "ExampleSolver")],
            &[("Purpose", "Never listed.")],
        );
        write_entry(
            &team,
            Kind::Tool,
            "TOOL-001",
            "Speedline post-processing script",
            "approved",
            &[("machine", "axial compressor")],
            &[("Purpose", "Never listed.")],
        );
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Related",
                "LES-010 · lesson · Compressor stall notes",
                "  Statement: Stall came early at low mass flow.",
                "  Valid for: Front stages only.",
                "team:LES-010 · lesson · Steady runs overpredict stall margin",
                "  Statement: Steady runs converge where rotating stall is seen.",
                "  Valid for: Transonic front stages.",
                "team:HOW-001 · howto · Set up speedline runs",
            ]
        );
    }

    #[test]
    fn a_team_dossier_that_cannot_be_read_adds_no_team_line() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        ctx.team = Some(dir.path().join("NOWHERE").join("DOSSIER"));
        write_entry(
            &ctx,
            Kind::Lesson,
            "LES-010",
            "Compressor stall notes",
            "approved",
            &[("machine", "axial compressor")],
            &[
                ("Statement", "Stall came early."),
                ("Valid for", "Front stages only."),
            ],
        );
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Related",
                "LES-010 · lesson · Compressor stall notes",
                "  Statement: Stall came early.",
                "  Valid for: Front stages only.",
            ]
        );
    }

    #[test]
    fn a_promoted_lesson_is_listed_once_when_its_project_entry_is_picked() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        let title = "Steady mixing-plane runs overpredict stall margin";
        let sections = [
            (
                "Statement",
                "Steady runs converge where rotating stall is seen.",
            ),
            ("Valid for", "Transonic front stages."),
        ];
        // The project's LES-001 and the team's copy of it, with source P-001:LES-001. Their tags
        // match alike, so both are picked for Related.
        let fields = [
            ("machine", "axial compressor"),
            ("analysis", "steady RANS, speedline"),
            ("confidence", "simulated"),
        ];
        write_entry(
            &ctx,
            Kind::Lesson,
            "LES-001",
            title,
            "approved",
            &fields,
            &sections,
        );
        let mut copied = fields.to_vec();
        copied.push(("source", "P-001:LES-001"));
        write_entry(
            &team,
            Kind::Lesson,
            "LES-001",
            title,
            "approved",
            &copied,
            &sections,
        );
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        // The project's copy is the one line; the team's copy is left out.
        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Related",
                "LES-001 · lesson · Steady mixing-plane runs overpredict stall margin",
                "  Statement: Steady runs converge where rotating stall is seen.",
                "  Valid for: Transonic front stages.",
            ]
        );
    }

    #[test]
    fn a_copy_is_left_out_when_its_project_entry_is_listed_above() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        let title = "Steady mixing-plane runs overpredict stall margin";
        let sections = [
            (
                "Statement",
                "Steady runs converge where rotating stall is seen.",
            ),
            ("Valid for", "Transonic front stages."),
        ];
        // The project's LES-001 is listed under Recorded. Its team copy names the project in lower
        // case, which still counts. The team's LES-002 is a copy of another project's LES-001, so
        // it stays.
        write_entry(
            &ctx,
            Kind::Lesson,
            "LES-001",
            title,
            "approved",
            &[
                ("task", "T-132"),
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                ("confidence", "simulated"),
            ],
            &sections,
        );
        write_entry(
            &team,
            Kind::Lesson,
            "LES-001",
            title,
            "approved",
            &[
                ("machine", "axial compressor"),
                ("analysis", "steady RANS, speedline"),
                ("confidence", "simulated"),
                ("source", "p-001:LES-001"),
            ],
            &sections,
        );
        write_entry(
            &team,
            Kind::Lesson,
            "LES-002",
            "Steady runs converge at the stability limit",
            "approved",
            &[
                ("machine", "axial compressor"),
                ("analysis", "steady RANS"),
                ("source", "P-002:LES-001"),
            ],
            &sections,
        );
        let show = stand_in(dir.path(), "todo-show", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Recorded for this task",
                "LES-001 · lesson · Steady mixing-plane runs overpredict stall margin",
                "## Related",
                "team:LES-002 · lesson · Steady runs converge at the stability limit",
                "  Statement: Steady runs converge where rotating stall is seen.",
                "  Valid for: Transonic front stages.",
            ]
        );
    }

    /// Writes glossary.md into `dossier` with `blocks` after its title (format.md §20).
    fn write_glossary(dossier: &Path, code: &str, blocks: &str) {
        let text = format!("<!-- dossier-format: 1 -->\n# Glossary of {code}\n\n{blocks}");
        fs::write(dossier.join("glossary.md"), text).expect("the glossary");
    }

    /// Writes usage-index.md into the team dossier `dossier` with `blocks` after its title.
    fn write_usage(dossier: &Path, blocks: &str) {
        let text = format!("<!-- dossier-format: 1 -->\n# Usage of TEAM\n\n{blocks}");
        fs::write(dossier.join("usage-index.md"), text).expect("the usage index");
    }

    #[test]
    fn terms_gives_the_terms_of_both_glossaries_that_the_task_names_as_whole_words() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        write_glossary(
            &ctx.dossier,
            "P-001",
            "## speedline\n\n- meaning: the curve of pressure ratio against mass flow\n\n\
             ## mass flow\n\n- meaning: fluid through a section per second\n- symbol: m-dot\n\
             - unit: kg/s\n- also: Massenstrom, mdot\n\n\
             ## compress\n\n- meaning: to press a gas into a smaller volume\n",
        );
        write_glossary(
            &team.dossier,
            "TEAM",
            "## pressure ratio\n\n- meaning: outlet pressure divided by inlet pressure\n\
             - also: Druckverhältnis\n\n\
             ## flow\n\n- meaning: a stream of a fluid\n",
        );
        let mut data = speedline_task();
        data["fields"]["software"] = serde_json::json!("ExampleSolver");
        data["fields"]["remark"] = serde_json::json!("Massenstrom at the inlet");
        data["notes"] = serde_json::json!([
            {"date": "2026-11-21", "agent": "anna-claude", "text": "Check the pressure ratio at the outlet"}
        ]);
        let show = stand_in(dir.path(), "todo-show", &envelope(data));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        // The title has "Speedline", the German spelling is in a field, the team's term is in a
        // note. "compress" is not a whole word of "compressor" and "flow" is not in the task.
        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Terms",
                "speedline: the curve of pressure ratio against mass flow",
                "mass flow (m-dot, kg/s): fluid through a section per second  [also: Massenstrom, mdot]",
                "team:pressure ratio: outlet pressure divided by inlet pressure  [also: Druckverhältnis]",
            ]
        );
    }

    #[test]
    fn terms_give_at_most_ten_lines_and_count_the_ones_left_out() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, ctx) = project();
        let blocks: String = (1..=12)
            .map(|number| format!("## item {number:02}\n\n- meaning: meaning {number:02}\n\n"))
            .collect();
        write_glossary(&ctx.dossier, "P-001", &blocks);
        let names: Vec<String> = (1..=12).map(|number| format!("item {number:02}")).collect();
        let mut data = speedline_task();
        data["fields"]["items"] = serde_json::json!(names.join(", "));
        let show = stand_in(dir.path(), "todo-show", &envelope(data));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        // Twelve terms match: nine are kept and the tenth line counts the three left out.
        let mut expected = vec![
            "Task T-132 · Speedline of the compressor".to_string(),
            "## Terms".to_string(),
        ];
        expected.extend((1..=9).map(|number| format!("item {number:02}: meaning {number:02}")));
        expected.push("3 more left out".to_string());
        assert_eq!(found.expect("the lines"), expected);
    }

    /// The usage index of the team of the tests (invented), in the order the blocks were written.
    /// Matching the software ExampleSolver and the analysis steady RANS are blocks 1, 2, 5, 6, 7
    /// and 8; block 3 has another analysis and block 4 another software. 5 and 6 are on one date.
    const USAGE_BLOCKS: &str = "## P-001:USE-001\n\n- date: 2026-10-01\n- code: P-001\n- software: ExampleSolver\n- version: 2312\n- analysis: steady RANS\n- machine: axial compressor\n- person: anna\n- history: 2026-10-01T10:00:00+01:00 anna: used\n\n\
        ## P-002:USE-004\n\n- date: 2026-11-02\n- code: P-002\n- software: ExampleSolver\n- version: 2406\n- analysis: steady RANS, speedline\n- howto: HOW-002\n- person: ben\n- history: 2026-11-02T08:00:00+01:00 ben: used\n\n\
        ## P-002:USE-005\n\n- date: 2026-11-03\n- code: P-002\n- software: ExampleSolver\n- version: 2406\n- analysis: transient URANS\n- person: ben\n- history: 2026-11-03T08:00:00+01:00 ben: used\n\n\
        ## P-003:USE-002\n\n- date: 2026-11-10\n- code: P-003\n- software: OtherSolver\n- version: 1.0\n- analysis: steady RANS\n- person: lena\n- history: 2026-11-10T08:00:00+01:00 lena: used\n\n\
        ## P-001:USE-006\n\n- date: 2026-11-05\n- code: P-001\n- software: examplesolver\n- version: 2406\n- analysis: Steady rans\n- person: anna\n- history: 2026-11-05T08:00:00+01:00 anna: used\n\n\
        ## P-002:USE-007\n\n- date: 2026-11-05\n- code: P-002\n- software: ExampleSolver\n- version: 2406\n- analysis: steady RANS\n- machine: radial turbine\n- person: ben\n- history: 2026-11-05T09:00:00+01:00 ben: used\n\n\
        ## P-003:USE-001\n\n- date: 2026-11-20\n- code: P-003\n- software: ExampleSolver\n- version: 2406\n- analysis: steady RANS\n- howto: HOW-002\n- person: anna\n- history: 2026-11-20T08:00:00+01:00 anna: used\n\n\
        ## P-001:USE-002\n\n- date: 2026-09-01\n- code: P-001\n- software: ExampleSolver\n- version: 2312\n- analysis: steady RANS\n- person: anna\n- history: 2026-09-01T08:00:00+01:00 anna: used\n";

    #[test]
    fn earlier_uses_give_the_last_three_uses_of_the_software_and_analysis_newest_first() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        write_usage(&team.dossier, USAGE_BLOCKS);
        let mut data = speedline_task();
        data["fields"]["software"] = serde_json::json!("ExampleSolver");
        let show = stand_in(dir.path(), "todo-show", &envelope(data));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // A task that names no software has no earlier uses.
        let bare = stand_in(dir.path(), "todo-bare", &envelope(speedline_task()));
        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::set_var("DOSSIER_TODO", &bare) };
        let no_software = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        assert_eq!(
            found.expect("the lines"),
            vec![
                "Task T-132 · Speedline of the compressor",
                "## Earlier uses",
                "2026-11-20 · P-003:USE-001 · ExampleSolver 2406 · steady RANS · how-to HOW-002 · anna",
                "2026-11-05 · P-002:USE-007 · ExampleSolver 2406 · steady RANS · radial turbine · ben",
                "2026-11-05 · P-001:USE-006 · examplesolver 2406 · Steady rans · anna",
            ]
        );
        assert_eq!(
            no_software.expect("the lines"),
            vec!["Task T-132 · Speedline of the compressor"]
        );
    }

    #[test]
    fn a_glossary_or_usage_index_that_cannot_be_read_is_reported_and_the_other_parts_go_on() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (dir, mut ctx) = project();
        let team = team_of(&ctx);
        ctx.team = Some(team.dossier.clone());
        fs::write(
            ctx.dossier.join("glossary.md"),
            "<!-- dossier-format: 1 -->\nno heading\n",
        )
        .expect("the broken glossary");
        let usage_path = team.dossier.join("usage-index.md");
        fs::write(&usage_path, "<!-- dossier-format: 1 -->\nno heading\n")
            .expect("the broken usage index");
        let mut data = speedline_task();
        data["fields"]["software"] = serde_json::json!("ExampleSolver");
        let show = stand_in(dir.path(), "todo-show", &envelope(data));
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment meanwhile.
        unsafe { std::env::set_var("DOSSIER_TODO", &show) };

        let found = lines(&ctx, "T-132", 100);

        // SAFETY: ENV_LOCK is still held.
        unsafe { std::env::remove_var("DOSSIER_TODO") };

        let found = found.expect("brief --task goes on");
        assert_eq!(found.len(), 5, "{found:?}");
        assert_eq!(found[1], "## Terms");
        assert_eq!(
            found[2],
            "glossary cannot be read: glossary.md: line 2: no heading"
        );
        assert_eq!(found[3], "## Earlier uses");
        assert_eq!(
            found[4],
            format!(
                "the team's usage index cannot be read: {}: line 2: no heading",
                usage_path.display()
            )
        );
    }

    #[test]
    fn words_are_split_at_each_character_that_is_not_a_letter_or_a_digit() {
        assert_eq!(
            words_of("Mass-Flow, m-dot  kg/s"),
            ["mass", "flow", "m", "dot", "kg", "s"]
        );
        let words = words_of("Massenstrom at the inlet");
        assert!(has_run(&words, &words_of("at the")));
        assert!(!has_run(&words, &words_of("inlet at")));
        assert!(!has_run(&words, &words_of("")));
        assert!(!has_run(&words_of("compressor"), &words_of("compress")));
    }
}
