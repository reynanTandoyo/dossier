//! The `hooks` command (docs/design.md section 9, R-AGENT-3, R-CTX-7, D34): installs or removes the
//! Claude Code hooks of the dossier in `.claude/settings.json` of the current folder, and the name
//! of the LLM that Claude's writes are for in `.claude/settings.local.json` (`DOSSIER_AGENT`). The
//! new texts come from `hooks::installed` or `hooks::uninstalled`, and from `hooks::local_installed`
//! or `hooks::local_uninstalled`. A dossier found from the current folder gets a journal line with
//! who and when (D29).

use crate::atomic;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::find;
use crate::hooks;
use crate::journal;
use crate::names;
use crate::output::Output;
use crate::time;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The settings file with the hooks, relative to the folder `hooks` runs in.
const HOOKS_FILE: &str = ".claude/settings.json";

/// The personal settings file with the variable that names the writer (D34), relative to the
/// folder `hooks` runs in.
const LOCAL_FILE: &str = ".claude/settings.local.json";

/// The variable of the local settings that names the writer of Claude's writes (D34).
const AGENT_VAR: &str = "DOSSIER_AGENT";

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
    /// Adds the dossier hooks to .claude/settings.json and sets DOSSIER_AGENT in
    /// .claude/settings.local.json; without --yes it only says what would change
    Install {
        /// Writes the files
        #[arg(long)]
        yes: bool,
    },
    /// Removes the dossier hooks from .claude/settings.json and DOSSIER_AGENT from
    /// .claude/settings.local.json; without --yes it only says what would change
    Uninstall {
        /// Writes the files
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
    /// The words for the hooks file and for the local file.
    fn words(self) -> (Words, Words) {
        match self {
            Direction::Install => (INSTALL_HOOKS, INSTALL_AGENT),
            Direction::Uninstall => (UNINSTALL_HOOKS, UNINSTALL_AGENT),
        }
    }
}

/// The words that `hooks` uses for one settings file: the phrase for a change that would be made,
/// the phrase for a change that was made, the journal detail, and the phrase for a file that
/// already is as wanted.
#[derive(Debug, Clone, Copy)]
struct Words {
    would: &'static str,
    done: &'static str,
    detail: &'static str,
    kept: &'static str,
}

const INSTALL_HOOKS: Words = Words {
    would: "add the dossier hooks to .claude/settings.json",
    done: "hooks installed in .claude/settings.json",
    detail: "hooks installed",
    kept: "the dossier hooks are in .claude/settings.json",
};

const INSTALL_AGENT: Words = Words {
    would: "set DOSSIER_AGENT in .claude/settings.local.json",
    done: "DOSSIER_AGENT set in .claude/settings.local.json",
    detail: "DOSSIER_AGENT set",
    kept: "DOSSIER_AGENT is set in .claude/settings.local.json",
};

const UNINSTALL_HOOKS: Words = Words {
    would: "remove the dossier hooks from .claude/settings.json",
    done: "hooks removed from .claude/settings.json",
    detail: "hooks removed",
    kept: "there are no dossier hooks in .claude/settings.json",
};

const UNINSTALL_AGENT: Words = Words {
    would: "remove DOSSIER_AGENT from .claude/settings.local.json",
    done: "DOSSIER_AGENT removed from .claude/settings.local.json",
    detail: "DOSSIER_AGENT removed",
    kept: "there is no DOSSIER_AGENT in .claude/settings.local.json",
};

/// One settings file that `hooks` may change: its text before and after, the lines that the change
/// adds and removes, and what was done with the file.
struct Change {
    /// The file, as the output names it, relative to the folder `hooks` runs in.
    file: &'static str,
    words: Words,
    path: PathBuf,
    /// The copy kept before a write: the same path with `.bak` added.
    kept: PathBuf,
    /// The text before the change; None when there is no file.
    before: Option<String>,
    /// The text to write when the file changes.
    after: String,
    /// Whether the settings differ, so that the file would be written.
    changed: bool,
    added: Vec<String>,
    removed: Vec<String>,
    /// Whether the file was written.
    written: bool,
    /// Whether the old text was kept as a copy first.
    backup: bool,
}

