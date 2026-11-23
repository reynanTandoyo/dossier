//! The `review-session` command (docs/design.md section 19, R-DRAFT-7, R-DRAFT-9): the decisive
//! parts of a session as material for drafting. The file is a session file (format.md §34), or
//! with `--claude` a transcript of Claude Code that is converted first. The drafting rules of
//! `review-task` are printed after the material. Nothing is changed, and no dossier is needed:
//! the file is read where it is, on the machine of the session.

use crate::claude;
use crate::cli::Global;
use crate::cmd::review_task::RULES;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::session;
use serde_json::json;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// What is printed when no turn of the session is relevant.
const NOTHING: &str = "nothing in this session looks like a decision or a result";

/// The arguments of `review-session`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The session file, or with --claude a transcript of Claude Code
    pub file: PathBuf,
    /// The file is a transcript of Claude Code, converted first
    #[arg(long)]
    pub claude: bool,
    /// The most characters of material; the oldest turns are left out above it
    #[arg(long, default_value_t = 20000, value_name = "characters")]
    pub limit: usize,
}

/// Runs `review-session`: reads the file, keeps the turns that state a decision or discuss a
/// result (`session::relevant`), and prints them with the drafting rules. Nothing is written.
pub fn run(args: &Args, _g: &Global) -> Result<Output> {
    let text = read(&args.file)?;
    let turns = if args.claude {
        claude::turns(&text)
    } else {
        session::parse(&text)?
    };
    let material = session::relevant(&turns, args.limit);
    let (total, kept) = (turns.len(), material.len());
    let text = if material.is_empty() {
        NOTHING.to_string()
    } else {
        let mut lines = vec![
            "# Review of the session".to_string(),
            format!("{kept} of {total} turns"),
            String::new(),
        ];
        lines.extend(
            material
                .iter()
                .flat_map(|turn| [turn.clone(), String::new()]),
        );
        lines.push("## Drafting rules".to_string());
        lines.extend(
            RULES
                .iter()
                .enumerate()
                .map(|(index, rule)| format!("{}. {rule}", index + 1)),
        );
        lines.join("\n")
    };
    let data = json!({
        "turns": total,
        "kept": kept,
        "material": material,
        "rules": RULES,
    });
    Ok(Output::with(text, data))
}

/// The text of the file. A missing file is Code::NotFound naming it; bytes that are not UTF-8 are
/// read lossily, so that one bad byte does not stop the review.
fn read(file: &Path) -> Result<String> {
    match std::fs::read(file) {
        Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(Error::new(
            Code::NotFound,
            format!("no file {}", file.display()),
        )),
        Err(error) => Err(error.into()),
    }
}
