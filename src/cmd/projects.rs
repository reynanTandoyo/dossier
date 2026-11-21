//! The `projects` command: lists the projects of this PC's register, or forgets one (R-REMOTE-10).
//! It runs on the PC and needs no dossier.

use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::remote::register::{self, Project};
use serde_json::{Value, json};

/// The arguments of `projects`.
#[derive(clap::Args, Debug)]
pub struct Args {
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

/// Runs `projects`: lists the register, or removes one project from it.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    match &args.action {
        None => list(),
        Some(Action::Remove { code }) => remove(code),
    }
}

/// One line per project, in the order of the register. The data is a list of
/// {code, server, folder, local}. An empty register says how to add a project.
fn list() -> Result<Output> {
    let projects = register::load()?;
    if projects.is_empty() {
        return Ok(Output::with(
            "no projects; add one with: dossier connect <user@server>:<folder>",
            json!([]),
        ));
    }
    let lines: Vec<String> = projects.iter().map(project_line).collect();
    let data: Vec<Value> = projects.iter().map(project_data).collect();
    Ok(Output::with(lines.join("\n"), Value::Array(data)))
}

/// `P-017  anna@sim01:/projects/P-017  C:\Users\anna\P-017`, with `-` when there is no local
/// folder.
fn project_line(project: &Project) -> String {
    let local = project.local.as_deref().unwrap_or("-");
    format!(
        "{}  {}:{}  {}",
        project.code, project.server, project.folder, local
    )
}

/// One project as JSON: code, server, folder and local (null when there is none).
fn project_data(project: &Project) -> Value {
    json!({
        "code": project.code,
        "server": project.server,
        "folder": project.folder,
        "local": project.local,
    })
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
