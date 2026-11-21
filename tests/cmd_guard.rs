//! Runs the built program for `guard`: a Claude Code PreToolUse payload on standard input. Exit 2
//! with a message on standard error blocks an edit of a file inside a DOSSIER folder; every other
//! payload, and text that is not a payload, exits 0 with nothing printed. The payloads are
//! invented, and the program runs in a temporary project folder (common::Project).

mod common;

use common::Project;
use predicates::str::{contains, is_empty};
use serde_json::json;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A Claude Code PreToolUse payload for the Edit tool on `path`, as JSON text.
fn edit(path: &str) -> String {
    json!({
        "session_id": "invented-session",
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": path,
            "old_string": "invented old text",
            "new_string": "invented new text",
        },
    })
    .to_string()
}

/// Runs `guard` in the project folder with `payload` on standard input.
fn run_guard(project: &Project, payload: &str) -> assert_cmd::assert::Assert {
    project
        .command("", NOW)
        .arg("guard")
        .write_stdin(payload)
        .assert()
}

#[test]
fn an_edit_of_an_entry_file_is_blocked_with_exit_2() {
    let project = Project::new();
    run_guard(&project, &edit("DOSSIER/findings/FND-001-x.md"))
        .code(2)
        .stdout(is_empty())
        .stderr(contains("DOSSIER"));
}

#[test]
fn an_edit_of_a_file_outside_the_dossier_is_allowed() {
    let project = Project::new();
    run_guard(&project, &edit("README.md"))
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn text_that_is_not_a_payload_is_allowed() {
    let project = Project::new();
    run_guard(&project, "this is not a payload")
        .success()
        .stdout(is_empty());
}

#[test]
fn a_windows_path_with_spaces_and_non_ascii_letters_is_blocked() {
    let project = Project::new();
    run_guard(
        &project,
        &edit(r"C:\Projekte\Mein Ordner\DOSSIER\findings\FND-001-ä.md"),
    )
    .code(2)
    .stderr(contains("DOSSIER"));
}

#[test]
fn a_payload_that_starts_with_a_byte_order_mark_is_read() {
    let project = Project::new();
    run_guard(
        &project,
        &format!("\u{feff}{}", edit("DOSSIER/findings/FND-001-x.md")),
    )
    .code(2)
    .stderr(contains("DOSSIER"));
}
