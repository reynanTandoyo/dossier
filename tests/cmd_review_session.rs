//! Runs the built program for `review-session`: the turns of a session file (or of a transcript
//! of Claude Code with `--claude`) that state a decision or discuss a result, with the drafting
//! rules, and nothing changed on disk. Each test runs in its own temporary folder.

mod common;

use assert_cmd::Command;
use common::Project;
use dossier::cmd::review_task::RULES;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// format.md section 34: the example as it is written there, and one turn without a cue after it.
const SESSION: &str = "<!-- dossier-session: 1 -->\n# Session\n\n- tool: claude code\n\n## user\n\nWe go with the SST model because the test data is for attached flow.\n\n## assistant\n\nRecorded as DEC-012. The first run diverges at iteration 40.\n\n## user\n\nThanks, that is all for today.\n";

/// The two turns of `SESSION` that have a cue, as `review-session` prints them.
const MATERIAL: [&str; 2] = [
    "user: We go with the SST model because the test data is for attached flow.",
    "assistant: Recorded as DEC-012. The first run diverges at iteration 40.",
];

/// The same two turns as a transcript of Claude Code: a summary line, then one line per turn.
const TRANSCRIPT: &str = concat!(
    r#"{"type":"summary","summary":"Sizing the SST run"}"#,
    "\n",
    r#"{"type":"user","message":{"role":"user","content":"We go with the SST model because the test data is for attached flow."}}"#,
    "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Recorded as DEC-012. The first run diverges at iteration 40."}]}}"#,
    "\n",
);

/// Two turns, the older one with no cue, the newer one with the cues `result` and `failed`.
const SWEEP: &str = "# Session\n\n## user\n\nCan you check the sweep order?\n\n## assistant\n\nThe result failed on run 3.\n";

/// What `review-session` prints for `count` (the line `<n> of <m> turns`) and `material`: the
/// header, the turns each followed by an empty line, then the drafting rules numbered.
fn review_text(count: &str, material: &[&str]) -> String {
    let mut lines = vec![
        "# Review of the session".to_string(),
        count.to_string(),
        String::new(),
    ];
    for turn in material {
        lines.push(turn.to_string());
        lines.push(String::new());
    }
    lines.push("## Drafting rules".to_string());
    lines.extend(
        RULES
            .iter()
            .enumerate()
            .map(|(index, rule)| format!("{}. {rule}", index + 1)),
    );
    format!("{}\n", lines.join("\n"))
}

/// Every file under `folder` with its bytes, sorted by the path below `folder`.
fn tree(folder: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    collect(folder, folder, &mut files);
    files.sort();
    files
}

/// Adds the files below `folder` to `files`, with paths relative to `base`.
fn collect(base: &Path, folder: &Path, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in fs::read_dir(folder).expect("a folder to read") {
        let path = entry.expect("a folder entry").path();
        if path.is_dir() {
            collect(base, &path, files);
        } else {
            let key = path
                .strip_prefix(base)
                .expect("a path below the base")
                .display()
                .to_string();
            files.push((key, fs::read(&path).expect("a file to read")));
        }
    }
}

#[test]
fn review_session_prints_the_turns_with_a_cue_and_the_rules() {
    let project = Project::new();
    project.write("session.md", SESSION);
    let text = project.stdout("anna", &["review-session", "session.md"]);
    // The turn without a cue at the end is left out; the other two are kept, in order.
    assert_eq!(text, review_text("2 of 3 turns", &MATERIAL));
}

#[test]
fn review_session_with_claude_converts_a_transcript_and_gives_the_same_material() {
    let project = Project::new();
    project.write("transcript.jsonl", TRANSCRIPT);
    let text = project.stdout("anna", &["review-session", "--claude", "transcript.jsonl"]);
    // The summary line is no turn, so the transcript has two turns, and both are kept.
    assert_eq!(text, review_text("2 of 2 turns", &MATERIAL));
}

