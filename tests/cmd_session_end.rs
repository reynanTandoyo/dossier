//! Runs the built program for `session-end` as the SessionEnd hook runs it: the hook payload on
//! standard input names the transcript of a session; the review material is written to
//! `.claude/dossier-session-review.md` in the project folder, and the review command is started on
//! it. A payload that leads to no material ends with exit 0 and no file. Each test runs in its own
//! temporary folder.

mod common;

use common::Project;
use serde_json::json;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

/// The clock of the runs. The command does not read it, but every run of the helper needs one.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The material file, relative to the project folder (format.md §33).
const MATERIAL: &str = ".claude/dossier-session-review.md";

/// A transcript of Claude Code with one turn that states a decision (a cue), the same as in
/// cmd_review_session.rs.
const TRANSCRIPT: &str = concat!(
    r#"{"type":"summary","summary":"Sizing the SST run"}"#,
    "\n",
    r#"{"type":"user","message":{"role":"user","content":"We go with the SST model because the test data is for attached flow."}}"#,
    "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Recorded as DEC-012. The first run diverges at iteration 40."}]}}"#,
    "\n",
);

/// A transcript whose turns have no cue, so that review-session finds nothing relevant.
const PLAIN_TRANSCRIPT: &str = concat!(
    r#"{"type":"user","message":{"role":"user","content":"Hello there."}}"#,
    "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hi."}]}}"#,
    "\n",
);

/// The hook payload that names `relative`, a file of the project folder, as the transcript.
fn payload(project: &Project, relative: &str) -> String {
    let transcript = project.root().join(relative);
    json!({ "transcript_path": transcript.display().to_string() }).to_string()
}

/// Runs `session-end --run <run>` in the project folder as anna with `stdin` as the payload, and
/// asserts exit 0.
fn session_end(project: &Project, run: &str, stdin: &str) {
    project
        .command("anna", NOW)
        .args(["session-end", "--run", run])
        .write_stdin(stdin)
        .assert()
        .success();
}

/// Writes a stand-in review command that copies the file it is given to `marker.md` in the
/// project folder, and returns the text for `--run`: its path in double quotes, so that a folder
/// with spaces in its name still works.
fn copy_command(project: &Project) -> String {
    let path = project.write_stand_in(
        "review",
        "@echo off\r\ncopy /Y \"%~1\" marker.md > nul\r\n",
        "#!/bin/sh\ncp \"$1\" marker.md\n",
    );
    format!("\"{}\"", path.display())
}

/// Whether `marker.md` in the project folder holds `expected` within five seconds. The review
/// command runs after session-end has returned, so the test polls for its result.
fn marker_holds(project: &Project, expected: &str) -> bool {
    let path = project.root().join("marker.md");
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if fs::read_to_string(&path).is_ok_and(|text| text == expected) {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn session_end_writes_the_review_material_of_a_transcript_with_a_cue() {
    let project = Project::new();
    project.write("transcript.jsonl", TRANSCRIPT);
    let review = project.stdout("anna", &["review-session", "--claude", "transcript.jsonl"]);
    session_end(&project, "echo", &payload(&project, "transcript.jsonl"));
    let material = project.read(MATERIAL);
    assert!(material.contains("# Review of the session"), "{material}");
    // The file is what review-session prints for the same transcript.
    assert_eq!(material, review);
}

#[test]
fn session_end_starts_the_review_command_with_the_material_path_last() {
    let project = Project::new();
    project.write("transcript.jsonl", TRANSCRIPT);
    let run = copy_command(&project);
    session_end(&project, &run, &payload(&project, "transcript.jsonl"));
    let material = project.read(MATERIAL);
    assert!(
        marker_holds(&project, &material),
        "marker.md does not hold the material within five seconds"
    );
}

#[test]
fn session_end_does_nothing_without_a_transcript_path() {
    let project = Project::new();
    session_end(&project, "echo", "{}");
    assert!(!project.root().join(".claude").exists());
}

#[test]
fn session_end_does_nothing_for_a_transcript_without_a_cue() {
    let project = Project::new();
    project.write("plain.jsonl", PLAIN_TRANSCRIPT);
    session_end(&project, "echo", &payload(&project, "plain.jsonl"));
    assert!(!project.root().join(MATERIAL).exists());
}

#[test]
fn session_end_does_nothing_for_a_transcript_that_does_not_exist() {
    let project = Project::new();
    session_end(&project, "echo", &payload(&project, "missing.jsonl"));
    assert!(!project.root().join(MATERIAL).exists());
}

#[test]
fn session_end_does_nothing_for_a_payload_that_is_not_json() {
    let project = Project::new();
    session_end(&project, "echo", "transcript_path: transcript.jsonl");
    assert!(!project.root().join(MATERIAL).exists());
}
