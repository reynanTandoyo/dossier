//! The `todo` command: runs `todo` in the project folder, with the arguments of the person, and
//! passes its output and its exit code on.

use crate::error::{Code, Error, Result};
use crate::find;
use crate::output::Output;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// What the person sees when todo cannot be started.
const MISSING: &str = "todo is not installed or not on PATH; dossier needs todo 0.8 or newer";

/// The arguments of `todo`: everything after the command name is passed on as it is.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The arguments of `todo`, for example `list --wide`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

/// Runs `todo <arguments>` in the project folder (the parent of the DOSSIER folder), with this
/// program's standard input, output and error. Exit code 0 gives no output of its own; any
/// other exit code of `todo` ends this program with the same code.
pub fn run(args: &Args, g: &crate::cli::Global) -> Result<Output> {
    let current = std::env::current_dir()?;
    // Made absolute, so that the project folder has a parent even for a relative --dossier.
    let dossier = current.join(find::find(&current, g.dossier.as_deref())?);
    let root = dossier.parent().ok_or_else(|| {
        Error::new(
            Code::NoDossier,
            format!("{} has no parent folder", dossier.display()),
        )
    })?;
    let program = program();
    if missing_file(&program) {
        return Err(Error::new(Code::TodoMissing, MISSING));
    }
    let status = Command::new(&program)
        .args(&args.args)
        .current_dir(root)
        .status()
        .map_err(|_| Error::new(Code::TodoMissing, MISSING))?;
    match status.code() {
        Some(0) => Ok(Output::text("")),
        Some(code) => {
            let _ = std::io::stdout().flush();
            let _ = std::io::stderr().flush();
            std::process::exit(code)
        }
        None => Err(Error::new(Code::Io, "todo ended without an exit code")),
    }
}

/// The program to run: the value of `DOSSIER_TODO` when it is set and not empty, else `todo`
/// (the same rule as the todo bridge, docs/design.md section 3).
fn program() -> String {
    match std::env::var_os("DOSSIER_TODO") {
        Some(value) if !value.is_empty() => value.to_string_lossy().into_owned(),
        _ => "todo".to_string(),
    }
}

/// Whether `program` is a path (it has a folder part) to a file that does not exist. On Windows
/// a .cmd or .bat program runs through cmd.exe, which starts even when the file is missing and
/// then exits with 1; so the missing file is reported here as Code::TodoMissing. Elsewhere a
/// missing program does not start and is reported when it is run.
fn missing_file(program: &str) -> bool {
    let path = Path::new(program);
    cfg!(windows) && path.components().count() > 1 && !path.is_file()
}
