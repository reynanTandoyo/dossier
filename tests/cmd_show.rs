//! Runs the built program for `show`: the file text as stored, the marks on a first line, the
//! `--json` data with every field, section and history line, and an unknown ID as exit 1. Each
//! test runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};
use std::fs;

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const LESSON_TITLE: &str = "Steady mixing-plane runs overpredict stall margin";
/// The statement after the change a test makes to the approved lesson by hand.
const CHANGED_STATEMENT: &str =
    "Steady runs with a mixing-plane interface underpredict the stall margin.";

/// The file text of the entry with `id`.
fn text_of(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// Records a lesson as `agent`: approved when a person writes it, a draft when an LLM does.
/// Returns the standard output of `learn`.
fn learn_by(project: &Project, agent: &str) -> String {
    project.stdout(
        agent,
        &[
            "learn",
            STATEMENT,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            EVIDENCE,
            "--confidence",
            "simulated",
            "--title",
            LESSON_TITLE,
        ],
    )
}

/// The data of `show <id> --json`, parsed. Asserts the envelope says it succeeded.
fn show_data(project: &Project, id: &str) -> Value {
    let stdout = project.stdout("anna", &["show", id, "--json"]);
    let envelope: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");
    assert_eq!(envelope["schema"], "dossier.show.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["warnings"], json!([]));
    envelope["data"].clone()
}

/// An invented finding written by hand, with the given status line and extra field lines.
fn hand_written_finding(id: &str, status: &str, extra: &str) -> String {
    format!(
        "<!-- dossier-format: 1 -->\n# {id} · An invented finding\n\n- kind: finding\n- status: {status}\n- created: 2026-11-20T10:00:00+01:00\n- author: anna\n{extra}- history: 2026-11-20T10:00:00+01:00 anna: created\n\n## Observation\n\nAn invented observation.\n"
    )
}

#[test]
fn an_approved_finding_prints_the_file_text_with_no_marks() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "found",
            "The oscillation stayed.",
            "--evidence",
            "Runs 12 and 13.",
        ],
    );

    let stored = text_of(&project, "FND-001");
    let stdout = project.stdout("anna", &["show", "FND-001"]);

    assert_eq!(stdout, stored);
    assert!(
        stdout.starts_with("<!-- dossier-format: 1 -->\n# FND-001 · The oscillation stayed.\n")
    );
}

#[test]
fn the_id_is_found_in_any_case_with_or_without_a_place_prefix_and_needs_no_name() {
    let project = Project::new();
    project.stdout("anna", &["found", "A finding."]);

    let stored = text_of(&project, "FND-001");
    for id in ["fnd-001", "P-001:FND-001", "p-001:fnd-001"] {
        assert_eq!(project.stdout("", &["show", id]), stored, "{id}");
    }
}

#[test]
fn a_draft_lesson_starts_with_the_draft_mark() {
    let project = Project::new();
    assert_eq!(
        learn_by(&project, "anna-claude"),
        "LES-001: added (draft, waiting for approval)\n"
    );

    let stdout = project.stdout("anna", &["show", "LES-001"]);

    assert_eq!(stdout, format!("[draft]\n{}", text_of(&project, "LES-001")));
    assert!(stdout.starts_with("[draft]\n<!-- dossier-format: 1 -->\n# LES-001 · "));
}

#[test]
fn json_gives_every_field_section_and_the_one_history_line_of_a_draft() {
    let project = Project::new();
    learn_by(&project, "anna-claude");

    let data = show_data(&project, "LES-001");

    assert_eq!(data["id"], "LES-001");
    assert_eq!(data["title"], LESSON_TITLE);
    assert_eq!(data["kind"], "lesson");
    assert_eq!(data["status"], "draft");
    assert_eq!(data["marks"], json!(["[draft]"]));
    assert_eq!(
        data["path"],
        "DOSSIER/lessons/LES-001-steady-mixing-plane-runs-overpredict-stall-margin.md"
    );
    assert_eq!(data["fields"]["author"], "anna");
    assert_eq!(data["fields"]["via"], "anna-claude");
    assert_eq!(data["fields"]["confidence"], "simulated");
    assert_eq!(data["fields"]["status"], "draft");
    assert_eq!(data["fields"]["created"], "2026-11-20T10:00:00+01:00");
    assert!(data["fields"]["approved"].is_null());
    assert_eq!(
        data["fields"]["history"],
        json!(["2026-11-20T10:00:00+01:00 anna via anna-claude: created"])
    );
    assert_eq!(data["sections"]["Statement"], STATEMENT);
    assert_eq!(data["sections"]["Valid for"], VALID_FOR);
    assert_eq!(data["sections"]["Evidence"], EVIDENCE);
}

