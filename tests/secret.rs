//! Runs the built program for R-CONF-7: text that looks like a credential is refused on every write
//! of a Markdown file. A person at a terminal can allow a false alarm with `--allow-credential-like`,
//! which is journaled; an LLM cannot. Each test runs in its own temporary project with the clock
//! fixed by `common::Project` (2026-11-20T10:00:00+01:00). The values of the tests are invented.

mod common;

use common::Project;
use dossier::entry::Entry;
use dossier::hash::sha256_hex;
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// The fixed clock of `common::Project`, as it appears in journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// An invented password line. Its value must never be printed.
const LINE: &str = "password: hunter2secret";
const VALUE: &str = "hunter2secret";

/// The body of an invented access token (`ghp_` in front of it). It must never be printed.
const TOKEN_BODY: &str = "invented0token0example";

/// Whether a file under DOSSIER (the state folder included) holds `text`. A file that is not
/// UTF-8 text is skipped.
fn dossier_holds(project: &Project, text: &str) -> bool {
    fn walk(folder: &Path, text: &str) -> bool {
        let Ok(entries) = fs::read_dir(folder) else {
            return false;
        };
        entries.flatten().any(|entry| {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, text)
            } else {
                fs::read_to_string(&path).is_ok_and(|content| content.contains(text))
            }
        })
    }
    walk(&project.root().join("DOSSIER"), text)
}

/// The names of the files in DOSSIER/findings; none when the folder does not exist.
fn finding_files(project: &Project) -> Vec<String> {
    let folder = project.root().join("DOSSIER").join("findings");
    fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Records the finding with the password line as anna at a terminal, with the flag, so that the
/// line is allowed. Returns the project.
fn allowed_finding() -> Project {
    let project = Project::new();
    project
        .command("anna", WHEN)
        .env("DOSSIER_ANSWERS", "")
        .args(["found", LINE, "--allow-credential-like"])
        .assert()
        .success();
    project
}

#[test]
fn a_found_with_a_password_line_is_refused_and_names_the_line_but_not_the_value() {
    let project = Project::new();
    project
        .run("anna", &["found", LINE])
        .failure()
        .code(1)
        .stderr(
            contains("FND-001: line")
                .and(contains("(password)"))
                .and(contains(VALUE).not()),
        );

    assert!(finding_files(&project).is_empty(), "no finding file");
    assert!(
        !dossier_holds(&project, VALUE),
        "the value is written nowhere"
    );
    assert!(
        project
            .entry_journal()
            .iter()
            .all(|record| !record.id.starts_with("FND-")),
        "nothing reaches the journal"
    );
}

#[test]
fn a_dry_run_with_a_password_line_is_refused_too_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna", &["found", LINE, "--dry-run"])
        .failure()
        .code(1)
        .stderr(contains("(password)").and(contains(VALUE).not()));

    assert!(finding_files(&project).is_empty(), "no finding file");
    assert!(
        !dossier_holds(&project, VALUE),
        "the value is written nowhere"
    );
}

#[test]
fn a_person_at_a_terminal_can_allow_the_line_and_the_allowing_is_journaled() {
    let project = Project::new();
    let stdout = project
        .command("anna", WHEN)
        .env("DOSSIER_ANSWERS", "")
        .args(["found", LINE, "--allow-credential-like"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(stdout).expect("the standard output is UTF-8"),
        "FND-001: added (approved)\n"
    );

    let text = fs::read_to_string(project.entry_file("FND-001")).expect("the finding is written");
    assert!(text.contains(LINE));
    let hits = dossier::secret::find(&text);
    assert!(!hits.is_empty(), "the finding holds credential-like lines");
    let lines: Vec<String> = hits.iter().map(|hit| hit.line.to_string()).collect();

    let allowed = fs::read_to_string(
        project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("credential-ok.json"),
    )
    .expect("the allowed lines are kept");
    assert!(allowed.contains(&sha256_hex(LINE.as_bytes())));
    assert!(!allowed.contains(VALUE), "only hashes are kept");

    let allowed_records: Vec<_> = project
        .entry_journal()
        .into_iter()
        .filter(|record| record.event == "allowed")
        .collect();
    assert_eq!(allowed_records.len(), 1);
    let record = &allowed_records[0];
    assert_eq!(record.id, "FND-001");
    assert_eq!(record.time, WHEN);
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.command, "found");
    assert_eq!(
        record.detail,
        format!("credential-like text, line {}", lines.join(", "))
    );
}

