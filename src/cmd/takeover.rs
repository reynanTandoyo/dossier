//! The `takeover` command (docs/design.md section 22, R-HAND-2, R-HAND-3 and R-HAND-5, D54): takes
//! over a task in todo, also one a colleague holds, and shows its progress. It never copies a
//! file: the files the previous holder has not returned are named, with the take command for them.

use crate::cli::Global;
use crate::config::Config;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::journal;
use crate::names::{self, Who};
use crate::output::Output;
use crate::paths;
use crate::taken::{self, Take};
use crate::todo_bridge;
use crate::tx;
use serde_json::{Value, json};
use std::ffi::OsString;
use std::process::{Command, Stdio};

/// The environment variable that names the agent in todo, when it is set (R-HAND-5).
const TODO_AGENT_VAR: &str = "TODO_AGENT";
/// The holder named in a note when the task had none.
const NOBODY: &str = "nobody";
/// The event of the journal line (R-HAND-2).
const TAKEOVER: &str = "takeover";

/// The arguments of `takeover`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The todo task, such as T-042
    pub task: String,
    /// The person who agreed to the takeover; an LLM needs it (D52)
    #[arg(long, value_name = "PERSON")]
    pub said_by: Option<String>,
}

/// Runs `takeover`: claims the task in todo for the writer, adds the note, and writes the journal
/// line in one transaction, so that a todo failure writes nothing. Then it names the files the
/// previous holder has not returned and prints what `brief --task` prints. The output is
/// `<task> taken over from <previous holder or nobody>`, or `<task> is already yours` when the
/// writer holds it, and then changes nothing.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "takeover")?;
    let who = ctx.writer()?.clone();
    let task = args.task.trim().to_string();
    let said_by = said_by(&ctx.cfg, &who, args.said_by.as_deref(), &task)?;
    let todo_name = todo_name(&who);

    let mut claimed = false;
    let mut output = tx::write(&ctx, |tx| {
        let shown = todo_bridge::json(&ctx.root, &["show", task.as_str()])?;
        let owner = owner_of(&shown);
        if owner
            .as_deref()
            .is_some_and(|holder| same(holder, &todo_name))
        {
            return Ok(Output::text(format!("{task} is already yours")));
        }

        let from = owner.clone().unwrap_or_else(|| NOBODY.to_string());
        let note = note_text(&who, &from, said_by.as_deref());
        let files = unreturned(&ctx, &who, &task)?;
        let mut lines = Vec::new();
        for take in &files {
            lines.push(format!(
                "{}: taken by {} on {} and not returned; changes on their PC are not on the server",
                paths::shown(&take.path),
                take.person,
                take.time
            ));
        }
        if !files.is_empty() {
            let listed: Vec<String> = files
                .iter()
                .map(|take| argument(&paths::shown(&take.path)))
                .collect();
            lines.push(format!(
                "take them with: dossier take {} --task {task} --force",
                listed.join(" ")
            ));
        }
        let files: Vec<Value> = files
            .iter()
            .map(|take| json!({"path": take.path, "person": take.person, "time": take.time}))
            .collect();
        let data = json!({"task": task, "from": owner, "files": files});
        if g.dry_run {
            // A dry run changes nothing: no claim, no note, no journal line.
            let warning =
                format!("dry run: would claim {task} from {from} and add the note \"{note}\"");
            return Ok(Output::with(lines.join("\n"), data).warn(warning));
        }

        let mut claim = vec!["claim", task.as_str(), "--agent", todo_name.as_str()];
        if owner.is_some() {
            claim.push("--force");
        }
        todo_bridge::json(&ctx.root, &claim)?;
        todo_bridge::json(
            &ctx.root,
            &[
                "note",
                task.as_str(),
                note.as_str(),
                "--agent",
                todo_name.as_str(),
            ],
        )?;
        journal::append(&ctx.state, &record(tx.time(), &who, &task, &note))?;
        claimed = true;
        lines.insert(0, format!("{task} taken over from {from}"));
        Ok(Output::with(lines.join("\n"), data))
    })?;

    if claimed && let Some(brief) = brief_of(g, &task) {
        output.text.push_str("\n\n");
        output.text.push_str(brief.trim_end());
    }
    Ok(output)
}

