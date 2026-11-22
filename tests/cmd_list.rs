//! Runs the built program for `list`: approved entries by default, the filters, `--wide` with who
//! and when, the JSON data, the marks of entries that are not plainly approved, and a broken
//! entry file that gives a warning while the list still works. Each test runs in its own
//! temporary project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00
//! unless a test gives another time.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};
use std::fs;

/// Invented text for the approved finding and the draft lesson.
const OBSERVATION: &str = "The invented test rig stalls above a tip Mach number of 1.1.";
const FINDING_TITLE: &str = "Stall above Mach 1.1";
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const LESSON_TITLE: &str = "Mixing plane margin";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// The time of the lesson, a day after the clock of the other commands.
const LATER: &str = "2026-11-21T09:00:00+01:00";

/// `learn` arguments for a complete lesson with the invented texts above, followed by `extra`.
fn learn<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "learn",
        STATEMENT,
        "--valid-for",
        VALID_FOR,
        "--evidence",
        EVIDENCE,
        "--title",
        LESSON_TITLE,
    ];
    args.extend_from_slice(extra);
    args
}

/// A project with two entries: FND-001, an approved finding that anna wrote, and LES-001, a draft
/// lesson that anna-claude wrote on LATER (so its author is anna, through anna-claude).
fn project_with_two_entries() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                OBSERVATION,
                "--title",
                FINDING_TITLE,
                "--evidence",
                EVIDENCE,
            ],
        )
        .success();
    project
        .run_at("anna-claude", LATER, &learn(&["--confidence", "simulated"]))
        .success();
    project
}

/// The text of an entry file in the form of format.md §1, with an invented statement.
fn entry_text(heading: &str, kind: &str, status: &str, extra: &str) -> String {
    format!(
        "<!-- dossier-format: 1 -->\n# {heading}\n\n- kind: {kind}\n- status: {status}\n- created: 2026-11-10T09:00:00+01:00\n- author: anna\n{extra}\n## Statement\n\nAn invented statement written for a test.\n"
    )
}

#[test]
fn by_default_only_approved_entries_are_listed_and_the_count_follows() {
    let project = project_with_two_entries();

    assert_eq!(
        project.stdout("anna", &["list"]),
        "FND-001  finding  Stall above Mach 1.1\n1 entry\n"
    );
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "lesson"]),
        "0 entries\n"
    );
}

#[test]
fn drafts_adds_the_draft_and_the_filters_choose_the_entries() {
    let project = project_with_two_entries();

    assert_eq!(
        project.stdout("anna", &["list", "--drafts"]),
        "FND-001  finding  Stall above Mach 1.1\nLES-001  lesson  [draft] Mixing plane margin\n2 entries\n"
    );
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "lesson", "--all"]),
        "LES-001  lesson  [draft] Mixing plane margin\n1 entry\n"
    );
    assert_eq!(
        project.stdout("anna", &["list", "--author", "ben", "--all"]),
        "0 entries\n"
    );
}

#[test]
fn wide_shows_who_created_and_approved_each_entry_and_when() {
    let project = project_with_two_entries();

    assert_eq!(
        project.stdout("anna", &["list", "--drafts", "--wide"]),
        "FND-001  finding  Stall above Mach 1.1  (created 2026-11-20T10:00:00+01:00 by anna, approved 2026-11-20T10:00:00+01:00 by anna)\nLES-001  lesson  [draft] Mixing plane margin  (created 2026-11-21T09:00:00+01:00 by anna)\n2 entries\n"
    );
}

#[test]
fn json_gives_one_object_per_entry_with_null_where_a_value_is_missing() {
    let project = project_with_two_entries();

    let stdout = project.stdout("anna", &["list", "--drafts", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("the JSON parses");

    assert_eq!(value["schema"], "dossier.list.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "id": "FND-001",
                "kind": "finding",
                "status": "approved",
                "title": FINDING_TITLE,
                "author": "anna",
                "created": "2026-11-20T10:00:00+01:00",
                "approved": "2026-11-20T10:00:00+01:00 by anna"
            },
            {
                "id": "LES-001",
                "kind": "lesson",
                "status": "draft",
                "title": LESSON_TITLE,
                "author": "anna",
                "created": LATER,
                "approved": null
            }
        ])
    );
}

