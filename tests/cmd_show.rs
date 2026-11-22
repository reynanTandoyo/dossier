//! Runs the built program for `show`: the file text as stored, the marks on a first line, the
//! `--json` data with every field, section and history line, and an unknown ID as exit 1. An ID
//! with the place `team:` shows an entry of the team dossier of the project (R-ID-2): without a
//! team dossier it is exit 4, and its JSON `id` keeps the place. Each test runs in its own project
//! folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const LESSON_TITLE: &str = "Steady mixing-plane runs overpredict stall margin";
/// The statement after the change a test makes to the approved lesson by hand.
const CHANGED_STATEMENT: &str =
    "Steady runs with a mixing-plane interface underpredict the stall margin.";
/// Invented text: why a proposal is turned down in the test that declines one.
const DECLINE_REASON: &str = "An invented reason: the inlet profile was not checked.";
/// The time of that declining: a day after the fixed clock of `common::Project`.
const DECLINED_AT: &str = "2026-11-21T09:30:00+01:00";
/// The fixed clock of `common::Project`, and a later run a day after it: a review date of the
/// fixed day is due then.
const NOW: &str = "2026-11-20T10:00:00+01:00";
const NEXT_DAY: &str = "2026-11-21T09:00:00+01:00";
/// Invented text for the how-to and the tool of the tests that approve them by hand.
const HOWTO_TITLE: &str = "Invented solver setup";
const HOWTO_PURPOSE: &str = "An invented how-to for the test solver setup.";
const HOWTO_STEPS: &str = "1. Invented step one.";
const TOOL_TITLE: &str = "Invented rebuild script for the test table";
const TOOL_PURPOSE: &str = "An invented script that rebuilds the test table.";
/// The script of the tool tests, relative to the project folder, and the same script edited.
const SCRIPT_FILE: &str = "scripts/rebuild.py";
const SCRIPT: &str = "print('invented rebuild step')\n";
const SCRIPT_EDITED: &str = "print('invented rebuild step, edited by hand')\n";

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

/// The standard output of `show <id>` run as anna at the time `now`; the run must succeed.
fn shown_at(project: &Project, now: &str, id: &str) -> String {
    let output = project
        .run_at("anna", now, &["show", id])
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("the standard output is UTF-8")
}

/// The checksum of a file of the project, written as format.md §18 writes it.
fn checksum_of(project: &Project, relative: &str) -> String {
    dossier::filesum::checksum(&project.root().join(relative)).expect("the checksum of the file")
}

/// Adds an entry of `kind` as anna, as a draft, with the given fields and sections (`key=value`
/// and `Name=text`), then has a person approve it: the approval records its checksum, as for a
/// lesson, how-to or tool. Returns the ID, such as TOOL-001.
fn approved_by_hand_of(
    project: &Project,
    kind: &str,
    title: &str,
    fields: &[&str],
    sections: &[&str],
) -> String {
    let mut args = vec!["add", kind, "--title", title, "--draft"];
    for field in fields.iter().copied() {
        args.extend(["--field", field]);
    }
    for section in sections.iter().copied() {
        args.extend(["--section", section]);
    }
    let added = project.stdout("anna", &args);
    let id = added
        .split(':')
        .next()
        .expect("the first line starts with the ID")
        .to_string();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", &id])
        .assert()
        .success();
    id
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
fn a_proposal_starts_with_the_proposed_mark() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "propose",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );

    let text = text_of(&project, "DEC-001");
    assert_eq!(
        project.stdout("anna", &["show", "DEC-001"]),
        format!("[proposed]\n{text}")
    );
    let data = show_data(&project, "DEC-001");
    assert_eq!(data["status"], "proposed");
    assert_eq!(data["marks"], json!(["[proposed]"]));
}

