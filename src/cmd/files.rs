//! The `files` command (docs/design.md section 9, R-FILE-11, R-LIFE-9): the file lists of the
//! tasks with their marks. With `--task` the list of one task, grouped by role in the order of
//! `filelist::ROLES` and then the other roles; without it every list, each line with its task.
//! `--role` and words narrow the lines. Detached files are left out unless `--all`. With `--on`
//! the lists are shown as they were on that day, without marks.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::config::Config;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::output::Output;
use crate::time;
use chrono::NaiveDate;
use serde_json::json;
use std::path::Path;

/// The arguments of `files`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The task, such as T-042: only its list, grouped by role
    #[arg(long, value_name = "task")]
    pub task: Option<String>,
    /// Only the files of this role, such as model or result
    #[arg(long, value_name = "role")]
    pub role: Option<String>,
    /// Only the files whose path or note contains each word, ignoring case
    pub words: Vec<String>,
    /// The day, YYYY-MM-DD: the lists as they were on that day, without marks (R-LIFE-9)
    #[arg(long, value_name = "date")]
    pub on: Option<String>,
    /// Also the detached files
    #[arg(long)]
    pub all: bool,
}

/// What a file of a list must hold to be shown.
struct Choice {
    /// The role, in lower case
    role: Option<String>,
    /// The words, in lower case: each must be in the path or in the note
    words: Vec<String>,
    /// The day of `--on`, YYYY-MM-DD
    on: Option<String>,
    /// Whether detached files are shown (without `--on`)
    all: bool,
}

/// One file of a list, as the command shows it.
struct Line {
    task: String,
    path: String,
    role: Option<String>,
    note: Option<String>,
    modified: Option<String>,
    /// The marks in order; None with `--on`, where marks are not shown.
    marks: Option<Vec<String>>,
}

/// Runs `files`: the lines of the lists that the arguments choose, as text and as data. A list
/// that cannot be read is a warning when every list is shown, and an error when one task is asked
/// for.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "files")?;
    let task = args.task.as_deref().map(task_of).transpose()?;
    let choice = Choice {
        role: args
            .role
            .as_deref()
            .map(|role| role_of(&ctx, role))
            .transpose()?,
        words: args
            .words
            .iter()
            .map(|word| word.trim().to_lowercase())
            .filter(|word| !word.is_empty())
            .collect(),
        on: args.on.as_deref().map(day_of).transpose()?,
        all: args.all,
    };

    let names = match &task {
        Some(task) => vec![task.clone()],
        None => filelist::tasks(&ctx.dossier),
    };
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    for name in &names {
        match filelist::load(&ctx.dossier, name) {
            Ok((file, _)) => lines.extend(list_lines(&ctx, name, &file, &choice)),
            Err(error) if task.is_some() => return Err(error),
            Err(error) => warnings.push(format!(
                "{}: {}",
                shown_path(&ctx.root, &filelist::path(&ctx.dossier, name)),
                error.message
            )),
        }
    }
    if task.is_some() {
        lines.sort_by_key(|line| role_rank(&ctx.cfg, line.role.as_deref()));
    }

    let text = render(task.as_deref(), &lines);
    let rows: Vec<serde_json::Value> = lines.iter().map(row).collect();
    let output = Output::with(text, json!(rows));
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// The lines of one list that the choice holds, in the order of the list.
fn list_lines(ctx: &Ctx, task: &str, file: &BlockFile, choice: &Choice) -> Vec<Line> {
    let mut keys: Vec<String> = Vec::new();
    for key in file.keys() {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys.into_iter()
        .filter(|key| is_listed(file, key, choice))
        .map(|key| Line {
            task: task.to_string(),
            role: non_empty(file.field(&key, "role")),
            note: non_empty(file.field(&key, "note")),
            modified: non_empty(file.field(&key, "modified")),
            marks: choice
                .on
                .is_none()
                .then(|| filelist::marks(&ctx.root, file, &key)),
            path: key,
        })
        .filter(|line| chosen(line, choice))
        .collect()
}

/// Whether a block is in the list the command shows. Without `--on`: the block is attached now,
/// or it is detached and `--all` is given. With `--on`: its first history line, the attach, is on
/// or before the day, and it was not detached on or before the day (R-LIFE-9).
fn is_listed(file: &BlockFile, key: &str, choice: &Choice) -> bool {
    match choice.on.as_deref() {
        Some(day) => attached_by(file, key, day) && !detached_by(file, key, day),
        None => choice.all || !is_detached(file, key),
    }
}

/// Whether the block was attached on or before the day: the date of its first history line,
/// which holds the time of the attach (format.md §6).
fn attached_by(file: &BlockFile, key: &str, day: &str) -> bool {
    file.history(key).first().is_some_and(|line| {
        let stamp: String = line.chars().take(25).collect();
        time::valid(&stamp) && time::date(&stamp) <= day
    })
}

/// Whether the block was detached on or before the day: the date of its `detached` field.
fn detached_by(file: &BlockFile, key: &str, day: &str) -> bool {
    file.field(key, "detached")
        .filter(|at| !at.is_empty())
        .is_some_and(|at| time::date(&at) <= day)
}

/// Whether the block has a `detached` field, which holds the time of the detach.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    file.field(key, "detached").is_some_and(|at| !at.is_empty())
}

