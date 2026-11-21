//! The `projects` command: lists the projects of this PC's register, or forgets one (R-REMOTE-10).
//! It runs on the PC and needs no dossier. With `--check`, it also runs ssh for each project to
//! show whether its server answers.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::remote::register::{self, Project};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::process::{Command, Stdio};

/// How long ssh may take to connect, in seconds (R-REMOTE-11, `ssh_timeout_seconds`).
const SSH_TIMEOUT_SECONDS: u64 = 10;

/// The arguments of `projects`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Shows for each project whether its server answers (runs ssh once per project)
    #[arg(long)]
    pub check: bool,
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// What `projects` does besides listing the register.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Forgets a project in the register; a folder connected to it keeps its pointer file
    Remove {
        /// The project code, such as P-017
        code: String,
    },
}

/// Runs `projects`: lists the register (with `--check`, whether the servers answer), or removes
/// one project from it. `--check` is for the list only.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    match (&args.action, args.check) {
        (None, check) => list(check),
        (Some(_), true) => Err(Error::new(
            Code::Usage,
            "--check only lists the projects; it takes no subcommand",
        )),
        (Some(Action::Remove { code }), false) => remove(code),
    }
}

/// One line per project, in the order of the register. The data is a list of
/// {code, server, folder, local}. With `check`, each project also says whether its server
/// answers (`answers`). An empty register says how to add a project.
fn list(check: bool) -> Result<Output> {
    let projects = register::load()?;
    if projects.is_empty() {
        return Ok(Output::with(
            "no projects; add one with: dossier connect <user@server>:<folder>",
            json!([]),
        ));
    }
    let mut lines = Vec::new();
    let mut data = Vec::new();
    for project in &projects {
        let answer = check.then(|| answers(project));
        lines.push(project_line(project, answer));
        data.push(project_data(project, answer));
    }
    Ok(Output::with(lines.join("\n"), Value::Array(data)))
}

/// `P-017  anna@sim01:/projects/P-017  C:\Users\anna\P-017`, with `-` when there is no local
/// folder. With `answer` (from `--check`), `  answers` or `  no answer` comes after it.
fn project_line(project: &Project, answer: Option<bool>) -> String {
    let local = project.local.as_deref().unwrap_or("-");
    let line = format!(
        "{}  {}:{}  {}",
        project.code, project.server, project.folder, local
    );
    match answer {
        Some(true) => format!("{line}  answers"),
        Some(false) => format!("{line}  no answer"),
        None => line,
    }
}

/// One project as JSON: code, server, folder and local (null when there is none). With `answer`
/// (from `--check`), also `answers`: whether the server answers.
fn project_data(project: &Project, answer: Option<bool>) -> Value {
    let mut data = json!({
        "code": project.code,
        "server": project.server,
        "folder": project.folder,
        "local": project.local,
    });
    if let Some(answer) = answer {
        data["answers"] = json!(answer);
    }
    data
}

/// Whether the server of `project` answers: ssh runs `true` there with the arguments
/// `-o BatchMode=yes -o ConnectTimeout=10 <server> true`, and the answer is exit code 0. Its
/// input is empty and its output is captured. Any other exit code, and no ssh program at all,
/// is no answer.
fn answers(project: &Project) -> bool {
    let connect_timeout = format!("ConnectTimeout={SSH_TIMEOUT_SECONDS}");
    Command::new(ssh_program())
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            connect_timeout.as_str(),
            project.server.as_str(),
            "true",
        ])
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.code() == Some(0))
}

/// The ssh program: DOSSIER_SSH when it is set (tests point it at a stand-in), else `ssh`.
fn ssh_program() -> OsString {
    std::env::var_os("DOSSIER_SSH")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("ssh"))
}

/// Removes the project `code` from the register. Code::NotFound when there is no such project.
fn remove(code: &str) -> Result<Output> {
    if !register::remove(code)? {
        return Err(Error::new(Code::NotFound, format!("no project {code}")));
    }
    Ok(Output::with(
        format!("removed {code}"),
        json!({"code": code, "removed": true}),
    ))
}