#[test]
fn a_declined_proposal_starts_with_the_declined_mark() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
        ],
    );
    project
        .command("ben", DECLINED_AT)
        .env("DOSSIER_ANSWERS", "")
        .args(["decline", "DEC-001", "--reason", DECLINE_REASON])
        .assert()
        .success()
        .stdout("DEC-001: declined\n");

    let text = text_of(&project, "DEC-001");
    assert_eq!(
        project.stdout("anna", &["show", "DEC-001"]),
        format!("[declined]\n{text}")
    );
    let data = show_data(&project, "DEC-001");
    assert_eq!(data["status"], "declined");
    assert_eq!(data["marks"], json!(["[declined]"]));
}

#[test]
fn an_entry_resting_on_wrong_assumptions_shows_the_rests_on_mark() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-rests-on-two.md",
        &hand_written_finding("FND-001", "approved", "- rests-on: ASM-001, ASM-002\n"),
    );

    let text = text_of(&project, "FND-001");
    assert_eq!(
        project.stdout("anna", &["show", "FND-001"]),
        format!("[rests on ASM-001, ASM-002]\n{text}")
    );
    let data = show_data(&project, "FND-001");
    assert_eq!(data["marks"], json!(["[rests on ASM-001, ASM-002]"]));
    assert_eq!(data["fields"]["rests-on"], "ASM-001, ASM-002");
}

#[test]
fn a_proposal_resting_on_a_wrong_assumption_has_both_marks_in_order() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "propose",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
        ],
    );
    let original = text_of(&project, "DEC-001");
    let changed = original.replacen("- history: ", "- rests-on: ASM-001\n- history: ", 1);
    assert_ne!(changed, original);
    fs::write(project.entry_file("DEC-001"), &changed).expect("the entry is changed by hand");

    assert_eq!(
        project.stdout("anna", &["show", "DEC-001"]),
        format!("[proposed] [rests on ASM-001]\n{changed}")
    );
    let data = show_data(&project, "DEC-001");
    assert_eq!(data["marks"], json!(["[proposed]", "[rests on ASM-001]"]));
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

#[test]
fn an_entry_whose_review_date_is_before_today_is_due_for_review() {
    let project = Project::new();
    project.stdout("anna", &["found", "The oscillation stayed."]);
    project.stdout("anna", &["set", "FND-001", "--field", "review=2026-11-19"]);

    let text = text_of(&project, "FND-001");
    assert_eq!(
        project.stdout("anna", &["show", "FND-001"]),
        format!("[due for review]\n{text}")
    );
    let data = show_data(&project, "FND-001");
    assert_eq!(data["marks"], json!(["[due for review]"]));
    assert_eq!(data["fields"]["review"], "2026-11-19");
}

#[test]
fn a_review_date_of_today_or_later_is_not_due_for_review() {
    let project = Project::new();
    project.stdout("anna", &["found", "The oscillation stayed."]);
    project.stdout("anna", &["found", "The oscillation grew."]);
    project.stdout("anna", &["set", "FND-001", "--field", "review=2026-11-20"]);
    project.stdout("anna", &["set", "FND-002", "--field", "review=2026-12-01"]);

    for id in ["FND-001", "FND-002"] {
        let text = text_of(&project, id);
        assert_eq!(project.stdout("anna", &["show", id]), text, "{id}");
        assert_eq!(show_data(&project, id)["marks"], json!([]), "{id}");
    }
}

#[test]
fn the_review_date_is_compared_with_the_day_of_the_run() {
    let project = Project::new();
    project.stdout("anna", &["found", "The oscillation stayed."]);
    project.stdout("anna", &["set", "FND-001", "--field", "review=2026-11-20"]);

    let text = text_of(&project, "FND-001");
    assert_eq!(shown_at(&project, NOW, "FND-001"), text);
    assert_eq!(
        shown_at(&project, NEXT_DAY, "FND-001"),
        format!("[due for review]\n{text}")
    );
}

#[test]
fn a_review_value_that_is_not_a_date_gives_no_due_mark() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-soon.md",
        &hand_written_finding("FND-001", "approved", "- review: soon\n"),
    );

    let text = text_of(&project, "FND-001");
    assert_eq!(project.stdout("anna", &["show", "FND-001"]), text);
    assert_eq!(show_data(&project, "FND-001")["marks"], json!([]));
}

