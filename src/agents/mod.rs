//! The rules block that `dossier agents install` keeps in AGENTS.md.

/// The rules text (agents/rules.md).
pub const RULES: &str = include_str!("rules.md");

/// The line that starts the block.
pub const BEGIN: &str = "<!-- dossier:begin -->";

/// The line that ends the block.
pub const END: &str = "<!-- dossier:end -->";

/// The block: BEGIN, the rules, END.
pub fn block() -> String {
    String::new()
}

/// The text of AGENTS.md with the block added (after todo's block if there is one, else at the
/// end) or replaced; every other line unchanged.
pub fn installed(existing: &str) -> String {
    String::new()
}

/// Whether the block in `existing` is current.
pub fn current(existing: &str) -> bool {
    false
}
