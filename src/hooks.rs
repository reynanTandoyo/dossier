//! The Claude Code hooks that `dossier hooks install` keeps in `.claude/settings.json`.

use crate::error::{Code, Error, Result};

/// The command of the SessionStart hook: it prints the brief.
pub const BRIEF: &str = "dossier brief";

/// The command of the PreToolUse hook: it blocks an edit inside a dossier folder.
pub const GUARD: &str = "dossier guard";

/// .claude/settings.json with a SessionStart hook running BRIEF and a PreToolUse hook (matcher
/// "Edit|Write|MultiEdit") running GUARD, each added once; other keys kept. Empty text is "{}".
/// Code::Parse when the text is not a JSON object.
pub fn installed(_existing: &str) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "hooks::installed"))
}

/// The settings text with exactly the two entries of `installed` removed; other keys kept.
pub fn uninstalled(_existing: &str) -> Result<String> {
    Err(Error::new(Code::NotImplemented, "hooks::uninstalled"))
}

/// The message to block with when a Claude Code PreToolUse payload edits a file inside a folder
/// named DOSSIER; None to allow.
pub fn guard(_payload: &str) -> Option<String> {
    None
}