#[test]
fn an_imported_entry_shows_the_imported_mark_whatever_its_status() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-imported.md",
        &hand_written_finding("FND-001", "approved", "- imported: yes\n"),
    );
    project.write(
        "DOSSIER/findings/FND-002-imported-draft.md",
        &hand_written_finding("FND-002", "draft", "- imported: yes\n"),
    );
    project.write(
        "DOSSIER/findings/FND-003-not-imported.md",
        &hand_written_finding("FND-003", "approved", "- imported: no\n"),
    );

    let text = text_of(&project, "FND-001");
    assert_eq!(
        project.stdout("anna", &["show", "FND-001"]),
        format!("[imported]\n{text}")
    );
    assert_eq!(
        show_data(&project, "FND-001")["marks"],
        json!(["[imported]"])
    );
    let text = text_of(&project, "FND-002");
    assert_eq!(
        project.stdout("anna", &["show", "FND-002"]),
        format!("[draft] [imported]\n{text}")
    );
    assert_eq!(
        show_data(&project, "FND-002")["marks"],
        json!(["[draft]", "[imported]"])
    );
    let text = text_of(&project, "FND-003");
    assert_eq!(project.stdout("anna", &["show", "FND-003"]), text);
}

#[test]
fn due_for_review_imported_and_rests_on_marks_come_in_that_order() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-all-marks.md",
        &hand_written_finding(
            "FND-001",
            "approved",
            "- review: 2026-11-19\n- imported: yes\n- rests-on: ASM-001\n",
        ),
    );

    let text = text_of(&project, "FND-001");
    assert_eq!(
        project.stdout("anna", &["show", "FND-001"]),
        format!("[due for review] [imported] [rests on ASM-001]\n{text}")
    );
    let data = show_data(&project, "FND-001");
    assert_eq!(
        data["marks"],
        json!(["[due for review]", "[imported]", "[rests on ASM-001]"])
    );
}

#[test]
fn an_approved_how_to_changed_by_hand_is_changed_after_approval() {
    let project = Project::new();
    let id = approved_by_hand_of(
        &project,
        "howto",
        HOWTO_TITLE,
        &["software=invented-solver"],
        &[
            &format!("Purpose={HOWTO_PURPOSE}"),
            &format!("Steps={HOWTO_STEPS}"),
        ],
    );
    assert_eq!(id, "HOW-001");
    let original = text_of(&project, &id);
    assert_eq!(project.stdout("anna", &["show", &id]), original);

    let changed = original.replace(HOWTO_PURPOSE, "An invented how-to, changed by hand.");
    assert_ne!(changed, original);
    fs::write(project.entry_file(&id), &changed).expect("the entry is changed by hand");

    assert_eq!(
        project.stdout("anna", &["show", &id]),
        format!("[changed after approval]\n{changed}")
    );
    assert_eq!(
        show_data(&project, &id)["marks"],
        json!(["[changed after approval]"])
    );
}