/// The person who agreed to the takeover, resolved as decide resolves --said-by (create.rs): an
/// LLM needs one, else Code::NeedsPerson, and it must name a person, member or plain name of the
/// settings (Code::UnknownName otherwise). A person's --said-by is not used, as decide does not use
/// it for a person. None when no person is named.
fn said_by(cfg: &Config, who: &Who, given: Option<&str>, task: &str) -> Result<Option<String>> {
    if !who.is_llm {
        return Ok(None);
    }
    let Some(name) = given.map(str::trim).filter(|name| !name.is_empty()) else {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("ask your person; with their yes: dossier takeover {task} --said-by <person>"),
        ));
    };
    as_written(cfg, name).map(Some)
}

/// The name called `name` (ignoring case) as the settings write it: a person, a member or a plain
/// name (names::is_known). Code::UnknownName for any other name. The same rule as create's
/// `name_as_written`.
fn as_written(cfg: &Config, name: &str) -> Result<String> {
    let unknown = || {
        Error::new(
            Code::UnknownName,
            format!("no person {name} in dossier.toml"),
        )
    };
    if !names::is_known(cfg, name) {
        return Err(unknown());
    }
    let wanted = name.trim().to_lowercase();
    cfg.people
        .iter()
        .map(|person| person.name.as_str())
        .chain(cfg.members.iter().map(String::as_str))
        .chain(cfg.names.iter().map(String::as_str))
        .find(|known| known.to_lowercase() == wanted)
        .map(str::to_string)
        .ok_or_else(unknown)
}

/// The name given to todo: TODO_AGENT when it is set and not empty, else the writer's name
/// (R-HAND-5).
fn todo_name(who: &Who) -> String {
    std::env::var(TODO_AGENT_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| who.name.clone())
}

/// The holder of a task in todo's data: its `owner`, None when it is null or empty.
fn owner_of(data: &Value) -> Option<String> {
    data.get("owner")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|owner| !owner.is_empty())
        .map(str::to_string)
}

/// The todo note of a takeover: `takeover — <writer> took over from <holder>`, with
/// ` (said by <person>)` at its end when a person is named (R-HAND-2).
fn note_text(who: &Who, from: &str, said_by: Option<&str>) -> String {
    let mut note = format!("takeover — {} took over from {from}", who.name);
    if let Some(person) = said_by {
        note.push_str(&format!(" (said by {person})"));
    }
    note
}

/// Names are compared ignoring case and surrounding spaces.
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// The files taken for the task by a person other than the writer, in file order (R-HAND-2).
fn unreturned(ctx: &Ctx, who: &Who, task: &str) -> Result<Vec<Take>> {
    let (file, _) = taken::load(&ctx.dossier, &ctx.cfg.code)?;
    Ok(taken::all(&file)
        .into_iter()
        .filter(|take| same(&take.task, task) && !same(&take.person, &who.person))
        .collect())
}

/// A path as a command-line argument: in double quotes when it has a space.
fn argument(path: &str) -> String {
    if path.contains(char::is_whitespace) {
        format!("\"{path}\"")
    } else {
        path.to_string()
    }
}

/// The journal line of a takeover: the task as its ID, the note as its detail (R-HAND-2, R-HAND-5).
fn record(time: &str, who: &Who, task: &str, note: &str) -> journal::Record {
    journal::Record {
        time: time.to_string(),
        name: who.name.clone(),
        person: who.person.clone(),
        llm: who.is_llm,
        command: "takeover".to_string(),
        id: task.to_string(),
        event: TAKEOVER.to_string(),
        detail: note.to_string(),
    }
}

/// The standard output of `brief --task <task>` run by this program in the current folder. The
/// global options of this run (--dossier, --project and --agent, each when it is set) are passed
/// on before `brief`, so the brief reads the same dossier from any folder. Standard input is empty
/// and standard error captured. None unless it started and ended with exit 0.
fn brief_of(g: &Global, task: &str) -> Option<String> {
    let mut args: Vec<OsString> = Vec::new();
    if let Some(dossier) = &g.dossier {
        args.push(OsString::from("--dossier"));
        args.push(OsString::from(dossier));
    }
    if let Some(project) = &g.project {
        args.push(OsString::from("--project"));
        args.push(OsString::from(project));
    }
    if let Some(agent) = &g.agent {
        args.push(OsString::from("--agent"));
        args.push(OsString::from(agent));
    }
    args.push(OsString::from("brief"));
    args.push(OsString::from("--task"));
    args.push(OsString::from(task));
    let program = std::env::current_exe().ok()?;
    let output = Command::new(program)
        .args(&args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}