#[test]
fn a_broken_entry_file_gives_a_warning_and_the_list_still_works() {
    let project = project_with_two_entries();
    project.write(
        "DOSSIER/findings/FND-009-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        project.stdout("anna", &["list"]),
        "FND-001  finding  Stall above Mach 1.1\n1 entry\n"
    );
    project.run("anna", &["list"]).success().stderr(contains(
        "warning: DOSSIER/findings/FND-009-broken.md: line 2: no heading",
    ));
}

#[test]
fn a_mark_shows_a_status_other_than_approved_and_a_changed_approved_lesson() {
    let project = Project::new();
    project
        .run("anna", &learn(&["--confidence", "measured"]))
        .success();

    // A person edits the approved lesson by hand: its text no longer matches the approval.
    let lesson = project.entry_file("LES-001");
    let text = fs::read_to_string(&lesson).expect("the lesson file");
    assert!(text.contains(STATEMENT), "the statement is in the file");
    fs::write(
        &lesson,
        text.replace(
            STATEMENT,
            "An edited statement, changed after the approval.",
        ),
    )
    .expect("the lesson file is written");

    project.write(
        "DOSSIER/decisions/DEC-001-old-choice.md",
        &entry_text("DEC-001 · Old choice", "decision", "rejected", ""),
    );
    project.write(
        "DOSSIER/assumptions/ASM-001-old-guess.md",
        &entry_text("ASM-001 · Old guess", "assumption", "retired", ""),
    );
    project.write(
        "DOSSIER/findings/FND-002-old-reading.md",
        &entry_text(
            "FND-002 · Old reading",
            "finding",
            "superseded",
            "- superseded-by: DEC-001\n",
        ),
    );

    assert_eq!(
        project.stdout("anna", &["list", "--all"]),
        "ASM-001  assumption  [retired] Old guess\nDEC-001  decision  [rejected] Old choice\nLES-001  lesson  [changed after approval] Mixing plane margin\nFND-002  finding  [superseded by DEC-001] Old reading\n4 entries\n"
    );
}