#[test]
fn an_approved_tool_changed_by_hand_is_changed_after_approval() {
    let project = Project::new();
    let id = approved_by_hand_of(
        &project,
        "tool",
        TOOL_TITLE,
        &["type=command", "location=python"],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    assert_eq!(id, "TOOL-001");
    let original = text_of(&project, &id);
    assert_eq!(project.stdout("anna", &["show", &id]), original);

    let changed = original.replace(TOOL_PURPOSE, "An invented tool, changed by hand.");
    assert_ne!(changed, original);
    fs::write(project.entry_file(&id), &changed).expect("the entry is changed by hand");

    assert_eq!(
        project.stdout("anna", &["show", &id]),
        format!("[changed after approval]\n{changed}")
    );
}

#[test]
fn an_approved_finding_changed_by_hand_is_not_marked_changed_after_approval() {
    let project = Project::new();
    project.stdout("anna", &["found", "The oscillation stayed."]);
    let original = text_of(&project, "FND-001");
    let changed = original.replace("The oscillation stayed.", "The oscillation grew.");
    assert_ne!(changed, original);
    fs::write(project.entry_file("FND-001"), &changed).expect("the entry is changed by hand");

    assert_eq!(project.stdout("anna", &["show", "FND-001"]), changed);
    assert_eq!(show_data(&project, "FND-001")["marks"], json!([]));
}

#[test]
fn a_tool_whose_script_changed_since_approval_shows_the_file_changed_mark() {
    let project = Project::new();
    project.write(SCRIPT_FILE, SCRIPT);
    let checksum = checksum_of(&project, SCRIPT_FILE);
    let location = format!("location={SCRIPT_FILE}");
    let recorded = format!("checksum={checksum}");
    let id = approved_by_hand_of(
        &project,
        "tool",
        TOOL_TITLE,
        &["type=script", &location, &recorded],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    assert_eq!(id, "TOOL-001");
    let text = text_of(&project, &id);
    assert_eq!(project.stdout("anna", &["show", &id]), text);

    project.write(SCRIPT_FILE, SCRIPT_EDITED);

    assert_eq!(
        project.stdout("anna", &["show", &id]),
        format!("[file changed since approval]\n{text}")
    );
    let data = show_data(&project, &id);
    assert_eq!(data["marks"], json!(["[file changed since approval]"]));
    assert_eq!(data["fields"]["checksum"], checksum);
    assert_eq!(data["fields"]["location"], SCRIPT_FILE);
}

#[test]
fn a_script_given_by_an_absolute_location_is_checked_too() {
    let project = Project::new();
    project.write(SCRIPT_FILE, SCRIPT);
    let checksum = checksum_of(&project, SCRIPT_FILE);
    let absolute = project.root().join(SCRIPT_FILE);
    let location = format!("location={}", absolute.display());
    let recorded = format!("checksum={checksum}");
    let id = approved_by_hand_of(
        &project,
        "tool",
        TOOL_TITLE,
        &["type=script", &location, &recorded],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    let text = text_of(&project, &id);
    assert_eq!(project.stdout("anna", &["show", &id]), text);

    project.write(SCRIPT_FILE, SCRIPT_EDITED);

    assert_eq!(
        project.stdout("anna", &["show", &id]),
        format!("[file changed since approval]\n{text}")
    );
}

#[test]
fn a_location_that_is_no_file_or_a_tool_without_checksum_gives_no_file_mark() {
    let project = Project::new();
    project.write(SCRIPT_FILE, SCRIPT);
    let wrong = format!("sha256:{}", "0".repeat(64));
    let wrong_checksum = format!("checksum={wrong}");
    // A command name and a URL are not files; the checksum of a missing file cannot differ.
    let command = approved_by_hand_of(
        &project,
        "tool",
        "Invented command tool",
        &["type=command", "location=python", &wrong_checksum],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    let url = approved_by_hand_of(
        &project,
        "tool",
        "Invented web tool",
        &[
            "type=repo",
            "location=https://example.invalid/tool",
            &wrong_checksum,
        ],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    // A script with no checksum recorded has nothing to differ from.
    let unrecorded = approved_by_hand_of(
        &project,
        "tool",
        "Invented script without checksum",
        &["type=script", &format!("location={SCRIPT_FILE}")],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    project.write(SCRIPT_FILE, SCRIPT_EDITED);

    for id in [command, url, unrecorded] {
        let text = text_of(&project, &id);
        assert_eq!(project.stdout("anna", &["show", &id]), text, "{id}");
        assert_eq!(show_data(&project, &id)["marks"], json!([]), "{id}");
    }
}

#[test]
fn a_tool_changed_by_hand_and_with_a_changed_script_has_both_marks_in_order() {
    let project = Project::new();
    project.write(SCRIPT_FILE, SCRIPT);
    let checksum = checksum_of(&project, SCRIPT_FILE);
    let location = format!("location={SCRIPT_FILE}");
    let recorded = format!("checksum={checksum}");
    let id = approved_by_hand_of(
        &project,
        "tool",
        TOOL_TITLE,
        &["type=script", &location, &recorded],
        &[&format!("Purpose={TOOL_PURPOSE}")],
    );
    let original = text_of(&project, &id);
    let changed = original.replace(TOOL_PURPOSE, "An invented script, changed by hand.");
    fs::write(project.entry_file(&id), &changed).expect("the entry is changed by hand");
    project.write(SCRIPT_FILE, SCRIPT_EDITED);

    assert_eq!(
        project.stdout("anna", &["show", &id]),
        format!("[changed after approval] [file changed since approval]\n{changed}")
    );
    assert_eq!(
        show_data(&project, &id)["marks"],
        json!(["[changed after approval]", "[file changed since approval]"])
    );
}

/// Invented text of the team's lesson: the project's own lessons do not share it.
const TEAM_STATEMENT: &str = "An invented team lesson: the inlet profile sets the stall margin.";
const TEAM_TITLE: &str = "Invented team lesson on the inlet profile";
/// The file of the team lesson `LES-001`, relative to the team folder.
const TEAM_LESSON_FILE: &str =
    "DOSSIER/lessons/LES-001-invented-team-lesson-on-the-inlet-profile.md";

/// A team dossier made by `init --team-dossier --code TEAM` in a folder of its own, whose name has
/// a space and an umlaut, run as anna, with anna as its member. Returns the folder (it keeps the
/// folder alive) and the DOSSIER folder in it.
fn team_dossier(project: &Project) -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    (folder, dossier)
}

/// Makes the project link to the team dossier at `team`, as `team use` does.
fn link(project: &Project, team: &Path) {
    project.stdout("anna", &["team", "use", &team.display().to_string()]);
}

/// Gives the project one confidential term, so that a team write from it passes the check of
/// R-CONF-2. The term is used by no other test text.
fn with_confidential_term(project: &Project) {
    project.stdout("anna", &["confidential", "add", "NT-999"]);
}

/// Records the team's lesson with `learn --team` as `agent`. Returns the standard output of `learn`.
fn team_lesson_by(project: &Project, agent: &str) -> String {
    project.stdout(
        agent,
        &[
            "learn",
            TEAM_STATEMENT,
            "--valid-for",
            VALID_FOR,
            "--evidence",
            EVIDENCE,
            "--confidence",
            "simulated",
            "--title",
            TEAM_TITLE,
            "--team",
        ],
    )
}

/// The file text of the entry `id` in the team dossier at `dossier`.
fn team_text(dossier: &Path, id: &str) -> String {
    let item = dossier::store::get(dossier, id).expect("the team entry");
    String::from_utf8(item.raw).expect("the entry file is UTF-8")
}

#[test]
fn a_team_lesson_is_shown_from_the_team_dossier_with_its_place_in_json() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    with_confidential_term(&project);
    assert_eq!(
        team_lesson_by(&project, "anna"),
        "LES-001: added (approved)\n"
    );

    let stored = team_text(&team, "LES-001");
    assert!(stored.starts_with("<!-- dossier-format: 1 -->\n# LES-001 · "));
    assert_eq!(project.stdout("anna", &["show", "team:LES-001"]), stored);

    let data = show_data(&project, "team:LES-001");
    assert_eq!(data["id"], "team:LES-001");
    assert_eq!(data["title"], TEAM_TITLE);
    assert_eq!(data["kind"], "lesson");
    assert_eq!(data["status"], "approved");
    assert_eq!(data["marks"], json!([]));
    assert_eq!(data["path"], TEAM_LESSON_FILE);
    assert_eq!(data["sections"]["Statement"], TEAM_STATEMENT);
}

#[test]
fn the_team_place_is_found_in_any_case_and_a_project_entry_with_the_same_number_stays_its_own() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    with_confidential_term(&project);
    learn_by(&project, "anna");
    team_lesson_by(&project, "anna");

    assert_eq!(
        project.stdout("anna", &["show", "LES-001"]),
        text_of(&project, "LES-001")
    );
    assert_eq!(
        show_data(&project, "LES-001")["sections"]["Statement"],
        STATEMENT
    );

    let stored = team_text(&team, "LES-001");
    for id in ["team:LES-001", "TEAM:les-001", "Team:LES-001"] {
        assert_eq!(project.stdout("anna", &["show", id]), stored, "{id}");
    }
    assert_eq!(
        show_data(&project, "team:LES-001")["sections"]["Statement"],
        TEAM_STATEMENT
    );
}

#[test]
fn a_team_draft_shows_the_draft_mark_and_the_project_holds_no_such_entry() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    with_confidential_term(&project);
    assert_eq!(
        team_lesson_by(&project, "anna-claude"),
        "LES-001: added (draft, waiting for approval)\n"
    );

    let stored = team_text(&team, "LES-001");
    assert_eq!(
        project.stdout("anna", &["show", "team:LES-001"]),
        format!("[draft]\n{stored}")
    );
    assert_eq!(
        show_data(&project, "team:LES-001")["marks"],
        json!(["[draft]"])
    );
    project
        .run("anna", &["show", "LES-001"])
        .code(1)
        .stderr(contains("no entry LES-001"));
}

#[test]
fn a_team_lesson_changed_by_hand_is_marked_from_the_team_dossier() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    with_confidential_term(&project);
    team_lesson_by(&project, "anna");

    let original = team_text(&team, "LES-001");
    let changed = original.replace("sets the stall margin", "does not set the stall margin");
    assert_ne!(changed, original);
    let file = dossier::store::get(&team, "LES-001")
        .expect("the team lesson")
        .path;
    fs::write(&file, &changed).expect("the team lesson is changed by hand");

    assert_eq!(
        project.stdout("anna", &["show", "team:LES-001"]),
        format!("[changed after approval]\n{changed}")
    );
    assert_eq!(
        show_data(&project, "team:LES-001")["marks"],
        json!(["[changed after approval]"])
    );
}

#[test]
fn a_team_id_in_a_project_without_a_team_is_no_dossier_and_says_how_to_link_one() {
    let project = Project::new();
    project
        .run("anna", &["show", "team:LES-001"])
        .code(4)
        .stderr(contains("dossier team use <path>"));
    project
        .run("anna", &["show", "team:LES-001", "--json"])
        .code(4)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"NO_DOSSIER\""));
}