impl Change {
    /// Plans the change of the file at `path`: `after` is the text the file should have, and
    /// `lines` gives the lines that are compared (the hook lines, or the variable line).
    fn plan(
        path: PathBuf,
        file: &'static str,
        words: Words,
        before: Option<String>,
        after: String,
        lines: fn(&Value) -> Vec<String>,
    ) -> Result<Change> {
        let old = value_of(before.as_deref().unwrap_or(""))?;
        let new = value_of(&after)?;
        let old_lines = lines(&old);
        let new_lines = lines(&new);
        let kept = backup_of(&path);
        Ok(Change {
            file,
            words,
            changed: old != new,
            added: unmatched(&new_lines, &old_lines),
            removed: unmatched(&old_lines, &new_lines),
            path,
            kept,
            before,
            after,
            written: false,
            backup: false,
        })
    }

    /// Writes the new text. The old file is first copied to `kept` when there was one.
    fn write(&mut self) -> Result<()> {
        if self.before.is_some() {
            std::fs::copy(&self.path, &self.kept).map_err(|error| io_error(&self.kept, error))?;
            self.backup = true;
        }
        atomic::replace(&self.path, self.after.as_bytes(), None)?;
        self.written = true;
        Ok(())
    }
}

/// Runs `hooks`.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match &args.action {
        Action::Install { yes } => change(Direction::Install, *yes, g),
        Action::Uninstall { yes } => change(Direction::Uninstall, *yes, g),
    }
}

/// Installs or removes the dossier hooks and the name of the writer. A file whose settings already
/// are as the direction makes them is not written, and without `yes` (or with a dry run) nothing is
/// written at all. The journal line comes after the writes, and only when a dossier is found and
/// something changed.
fn change(direction: Direction, yes: bool, g: &Global) -> Result<Output> {
    let cwd = std::env::current_dir()?;
    // The dossier is found before anything is written, so that a refused --dossier changes nothing.
    let state = journal_state(&cwd, g)?;
    let (hooks_words, agent_words) = direction.words();
    let hooks_path = cwd.join(HOOKS_FILE);
    let local_path = cwd.join(LOCAL_FILE);
    let hooks_before = read_settings(&hooks_path)?;
    let local_before = read_settings(&local_path)?;
    // Both new texts are made before anything is written, so that a settings file that is not
    // valid refuses the whole change.
    let hooks_after = match direction {
        Direction::Install => hooks::installed(hooks_before.as_deref().unwrap_or(""))?,
        Direction::Uninstall => hooks::uninstalled(hooks_before.as_deref().unwrap_or(""))?,
    };
    let local_after = match direction {
        Direction::Install => {
            let agent = format!("{} - claude", writer_person(g));
            hooks::local_installed(local_before.as_deref().unwrap_or(""), &agent)?
        }
        Direction::Uninstall => hooks::local_uninstalled(local_before.as_deref().unwrap_or(""))?,
    };
    let mut plans = [
        Change::plan(
            hooks_path,
            HOOKS_FILE,
            hooks_words,
            hooks_before,
            hooks_after,
            hook_lines,
        )?,
        Change::plan(
            local_path,
            LOCAL_FILE,
            agent_words,
            local_before,
            local_after,
            agent_lines,
        )?,
    ];

    if plans.iter().all(|plan| !plan.changed) {
        let headline = format!(
            "{} and {}: nothing to change",
            hooks_words.kept, agent_words.kept
        );
        return Ok(Output::with(headline, data(&plans)));
    }
    let lines: Vec<String> = plans
        .iter()
        .flat_map(|plan| {
            plan.added
                .iter()
                .map(|line| format!("+ {line}"))
                .chain(plan.removed.iter().map(|line| format!("- {line}")))
        })
        .collect();

    if !yes {
        let count = plans.iter().filter(|plan| plan.changed).count();
        let headline = format!(
            "would {}; add --yes to write {}",
            joined(&plans, |plan| plan.words.would, " and "),
            if count == 1 { "it" } else { "them" }
        );
        return Ok(Output::with(listing(&headline, &lines), data(&plans)));
    }
    if g.dry_run {
        let headline = format!(
            "dry run: would write {}",
            joined(&plans, |plan| plan.file, " and ")
        );
        return Ok(Output::with(listing(&headline, &lines), data(&plans)));
    }

    for plan in plans.iter_mut().filter(|plan| plan.changed) {
        plan.write()?;
    }
    if let Some(state) = state {
        let detail = joined(&plans, |plan| plan.words.detail, "; ");
        journal::append(&state, &record(&detail, g))?;
    }
    let done = joined(&plans, |plan| plan.words.done, " and ");
    let kept: Vec<String> = plans
        .iter()
        .filter(|plan| plan.backup)
        .map(|plan| format!("{}.bak", plan.file))
        .collect();
    let headline = match kept.as_slice() {
        [] => done,
        [only] => format!("{done} (the old file is kept as {only})"),
        _ => format!("{done} (the old files are kept as {})", kept.join(" and ")),
    };
    Ok(Output::with(listing(&headline, &lines), data(&plans)))
}

