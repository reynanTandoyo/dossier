//! The `hooks` command (docs/design.md section 9, R-AGENT-3, R-CTX-7): installs or removes the
//! Claude Code hooks of the dossier in `.claude/settings.json` of the current folder. The new text
//! comes from `hooks::installed` or `hooks::uninstalled`. A dossier found from the current folder
//! gets a journal line with who and when (D29).

use crate::atomic;
use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::find;
use crate::hooks;
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The name of the copy of the settings file kept before a change, in the same folder.
const BACKUP: &str = "settings.json.bak";

/// The name of the state folder inside the dossier (format.md §9).
const STATE: &str = ".state";

/// The ID of the journal lines of this command (format.md §11).
const ID: &str = "HOOKS";

/// The arguments of `hooks`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `hooks` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds the dossier hooks to .claude/settings.json; without --yes it only says what would change
    Install {
        /// Writes the file
        #[arg(long)]
        yes: bool,
    },
    /// Removes the dossier hooks from .claude/settings.json; without --yes it only says what would change
    Uninstall {
        /// Writes the file
        #[arg(long)]
        yes: bool,
    },
}

/// Which way `hooks` changes the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Install,
    Uninstall,
}

impl Direction {
    /// The result when the settings already are as this direction makes them.
    fn unchanged(self) -> &'static str {
        match self {
            Direction::Install => {
                "the dossier hooks are already in .claude/settings.json: nothing to change"
            }
            Direction::Uninstall => {
                "there are no dossier hooks in .claude/settings.json: nothing to change"
            }
        }
    }

    /// The first line when nothing is written because `--yes` is missing.
    fn planned(self) -> &'static str {
        match self {
            Direction::Install => {
                "would add the dossier hooks to .claude/settings.json; add --yes to write it"
            }
            Direction::Uninstall => {
                "would remove the dossier hooks from .claude/settings.json; add --yes to write it"
            }
        }
    }

    /// The first line after the settings are written.
    fn done(self) -> &'static str {
        match self {
            Direction::Install => "hooks installed in .claude/settings.json",
            Direction::Uninstall => "hooks removed from .claude/settings.json",
        }
    }

    /// The detail of the journal line.
    fn detail(self) -> &'static str {
        match self {
            Direction::Install => "hooks installed",
            Direction::Uninstall => "hooks removed",
        }
    }
}

/// Runs `hooks`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match &args.action {
        Action::Install { yes } => change(Direction::Install, *yes, g),
        Action::Uninstall { yes } => change(Direction::Uninstall, *yes, g),
    }
}

/// Installs or removes the dossier hooks. Nothing is written when the settings already are as
/// the direction makes them, and without `yes` (or with a dry run) nothing is written at all.
/// The journal line comes after the write, and only when a dossier is found.
fn change(direction: Direction, yes: bool, g: &Global) -> Result<Output> {
    let cwd = std::env::current_dir()?;
    let path = cwd.join(".claude").join("settings.json");
    let file = path.display().to_string();
    let existing = read_settings(&path)?;
    let before_text = existing.as_deref().unwrap_or("");
    // The dossier is found before anything is written, so that a refused --dossier changes nothing.
    let state = journal_state(&cwd, g)?;
    let after_text = match direction {
        Direction::Install => hooks::installed(before_text)?,
        Direction::Uninstall => hooks::uninstalled(before_text)?,
    };
    let before = value_of(before_text)?;
    let after = value_of(&after_text)?;
    if before == after {
        return Ok(Output::with(
            direction.unchanged(),
            data(&file, false, false, false, &[], &[]),
        ));
    }

    let before_hooks = hook_lines(&before);
    let after_hooks = hook_lines(&after);
    let added = unmatched(&after_hooks, &before_hooks);
    let removed = unmatched(&before_hooks, &after_hooks);
    let mut lines: Vec<String> = added.iter().map(|line| format!("+ {line}")).collect();
    lines.extend(removed.iter().map(|line| format!("- {line}")));

    if !yes {
        return Ok(Output::with(
            listing(direction.planned(), &lines),
            data(&file, true, false, false, &added, &removed),
        ));
    }
    if g.dry_run {
        let headline = "dry run: would write .claude/settings.json";
        return Ok(Output::with(
            listing(headline, &lines),
            data(&file, true, false, false, &added, &removed),
        ));
    }

    let backup = existing.is_some();
    if backup {
        let kept = path.with_file_name(BACKUP);
        std::fs::copy(&path, &kept).map_err(|error| io_error(&kept, error))?;
    }
    atomic::replace(&path, after_text.as_bytes(), None)?;
    if let Some(state) = state {
        journal::append(&state, &record(direction, g))?;
    }
    let headline = if backup {
        format!(
            "{} (the old file is kept as .claude/{BACKUP})",
            direction.done()
        )
    } else {
        direction.done().to_string()
    };
    Ok(Output::with(
        listing(&headline, &lines),
        data(&file, true, true, backup, &added, &removed),
    ))
}

