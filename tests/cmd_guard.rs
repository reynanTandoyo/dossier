//! Runs the built program for `guard`: a Claude Code PreToolUse payload on standard input. Exit 2
//! with a message on standard error blocks an edit of a file inside a DOSSIER folder. The first
//! edit of a file in a session that entries in force bind is refused once too, with those entries
//! named; the next edit of it in that session goes through. Every other payload, and text that is
//! not a payload, exits 0 with nothing printed. The payloads are invented, and the program runs in
//! a temporary project folder (common::Project).

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

/// A Claude Code PreToolUse payload for the Edit tool on `path` in `session`, as JSON text; without
/// a session_id when `session` is None.
fn edit_in_session(path: &str, session: Option<&str>) -> String {
    let mut payload = json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": path,
            "old_string": "invented old text",
            "new_string": "invented new text",
        },
    });
    if let Some(session) = session {
        payload["session_id"] = json!(session);
    }
    payload.to_string()
}

/// Makes the approved decision DEC-001 about the folder src/pump, as anna.
fn decide_about_pump(project: &Project) {
    project.stdout(
        "anna",
        &[
            "decide",
            "Keep the seal gap at 0.2 mm.",
            "--reason",
            "Invented tests leaked above it.",
            "--title",
            "Seal gap",
        ],
    );
    project.stdout("anna", &["set", "DEC-001", "--field", "code=src/pump"]);
}

#[test]
fn the_first_edit_of_a_bound_file_in_a_session_is_refused_with_its_entries() {
    let project = Project::new();
    decide_about_pump(&project);
    run_guard(
        &project,
        &edit_in_session("src/pump/seal.rs", Some("s1")),
    )
    .code(2)
    .stdout(is_empty())
    .stderr(contains(
        "Entries in force for src/pump/seal.rs; read them, then make your edit again (dossier show <ID> prints one in full):\n",
    ))
    .stderr(contains("DEC-001 · decision · Seal gap"))
    .stderr(contains("  Keep the seal gap at 0.2 mm."));
}

#[test]
fn the_refusal_names_the_approved_entry_linked_under_the_bound_decision() {
    let project = Project::new();
    decide_about_pump(&project);
    project.stdout(
        "anna",
        &[
            "found",
            "The seal leaks above 40 bar.",
            "--title",
            "Seal leak",
        ],
    );
    project.stdout("anna", &["link", "DEC-001", "FND-001"]);
    run_guard(&project, &edit_in_session("src/pump/seal.rs", Some("s1")))
        .code(2)
        .stdout(is_empty())
        .stderr(contains("DEC-001 · decision · Seal gap"))
        .stderr(contains(
            "  Keep the seal gap at 0.2 mm.\n  linked: FND-001 · Seal leak\n",
        ));
}

#[test]
fn the_same_edit_again_in_the_same_session_is_allowed_without_output() {
    let project = Project::new();
    decide_about_pump(&project);
    let payload = edit_in_session("src/pump/seal.rs", Some("s1"));
    run_guard(&project, &payload).code(2);
    run_guard(&project, &payload)
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn a_different_session_is_refused_again_for_the_same_file() {
    let project = Project::new();
    decide_about_pump(&project);
    run_guard(&project, &edit_in_session("src/pump/seal.rs", Some("s1"))).code(2);
    run_guard(&project, &edit_in_session("src/pump/seal.rs", Some("s2")))
        .code(2)
        .stdout(is_empty())
        .stderr(contains("DEC-001 · decision · Seal gap"));
}

#[test]
fn an_edit_without_a_session_is_allowed_without_output() {
    let project = Project::new();
    decide_about_pump(&project);
    run_guard(&project, &edit_in_session("src/pump/seal.rs", None))
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn an_edit_with_an_empty_session_is_allowed_without_output() {
    let project = Project::new();
    decide_about_pump(&project);
    run_guard(&project, &edit_in_session("src/pump/seal.rs", Some("")))
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn an_edit_of_a_file_that_nothing_points_to_is_allowed_without_output() {
    let project = Project::new();
    decide_about_pump(&project);
    run_guard(&project, &edit_in_session("src/valve/gate.rs", Some("s1")))
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn an_edit_outside_any_dossier_is_allowed_without_output() {
    let outside = tempfile::TempDir::new().expect("a temporary folder");
    assert_cmd::Command::cargo_bin("dossier")
        .expect("the dossier program is built")
        .current_dir(outside.path())
        .env("DOSSIER_NOW", "2026-11-20T10:00:00+01:00")
        .env(
            "DOSSIER_REGISTER",
            outside.path().join(".test").join("projects.toml"),
        )
        .arg("guard")
        .write_stdin(edit_in_session("src/pump/seal.rs", Some("s1")))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
}
