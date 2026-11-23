//! Runs the built program for `session convert --claude`: a transcript of Claude Code is converted
//! to a session file, printed, or written with `--to`. No dossier is needed; each test runs in its
//! own temporary folder, and the text converted is invented.

use assert_cmd::Command;
use predicates::str::{contains, starts_with};
use std::path::Path;

/// A transcript of three JSON lines: a summary, a user turn and an assistant turn with a tool use.
const TRANSCRIPT: &str = concat!(
    r#"{"type":"summary","summary":"Sizing the flow test"}"#,
    "\n",
    r#"{"type":"user","message":{"role":"user","content":"We go with the SST model."}}"#,
    "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Recorded as DEC-012."},{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"dossier list"}}]}}"#,
    "\n",
);

/// The session file that `TRANSCRIPT` converts to.
const SESSION: &str = "<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n\n## user\n\nWe go with the SST model.\n\n## assistant\n\nRecorded as DEC-012.\n";

/// The program run in `folder` with `args`, not yet checked.
fn dossier(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("dossier")
        .unwrap()
        .current_dir(folder)
        .args(args)
        .assert()
}

/// A new temporary folder with the file `name` holding `bytes`.
fn folder_with(name: &str, bytes: &[u8]) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join(name), bytes).unwrap();
    folder
}

#[test]
fn convert_prints_the_session_file_of_a_transcript() {
    let folder = folder_with("transcript.jsonl", TRANSCRIPT.as_bytes());
    dossier(
        folder.path(),
        &["session", "convert", "--claude", "transcript.jsonl"],
    )
    .success()
    .stdout(SESSION)
    .stderr("");
}

#[test]
fn to_writes_the_session_file_and_says_how_many_turns_it_holds() {
    // Both the transcript name and the folder of the file have a space and a non-ASCII letter.
    let folder = folder_with("Protokoll café.jsonl", TRANSCRIPT.as_bytes());
    dossier(
        folder.path(),
        &[
            "session",
            "convert",
            "--claude",
            "Protokoll café.jsonl",
            "--to",
            "Sitzung café/session 1.md",
        ],
    )
    .success()
    .stdout("2 turns written to Sitzung café/session 1.md\n")
    .stderr("");
    let written = folder.path().join("Sitzung café").join("session 1.md");
    let text = std::fs::read_to_string(&written).unwrap();
    assert_eq!(text, SESSION);
    // The file reads back as the turns that review-session reads.
    let turns = dossier::session::parse(&text).unwrap();
    assert_eq!(
        turns,
        vec![
            dossier::session::Turn {
                role: "user".to_string(),
                text: "We go with the SST model.".to_string(),
            },
            dossier::session::Turn {
                role: "assistant".to_string(),
                text: "Recorded as DEC-012.".to_string(),
            },
        ]
    );
}

#[test]
fn json_gives_the_turn_count_and_the_file_written() {
    let folder = folder_with("transcript.jsonl", TRANSCRIPT.as_bytes());
    let stdout = dossier(
        folder.path(),
        &[
            "session",
            "convert",
            "--claude",
            "transcript.jsonl",
            "--to",
            "session.md",
            "--json",
        ],
    )
    .success()
    .get_output()
    .stdout
    .clone();
    let value: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "schema": "dossier.session.v1",
            "ok": true,
            "data": {"turns": 2, "file": "session.md"},
            "warnings": [],
        })
    );
}

#[test]
fn a_transcript_with_no_readable_line_gives_the_warning_and_exit_0() {
    let folder = folder_with(
        "transcript.jsonl",
        b"not json\n{\"type\":\"summary\",\"summary\":\"x\"}\n",
    );
    dossier(
        folder.path(),
        &["session", "convert", "--claude", "transcript.jsonl"],
    )
    .success()
    .stdout("<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n")
    .stderr(contains(
        "warning: no turn could be read from transcript.jsonl; the transcript format may have changed",
    ));
}

#[test]
fn to_with_no_readable_line_writes_no_turns_and_gives_the_warning() {
    let folder = folder_with("transcript.jsonl", b"not json\n");
    dossier(
        folder.path(),
        &[
            "session",
            "convert",
            "--claude",
            "transcript.jsonl",
            "--to",
            "session.md",
        ],
    )
    .success()
    .stdout("0 turns written to session.md\n")
    .stderr(contains(
        "warning: no turn could be read from transcript.jsonl; the transcript format may have changed",
    ));
    assert!(folder.path().join("session.md").is_file());
}

#[test]
fn a_transcript_that_is_not_utf8_is_read_with_replacement_characters() {
    // The byte FF is not UTF-8; the line still holds one turn.
    let folder = folder_with(
        "transcript.jsonl",
        b"{\"type\":\"user\",\"message\":{\"content\":\"Ben\xFF says yes\"}}\n",
    );
    dossier(
        folder.path(),
        &["session", "convert", "--claude", "transcript.jsonl"],
    )
    .success()
    .stdout(contains("user\n\nBen\u{FFFD} says yes\n"));
}

#[test]
fn a_transcript_that_is_not_there_is_not_found() {
    let folder = tempfile::tempdir().unwrap();
    dossier(
        folder.path(),
        &["session", "convert", "--claude", "missing.jsonl"],
    )
    .code(1)
    .stderr(contains("the transcript is not there: missing.jsonl"));
}

#[test]
fn convert_without_claude_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    dossier(folder.path(), &["session", "convert"])
        .code(2)
        .stdout("")
        .stderr(starts_with(
            "error: name the tool the transcript comes from: --claude <transcript>",
        ));
}