#[test]
fn an_llm_with_the_flag_gets_needs_person_in_json_and_writes_nothing() {
    let project = Project::new();
    let stdout = project
        .command("anna - claude", WHEN)
        .env("DOSSIER_ANSWERS", "")
        .args(["--json", "found", LINE, "--allow-credential-like"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&stdout).expect("the JSON output parses");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "NEEDS_PERSON");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("anna - claude is an LLM")),
        "{value}"
    );

    assert!(finding_files(&project).is_empty(), "no finding file");
    assert!(
        !dossier_holds(&project, VALUE),
        "the value is written nowhere"
    );
    assert!(
        !project
            .root()
            .join("DOSSIER")
            .join(".state")
            .join("credential-ok.json")
            .exists(),
        "nothing is allowed"
    );
}

#[test]
fn with_the_flag_but_no_terminal_the_line_is_refused() {
    let project = Project::new();
    project
        .command("anna", WHEN)
        .env_remove("DOSSIER_ANSWERS")
        .args(["found", LINE, "--allow-credential-like"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));

    assert!(finding_files(&project).is_empty(), "no finding file");
    assert!(
        !dossier_holds(&project, VALUE),
        "the value is written nowhere"
    );
}

#[test]
fn once_allowed_the_same_line_is_written_again_without_the_flag() {
    // The default title repeats the ID, so it is a new line; a title of its own keeps the
    // observation, the allowed line, the only one that is repeated.
    let project = allowed_finding();
    let stdout = project.stdout("anna", &["found", LINE, "--title", "Login note"]);
    assert_eq!(stdout, "FND-002: added (approved)\n");
    assert!(
        dossier_holds(&project, VALUE),
        "the allowed line is written"
    );
}

#[test]
fn after_allowing_a_change_that_keeps_the_line_still_works() {
    let project = allowed_finding();
    let stdout = project.stdout("anna", &["set", "FND-001", "--field", "tags=x"]);
    assert_eq!(stdout, "FND-001: changed\n");

    let entry = Entry::parse(
        &fs::read_to_string(project.entry_file("FND-001")).expect("the finding is read"),
    )
    .expect("the finding parses");
    assert_eq!(entry.field("tags").as_deref(), Some("x"));
    assert!(
        fs::read_to_string(project.entry_file("FND-001"))
            .expect("the finding is read")
            .contains(LINE)
    );
}

#[test]
fn a_value_with_a_token_in_its_note_is_refused_and_nothing_is_written() {
    let project = Project::new();
    let note = format!("token: ghp_{TOKEN_BODY}");
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.massflow",
                "12.4 kg/s",
                "--note",
                note.as_str(),
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("access token").and(contains(TOKEN_BODY).not()));

    assert!(
        !dossier_holds(&project, TOKEN_BODY),
        "the token is written nowhere"
    );
}

#[test]
fn an_attach_note_with_a_token_is_refused_and_the_file_list_is_not_written() {
    let project = Project::new();
    project.write("models/wing-v3.step", "invented model data\n");
    let note = format!("token: ghp_{TOKEN_BODY}");
    project
        .run(
            "anna",
            &[
                "attach",
                "T-001",
                "models/wing-v3.step",
                "--role",
                "model",
                "--note",
                note.as_str(),
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("FILES-T-001: line").and(contains(TOKEN_BODY).not()));

    assert!(
        !project
            .root()
            .join("DOSSIER")
            .join("files")
            .join("T-001.md")
            .exists(),
        "no file list"
    );
    assert!(
        !dossier_holds(&project, TOKEN_BODY),
        "the token is written nowhere"
    );
}