/// The text of `part` for each changed file, in order, joined with `joiner`.
fn joined(plans: &[Change], part: fn(&Change) -> &'static str, joiner: &str) -> String {
    plans
        .iter()
        .filter(|plan| plan.changed)
        .map(part)
        .collect::<Vec<_>>()
        .join(joiner)
}

/// The data of a result: whether any settings differ, whether any file was written, and for each
/// of the two files its path, whether it changed and was written, whether its old text was kept as
/// a copy, and the lines that are added or removed.
fn data(plans: &[Change]) -> Value {
    let files: Vec<Value> = plans
        .iter()
        .map(|plan| {
            json!({
                "file": plan.path.display().to_string(),
                "changed": plan.changed,
                "written": plan.written,
                "backup": plan.backup,
                "added": plan.added,
                "removed": plan.removed,
            })
        })
        .collect();
    json!({
        "changed": plans.iter().any(|plan| plan.changed),
        "written": plans.iter().any(|plan| plan.written),
        "files": files,
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

/// The person whose LLM Claude's writes are for (D34): the person of the name that writes, when a
/// dossier is found and the name resolves; otherwise the name from `--agent`, DOSSIER_AGENT or the
/// operating-system user, as `names::unchecked` gives it. Never an error.
fn writer_person(g: &Global) -> String {
    Ctx::load(g, "hooks")
        .ok()
        .and_then(|ctx| ctx.writer().ok().map(|who| who.person.clone()))
        .unwrap_or_else(|| names::unchecked(g.agent.as_deref()).person)
}

/// The journal line of a change made by `hooks`, with who and when (D29).
fn record(detail: &str, g: &Global) -> journal::Record {
    let who = names::unchecked(g.agent.as_deref());
    journal::Record {
        time: time::now(),
        name: who.name,
        person: who.person,
        llm: who.is_llm,
        command: "hooks".to_string(),
        id: ID.to_string(),
        event: "changed".to_string(),
        detail: detail.to_string(),
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

/// The copy of a settings file kept before a change: the same path with `.bak` added.
fn backup_of(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
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

/// The line of a local settings value that `hooks` compares: `env DOSSIER_AGENT: <name>` when the
/// variable is set to a text, and no line otherwise.
fn agent_lines(settings: &Value) -> Vec<String> {
    settings
        .get("env")
        .and_then(|env| env.get(AGENT_VAR))
        .and_then(Value::as_str)
        .map(|agent| format!("env {AGENT_VAR}: {agent}"))
        .into_iter()
        .collect()
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
