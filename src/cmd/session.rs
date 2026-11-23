//! The `session` command (docs/design.md section 19, format.md section 34, D44): converts a
//! transcript of Claude Code into a session file, printed or written with `--to`. It needs no
//! dossier and no name, and reads and writes files of the machine it runs on (R-DRAFT-9).

use crate::atomic;
use crate::claude;
use crate::cli::Global;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::session;
use serde_json::json;
use std::path::{Path, PathBuf};

/// The arguments of `session`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `session` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Makes a session file from a transcript of Claude Code
    Convert {
        /// The transcript of Claude Code, a file with one JSON object per line
        #[arg(long, value_name = "transcript")]
        claude: Option<PathBuf>,
        /// Writes the session file to this file instead of printing it
        #[arg(long, value_name = "file")]
        to: Option<PathBuf>,
    },
}

/// Runs `session`.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    match &args.action {
        Action::Convert { claude, to } => convert(claude.as_deref(), to.as_deref()),
    }
}

/// Converts the transcript `claude` (Code::Usage without one). The session file is printed, or
/// written to `to` when it is given. When no turn could be read, a warning says that the format
/// of the transcript may have changed.
fn convert(claude: Option<&Path>, to: Option<&Path>) -> Result<Output> {
    let Some(transcript) = claude else {
        return Err(Error::new(
            Code::Usage,
            "name the tool the transcript comes from: --claude <transcript>",
        ));
    };
    let turns = claude::turns(&read_transcript(transcript)?);
    let text = session::render("claude code", &turns);
    let count = turns.len();
    let noun = if count == 1 { "turn" } else { "turns" };
    let output = match to {
        Some(file) => {
            atomic::replace(file, text.as_bytes(), None)?;
            Output::with(
                format!("{count} {noun} written to {}", file.display()),
                json!({"turns": count, "file": file.display().to_string()}),
            )
        }
        None => Output::with(text, json!({"turns": count, "file": null})),
    };
    if turns.is_empty() {
        Ok(output.warn(format!(
            "no turn could be read from {}; the transcript format may have changed",
            transcript.display()
        )))
    } else {
        Ok(output)
    }
}

/// The text of the transcript file. A missing file is Code::NotFound; bytes that are not UTF-8
/// are read with replacement characters, so that the lines that are readable still convert.
fn read_transcript(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::new(
                Code::NotFound,
                format!("the transcript is not there: {}", path.display()),
            )
        } else {
            Error::new(Code::Io, format!("{}: {error}", path.display()))
        }
    })?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