#[test]
fn review_session_refuses_a_session_file_of_a_later_format_with_exit_4() {
    let project = Project::new();
    project.write(
        "session.md",
        "<!-- dossier-session: 2 -->\n# Session\n\n## user\n\nhi\n",
    );
    project
        .run("anna", &["review-session", "session.md"])
        .code(4)
        .stderr(contains("update dossier"));
}

#[test]
fn review_session_says_nothing_relevant_for_an_empty_file_or_a_session_without_a_cue() {
    let project = Project::new();
    project.write("empty.md", "");
    project.write(
        "plain.md",
        "# Session\n\n## user\n\nHello.\n\n## assistant\n\nHi.\n",
    );
    for file in ["empty.md", "plain.md"] {
        let text = project.stdout("anna", &["review-session", file]);
        assert_eq!(
            text, "nothing in this session looks like a decision or a result\n",
            "{file}"
        );
    }
}

#[test]
fn review_session_of_a_missing_file_exits_with_1_and_names_it() {
    let project = Project::new();
    project
        .run("anna", &["review-session", "missing.md"])
        .code(1)
        .stderr(contains("missing.md"));
}

#[test]
fn review_session_limit_keeps_only_the_newest_turn_that_fits() {
    let project = Project::new();
    project.write("sweep.md", SWEEP);
    // Without a limit both turns are kept: the older one is kept because it is before a cue.
    let all = project.stdout("anna", &["review-session", "sweep.md"]);
    assert!(all.contains("2 of 2 turns"), "{all}");
    assert!(
        all.contains("user: Can you check the sweep order?"),
        "{all}"
    );
    // With 50 characters only the newer turn (38 characters) fits.
    let text = project.stdout("anna", &["review-session", "sweep.md", "--limit", "50"]);
    assert_eq!(
        text,
        review_text("1 of 2 turns", &["assistant: The result failed on run 3."])
    );
}

#[test]
fn review_session_json_gives_the_counts_the_material_and_the_rules() {
    let project = Project::new();
    project.write("session.md", SESSION);
    let text = project.stdout("anna", &["--json", "review-session", "session.md"]);
    let value: serde_json::Value = serde_json::from_str(&text).expect("the JSON of review-session");
    assert_eq!(value["schema"], "dossier.review-session.v1", "{text}");
    assert_eq!(value["ok"], true, "{text}");
    assert_eq!(value["data"]["turns"], 3, "{text}");
    assert_eq!(value["data"]["kept"], 2, "{text}");
    assert_eq!(
        value["data"]["material"],
        serde_json::json!(MATERIAL),
        "{text}"
    );
    assert_eq!(
        value["data"]["rules"].as_array().map(Vec::len),
        Some(RULES.len()),
        "{text}"
    );
}

#[test]
fn review_session_changes_nothing_on_disk() {
    let project = Project::new();
    project.write("session.md", SESSION);
    let before = tree(project.root());
    project.stdout("anna", &["review-session", "session.md"]);
    assert_eq!(tree(project.root()), before);
}

#[test]
fn review_session_runs_in_a_folder_without_a_dossier() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    fs::write(folder.path().join("session.md"), SESSION).expect("the session file");
    Command::cargo_bin("dossier")
        .expect("the dossier program is built")
        .current_dir(folder.path())
        .args(["review-session", "session.md"])
        .assert()
        .success()
        .stdout(contains("2 of 3 turns"))
        .stdout(contains("## Drafting rules"));
}

#[test]
fn review_session_reads_bad_utf8_bytes_lossily() {
    let project = Project::new();
    fs::write(
        project.root().join("lossy.md"),
        b"# Session\n\n## user\n\nWe decided on the model \xff here.\n".as_slice(),
    )
    .expect("the session file");
    let text = project.stdout("anna", &["review-session", "lossy.md"]);
    assert!(
        text.contains("user: We decided on the model \u{FFFD} here."),
        "{text}"
    );
}

#[test]
fn review_session_takes_a_path_with_spaces_and_non_ascii_characters() {
    let project = Project::new();
    project.write("Sitzung ü/session file.md", SESSION);
    let text = project.stdout("anna", &["review-session", "Sitzung ü/session file.md"]);
    assert_eq!(text, review_text("2 of 3 turns", &MATERIAL));
}
