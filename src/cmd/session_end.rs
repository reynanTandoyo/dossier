//! The `session-end` command (docs/design.md section 19, R-DRAFT-9): the command the SessionEnd
//! hook of Claude Code runs with `--run <command>`. It reads the hook payload from standard input,
//! writes the review material of the session's transcript to `.claude/dossier-session-review.md`
//! in the current folder (format.md §33), and starts the team's review command on that file
//! without waiting for it. It never fails: whatever goes wrong, the hook ends quietly.

use crate::cli::Global;
use crate::error::Result;
use crate::output::Output;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The folder below the current folder that holds the review material.
const FOLDER: &str = ".claude";

/// The name of the review material in `FOLDER`.
const FILE: &str = "dossier-session-review.md";

/// The start of the output of `review-session` when no turn of the session is relevant.
const NOTHING: &[u8] = b"nothing in this session";

/// The environment variable that holds the command line for cmd.exe on Windows (see `start`).
const COMMAND_LINE: &str = "DOSSIER_REVIEW_COMMAND";

/// The arguments of `session-end`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The team's review command. It is started with the path of the review material as its
    /// last argument.
    #[arg(long)]
    pub run: String,
}

/// Runs `session-end`. It never fails: every error ends the hook quietly, so the output is empty
/// and the exit code 0 in every case.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    let _ = review(args);
    Ok(Output::text(""))
}

/// The steps of `session-end`: the transcript named by the payload, its review material, the
/// file that holds the material, and the review command started on it. None when a step finds
/// nothing to do or fails.
fn review(args: &Args) -> Option<()> {
    let transcript = transcript_path()?;
    let material = material(&transcript)?;
    let path = write_material(&material)?;
    start(&args.run, &path)
}

/// The `transcript_path` of the hook payload on standard input. None when the input is not JSON
/// or has no string `transcript_path`.
fn transcript_path() -> Option<String> {
    let mut bytes = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).ok()?;
    let payload: Value = serde_json::from_slice(&bytes).ok()?;
    payload.get("transcript_path")?.as_str().map(str::to_string)
}

/// The output of `review-session --claude <transcript>`, run by this same program with standard
/// input null. None when it does not exit 0, or when it says that nothing in the session is
/// relevant.
fn material(transcript: &str) -> Option<Vec<u8>> {
    let program = std::env::current_exe().ok()?;
    let output = Command::new(program)
        .args(["review-session", "--claude"])
        .arg(transcript)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.starts_with(NOTHING) {
        return None;
    }
    Some(output.stdout)
}

/// Writes the material to `.claude/dossier-session-review.md` in the current folder, creating
/// `.claude` when it is missing and replacing an older file. Returns the full path of the file.
fn write_material(material: &[u8]) -> Option<PathBuf> {
    let folder = std::env::current_dir().ok()?.join(FOLDER);
    std::fs::create_dir_all(&folder).ok()?;
    let path = folder.join(FILE);
    std::fs::write(&path, material).ok()?;
    Some(path)
}

/// Starts `command` with the full path of the material as its last argument and returns at once,
/// without waiting for the process (Claude Code gives the hook about 1.5 seconds). Its standard
/// input, output and error are null.
///
/// On Windows the command is run by `cmd /C`. The line goes in an environment variable that cmd
/// expands, not in an argument: Rust escapes each quote of an argument with a backslash, which cmd
/// does not read, so the quotes around the path would arrive wrong. Elsewhere `sh -c` gets the
/// path as `$1`.
fn start(command: &str, path: &Path) -> Option<()> {
    let mut process = if cfg!(windows) {
        let line = format!("{command} \"{}\"", path.display());
        let mut process = Command::new("cmd");
        process
            .env(COMMAND_LINE, line)
            .args(["/C", "%DOSSIER_REVIEW_COMMAND%"]);
        process
    } else {
        let mut process = Command::new("sh");
        process
            .args(["-c", &format!("{command} \"$1\""), "sh"])
            .arg(path);
        process
    };
    // The child is not waited for: the handle is dropped and the process runs on its own.
    process
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
        .map(drop)
}