/// Whether a line holds the role and the words that the choice asks for.
fn chosen(line: &Line, choice: &Choice) -> bool {
    let role = line.role.as_deref().unwrap_or_default().to_lowercase();
    let path = line.path.to_lowercase();
    let note = line.note.as_deref().unwrap_or_default().to_lowercase();
    choice.role.as_deref().is_none_or(|wanted| role == wanted)
        && choice
            .words
            .iter()
            .all(|word| path.contains(word.as_str()) || note.contains(word.as_str()))
}

/// The text of the lines. With a task they are grouped by role, each group under its role (or
/// "(no role)"), and each line is indented. Without one, each line starts with its task. With no
/// line at all, "no files", after the task when there is one.
fn render(task: Option<&str>, lines: &[Line]) -> String {
    if lines.is_empty() {
        return match task {
            Some(task) => format!("{task}: no files"),
            None => "no files".to_string(),
        };
    }
    if task.is_none() {
        return lines
            .iter()
            .map(|line| row_text(line, true))
            .collect::<Vec<String>>()
            .join("\n");
    }
    let mut text: Vec<String> = Vec::new();
    let mut group: Option<String> = None;
    for line in lines {
        let key = line.role.as_deref().unwrap_or_default().to_lowercase();
        if group.as_ref() != Some(&key) {
            text.push(line.role.clone().unwrap_or_else(|| "(no role)".to_string()));
            group = Some(key);
        }
        text.push(format!("  {}", row_text(line, false)));
    }
    text.join("\n")
}

/// A line as text: the task first when asked, then the path, the note, the date of the
/// modification and the marks in brackets. The parts that a file does not have are left out.
fn row_text(line: &Line, with_task: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    if with_task {
        parts.push(line.task.clone());
    }
    parts.push(line.path.clone());
    parts.extend(line.note.clone());
    parts.extend(
        line.modified
            .as_deref()
            .map(|stamp| time::date(stamp).to_string())
            .filter(|day| !day.is_empty()),
    );
    if let Some(marks) = line.marks.as_ref().filter(|marks| !marks.is_empty()) {
        parts.push(format!("[{}]", marks.join(", ")));
    }
    parts.join("  ")
}

/// A line as data: {task, path, role, note, modified, marks}. A value the file does not have, and
/// the marks with `--on`, are `null`.
fn row(line: &Line) -> serde_json::Value {
    json!({
        "task": line.task,
        "path": line.path,
        "role": line.role,
        "note": line.note,
        "modified": line.modified,
        "marks": line.marks,
    })
}

/// The place of a role in the order of the lines of a task: the roles of `filelist::ROLES`, then
/// the roles of the settings, then the other roles by name. A line without a role comes last.
fn role_rank(cfg: &Config, role: Option<&str>) -> (usize, String) {
    let Some(role) = role else {
        return (usize::MAX, String::new());
    };
    let lower = role.to_lowercase();
    let place = filelist::ROLES
        .iter()
        .map(|known| known.to_string())
        .chain(cfg.roles.iter().cloned())
        .position(|known| known.to_lowercase() == lower)
        .unwrap_or(filelist::ROLES.len() + cfg.roles.len());
    (place, lower)
}

/// A text that is empty is no value.
fn non_empty(text: Option<String>) -> Option<String> {
    text.filter(|text| !text.is_empty())
}

/// The task as the list names it: upper-cased. A task is letters, digits, `-` and `_`, since its
/// name is a file name (Code::BadValue otherwise).
fn task_of(task: &str) -> Result<String> {
    let task = task.trim();
    if task.is_empty()
        || !task
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(bad(format!("a task is written like T-042: {task}")));
    }
    Ok(task.to_uppercase())
}

/// The role in lower case, as the choice compares it. Code::BadValue for a role that is neither
/// one of the roles nor in the settings (R-FILE-9).
fn role_of(ctx: &Ctx, role: &str) -> Result<String> {
    let role = role.trim();
    if !filelist::valid_role(&ctx.cfg, role) {
        return Err(bad(format!(
            "{role} is not a role; the roles are: {}",
            roles_text(ctx)
        )));
    }
    Ok(role.to_lowercase())
}

/// The roles of the tool and of the settings, separated by commas.
fn roles_text(ctx: &Ctx) -> String {
    filelist::ROLES
        .iter()
        .map(|role| role.to_string())
        .chain(ctx.cfg.roles.iter().cloned())
        .collect::<Vec<String>>()
        .join(", ")
}

/// The day of `--on`, as YYYY-MM-DD; a time stamp counts by its date. Code::BadValue when it is
/// not a date.
fn day_of(text: &str) -> Result<String> {
    let day = time::date(text.trim());
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .map(|date| date.format("%Y-%m-%d").to_string())
        .map_err(|_| {
            bad(format!(
                "--on needs a date such as 2026-11-21, not \"{text}\""
            ))
        })
}

/// A path as people see it: relative to the project folder, with `/` as the separator.
fn shown_path(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