#[test]
fn json_of_an_approved_finding_has_the_created_and_approved_history_lines() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "found",
            "The oscillation stayed.",
            "--evidence",
            "Runs 12 and 13.",
        ],
    );

    let data = show_data(&project, "FND-001");

    assert_eq!(data["kind"], "finding");
    assert_eq!(data["status"], "approved");
    assert_eq!(data["marks"], json!([]));
    assert_eq!(
        data["path"],
        "DOSSIER/findings/FND-001-the-oscillation-stayed.md"
    );
    assert_eq!(data["fields"]["author"], "anna");
    assert_eq!(
        data["fields"]["approved"],
        "2026-11-20T10:00:00+01:00 by anna"
    );
    assert_eq!(data["fields"]["valid-from"], "2026-11-20T10:00:00+01:00");
    assert_eq!(
        data["fields"]["history"],
        json!([
            "2026-11-20T10:00:00+01:00 anna: created",
            "2026-11-20T10:00:00+01:00 anna: approved",
        ])
    );
    assert_eq!(data["sections"]["Observation"], "The oscillation stayed.");
    assert_eq!(data["sections"]["Evidence"], "Runs 12 and 13.");
}

#[test]
fn an_approved_lesson_changed_by_hand_is_marked_changed_after_approval() {
    let project = Project::new();
    learn_by(&project, "anna");
    let original = text_of(&project, "LES-001");
    assert_eq!(project.stdout("anna", &["show", "LES-001"]), original);

    let changed = original.replace(
        "overpredict the stall margin",
        "underpredict the stall margin",
    );
    assert_ne!(changed, original);
    fs::write(project.entry_file("LES-001"), &changed).expect("the entry is changed by hand");

    let stdout = project.stdout("anna", &["show", "LES-001"]);
    assert_eq!(stdout, format!("[changed after approval]\n{changed}"));
    let data = show_data(&project, "LES-001");
    assert_eq!(data["marks"], json!(["[changed after approval]"]));
    assert_eq!(data["sections"]["Statement"], CHANGED_STATEMENT);
}

#[test]
fn the_status_marks_of_rejected_retired_and_superseded_entries() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-rejected.md",
        &hand_written_finding("FND-001", "rejected", ""),
    );
    project.write(
        "DOSSIER/findings/FND-002-retired.md",
        &hand_written_finding("FND-002", "retired", ""),
    );
    project.write(
        "DOSSIER/findings/FND-003-superseded.md",
        &hand_written_finding("FND-003", "superseded", "- superseded-by: DEC-002\n"),
    );

    for (id, mark) in [
        ("FND-001", "[rejected]"),
        ("FND-002", "[retired]"),
        ("FND-003", "[superseded by DEC-002]"),
    ] {
        let text = text_of(&project, id);
        assert_eq!(
            project.stdout("anna", &["show", id]),
            format!("{mark}\n{text}"),
            "{id}"
        );
    }
    let data = show_data(&project, "FND-003");
    assert_eq!(data["marks"], json!(["[superseded by DEC-002]"]));
    assert_eq!(data["fields"]["superseded-by"], "DEC-002");
}

#[test]
fn a_broken_file_elsewhere_is_a_warning_and_the_entry_still_shows() {
    let project = Project::new();
    project.stdout("anna", &["found", "A finding."]);
    project.write(
        "DOSSIER/decisions/DEC-009-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    project
        .run("anna", &["show", "FND-001"])
        .success()
        .stdout(contains("# FND-001 · A finding."))
        .stderr(contains("warning:"))
        .stderr(contains("DEC-009-broken.md"));
}

#[test]
fn an_unknown_id_is_exit_1_and_as_json_names_not_found() {
    let project = Project::new();
    project.stdout("anna", &["found", "A finding."]);

    project
        .run("anna", &["show", "DEC-999"])
        .code(1)
        .stderr(contains("error: no entry DEC-999"));
    project
        .run("anna", &["show", "DEC-999", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"NOT_FOUND\""));
}