#[test]
fn a_team_that_cannot_be_reached_is_no_dossier_naming_its_path() {
    let project = Project::new();
    let missing = project.root().join("NOWHERE").join("DOSSIER");
    let stored = missing.display().to_string().replace('\\', "/");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("set the team path");

    project
        .run("anna", &["show", "team:LES-001"])
        .code(4)
        .stderr(contains("the team dossier cannot be reached"))
        .stderr(contains("NOWHERE"));
}

#[test]
fn a_team_id_the_team_does_not_hold_is_not_found() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);

    project
        .run("anna", &["show", "team:LES-999"])
        .code(1)
        .stderr(contains("no entry LES-999"));
    project
        .run("anna", &["show", "team:LES-999", "--json"])
        .code(1)
        .stdout(contains("\"code\":\"NOT_FOUND\""));
}

#[test]
fn a_broken_file_in_the_team_dossier_is_a_warning_with_its_path_from_the_team_folder() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    with_confidential_term(&project);
    team_lesson_by(&project, "anna");
    let broken = team.join("decisions").join("DEC-009-broken.md");
    fs::create_dir_all(broken.parent().expect("the decisions folder")).expect("the folder");
    fs::write(
        &broken,
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    )
    .expect("the broken file");

    project
        .run("anna", &["show", "team:LES-001"])
        .success()
        .stdout(contains("# LES-001 · "))
        .stderr(contains("warning:"))
        .stderr(contains(
            "DOSSIER/decisions/DEC-009-broken.md: line 2: no heading",
        ));
}