/// The data of a result: the file, whether its settings differ, whether it was written, whether
/// the old file was kept as a copy, and the hook lines that are added or removed.
fn data(
    file: &str,
    changed: bool,
    written: bool,
    backup: bool,
    added: &[String],
    removed: &[String],
) -> Value {
    json!({
        "file": file,
        "changed": changed,
        "written": written,
        "backup": backup,
        "added": added,
        "removed": removed,
    })
}

/// The headline, then one line per change.
fn listing(headline: &str, lines: &[String]) -> String {
    let mut text = headline.to_string();
    for line in lines {
        text.push('\n');
        text.push_str(line);
    }
    text
}

/// The state folder of the dossier that `hooks` journals to: the dossier found from `cwd`, or the
/// one `--dossier` names. None when no dossier is found; a `--dossier` that is not one is refused.
fn journal_state(cwd: &Path, g: &Global) -> Result<Option<PathBuf>> {
    let explicit = g.dossier.as_ref().map(|path| cwd.join(path));
    match find::find(cwd, explicit.as_deref()) {
        Ok(dossier) => Ok(Some(dossier.join(STATE))),
        Err(error) if error.code == Code::NoDossier && explicit.is_none() => Ok(None),
        Err(error) => Err(error),
    }
}

/// The journal line of a change made by `hooks`, with who and when (D29).
fn record(direction: Direction, g: &Global) -> journal::Record {
    let who = names::unchecked(g.agent.as_deref());
    journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "hooks".to_string(),
        id: ID.to_string(),
        event: "changed".to_string(),
        detail: direction.detail().to_string(),
    }
}

/// The text of the settings file; None when there is no file.
fn read_settings(path: &Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_error(path, error)),
    }
}

/// The settings as a JSON value, so that two texts that differ only in layout count as the same.
/// A byte-order mark is skipped and blank text is an empty object, as in `hooks::installed`.
fn value_of(text: &str) -> Result<Value> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(text).map_err(|error| {
        Error::new(
            Code::Parse,
            format!("the settings are not valid JSON: {error}"),
        )
    })
}

/// The hooks of a settings value, one line each: `SessionStart: dossier brief`, or
/// `PreToolUse (Edit|Write|MultiEdit): dossier guard` when the entry has a matcher.
fn hook_lines(settings: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    let events = settings.get("hooks").and_then(Value::as_object);
    for (event, entries) in events.into_iter().flatten() {
        for entry in entries.as_array().into_iter().flatten() {
            let matcher = entry
                .get("matcher")
                .and_then(Value::as_str)
                .filter(|matcher| !matcher.is_empty());
            let hook_list = entry.get("hooks").and_then(Value::as_array);
            for hook in hook_list.into_iter().flatten() {
                let Some(command) = hook.get("command").and_then(Value::as_str) else {
                    continue;
                };
                let label = match matcher {
                    Some(matcher) => format!("{event} ({matcher})"),
                    None => event.clone(),
                };
                lines.push(format!("{label}: {command}"));
            }
        }
    }
    lines
}

/// The lines of `lines` that `other` does not have. Each line of `other` matches one line only,
/// so a line that occurs twice in `lines` is reported when `other` has it once.
fn unmatched(lines: &[String], other: &[String]) -> Vec<String> {
    let mut rest: Vec<&String> = other.iter().collect();
    let mut missing = Vec::new();
    for line in lines {
        match rest.iter().position(|item| *item == line) {
            Some(index) => {
                rest.swap_remove(index);
            }
            None => missing.push(line.clone()),
        }
    }
    missing
}

/// A failure on `path` as Code::Io, with the path in the message.
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::new(Code::Io, format!("{}: {error}", path.display()))
}
