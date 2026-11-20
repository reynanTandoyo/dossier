//! The `where` command: the project folder, the DOSSIER folder, the state folder and the project
//! code that a command run here would work on (R-PLACE-7). No name is needed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::Result;
use crate::output::Output;
use serde_json::json;

/// The arguments of `where`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// Runs `where`: finds the dossier from the current folder and prints its four paths and code.
pub fn run(_args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "where")?;
    let project = ctx.root.display().to_string();
    let dossier = ctx.dossier.display().to_string();
    let state = ctx.state.display().to_string();
    let code = ctx.cfg.code.clone();
    let text = format!("project: {project}\ndossier: {dossier}\nstate: {state}\ncode: {code}");
    let data = json!({
        "project": project,
        "dossier": dossier,
        "state": state,
        "code": code,
    });
    Ok(Output::with(text, data))
}