#[test]
fn on_lists_the_decision_in_force_on_a_day_so_a_superseded_one_shows_only_before_it() {
    let project = Project::new();
    project
        .run_at(
            "anna",
            "2026-11-20T10:00:00+01:00",
            &[
                "decide",
                "Invented first choice of the rig speed.",
                "--reason",
                "An invented reason for the first choice.",
                "--title",
                "First choice",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-25T10:00:00+01:00",
            &[
                "decide",
                "Invented second choice of the rig speed.",
                "--reason",
                "An invented reason for the second choice.",
                "--title",
                "Second choice",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            "2026-11-25T10:00:00+01:00",
            &["supersede", "DEC-001", "--by", "DEC-002"],
        )
        .success();

    // Nothing was in force before the first decision.
    assert_eq!(
        project.stdout("anna", &["list", "--on", "2026-11-19"]),
        "0 entries\n"
    );
    // On the 22nd the first decision was in force and the second was not yet.
    assert_eq!(
        project.stdout("anna", &["list", "--on", "2026-11-22"]),
        "DEC-001  decision  [superseded by DEC-002] First choice\n1 entry\n"
    );
    // From the 25th on, the second decision is in force and the first one is not.
    assert_eq!(
        project.stdout("anna", &["list", "--on", "2026-11-26"]),
        "DEC-002  decision  Second choice\n1 entry\n"
    );
    // Without --on the list shows only what is approved now.
    assert_eq!(
        project.stdout("anna", &["list"]),
        "DEC-002  decision  Second choice\n1 entry\n"
    );
}

/// Invented text for the how-to and the tool of the purpose tests.
const HOWTO_TITLE: &str = "Export the rig mesh";
const HOWTO_PURPOSE: &str = "Export the mesh of an invented test rig for the solver.";
const HOWTO_STEPS: &str = "Open the invented rig file, then export the mesh.";
const HOWTO_TESTED: &str = "MeshTool 2.4.1 on an invented rig.";
const TOOL_TITLE: &str = "Count solver warnings";
const TOOL_PURPOSE: &str = "Counts the warnings in an invented solver log.";
const TOOL_TESTED: &str = "Python 3.12 on an invented log.";

/// anna writes a how-to and a tool with purpose and tested-with sections. She is a person, so
/// both are approved at once: HOW-001 and TOOL-001.
fn anna_writes_howto_and_tool(project: &Project) {
    let purpose = format!("Purpose={HOWTO_PURPOSE}");
    let steps = format!("Steps={HOWTO_STEPS}");
    let tested = format!("Tested with={HOWTO_TESTED}");
    project
        .run(
            "anna",
            &[
                "add",
                "howto",
                "--title",
                HOWTO_TITLE,
                "--field",
                "software=MeshTool 2.4",
                "--section",
                &purpose,
                "--section",
                &steps,
                "--section",
                &tested,
            ],
        )
        .success();
    let purpose = format!("Purpose={TOOL_PURPOSE}");
    let tested = format!("Tested with={TOOL_TESTED}");
    project
        .run(
            "anna",
            &[
                "add",
                "tool",
                "--title",
                TOOL_TITLE,
                "--field",
                "type=script",
                "--section",
                &purpose,
                "--section",
                &tested,
            ],
        )
        .success();
}

#[test]
fn a_howto_and_a_tool_show_their_purpose_and_the_versions_tested() {
    let project = Project::new();
    anna_writes_howto_and_tool(&project);

    assert_eq!(
        project.stdout("anna", &["list", "--kind", "howto"]),
        format!(
            "HOW-001  howto  {HOWTO_TITLE}  — {HOWTO_PURPOSE}  (tested with {HOWTO_TESTED})\n1 entry\n"
        )
    );
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "tool"]),
        format!(
            "TOOL-001  tool  {TOOL_TITLE}  — {TOOL_PURPOSE}  (tested with {TOOL_TESTED})\n1 entry\n"
        )
    );
}

#[test]
fn the_purpose_is_its_first_line_and_a_missing_or_empty_part_is_left_out() {
    let project = Project::new();
    // A how-to with a two-line purpose and an empty Tested with section.
    project.write(
        "DOSSIER/howtos/HOW-001-export-the-rig-mesh.md",
        "<!-- dossier-format: 1 -->\n# HOW-001 · Export the rig mesh\n\n- kind: howto\n- status: approved\n- created: 2026-11-10T09:00:00+01:00\n- author: anna\n- software: MeshTool 2.4\n\n## Purpose\n\nFirst line of the invented purpose.\nA second line that the list does not show.\n\n## Steps\n\nOpen the invented rig file.\n\n## Tested with\n\n",
    );
    // A tool without a Purpose section, with its Tested with section.
    project.write(
        "DOSSIER/tools/TOOL-001-count-solver-warnings.md",
        "<!-- dossier-format: 1 -->\n# TOOL-001 · Count solver warnings\n\n- kind: tool\n- status: approved\n- created: 2026-11-10T09:00:00+01:00\n- author: anna\n- type: script\n\n## Tested with\n\nPython 3.12 on an invented log.\n",
    );

    assert_eq!(
        project.stdout("anna", &["list"]),
        "HOW-001  howto  Export the rig mesh  — First line of the invented purpose.\nTOOL-001  tool  Count solver warnings  (tested with Python 3.12 on an invented log.)\n2 entries\n"
    );
}

#[test]
fn wide_puts_the_purpose_before_who_created_and_approved_the_how_to() {
    let project = Project::new();
    anna_writes_howto_and_tool(&project);

    assert_eq!(
        project.stdout("anna", &["list", "--kind", "howto", "--wide"]),
        format!(
            "HOW-001  howto  {HOWTO_TITLE}  — {HOWTO_PURPOSE}  (tested with {HOWTO_TESTED})  (created 2026-11-20T10:00:00+01:00 by anna, approved 2026-11-20T10:00:00+01:00 by anna)\n1 entry\n"
        )
    );
}
