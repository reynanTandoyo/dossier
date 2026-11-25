//! Runs the built program for `list`: approved entries by default, the filters, `--wide` with who
//! and when, the JSON data, the marks of entries that are not plainly approved (and `not tested
//! with` on a how-to, as `show` marks it), the state of an investigation (`[open]`, `[solved]` or
//! `[given up]`), its `--state` filter, the `--topic` filter of list and search, and a broken
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

/// The invented script of the tool test, its place in the project folder, and the same script
/// edited by hand.
const SCRIPT_FILE: &str = "scripts/count.py";
const SCRIPT: &str = "print('invented count')\n";
const SCRIPT_EDITED: &str = "print('invented count, edited by hand')\n";

#[test]
fn the_marks_come_in_the_order_show_gives_them() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-all-marks.md",
        &entry_text(
            "FND-001 · All marks",
            "finding",
            "approved",
            "- review: 2026-11-19\n- imported: yes\n- rests-on: ASM-001\n",
        ),
    );
    project.write(
        "DOSSIER/findings/FND-002-imported-draft.md",
        &entry_text(
            "FND-002 · Imported draft",
            "finding",
            "draft",
            "- imported: yes\n",
        ),
    );

    assert_eq!(
        project.stdout("anna", &["list", "--all"]),
        "FND-001  finding  [due for review] [imported] [rests on ASM-001] All marks\nFND-002  finding  [draft] [imported] Imported draft\n2 entries\n"
    );
}

#[test]
fn a_review_date_before_the_day_of_the_run_is_due_for_review() {
    let project = Project::new();
    project
        .run("anna", &["found", OBSERVATION, "--title", FINDING_TITLE])
        .success();
    project
        .run("anna", &["set", "FND-001", "--field", "review=2026-11-20"])
        .success();

    // On the day of the review date the entry is not yet due.
    assert_eq!(
        project.stdout("anna", &["list"]),
        "FND-001  finding  Stall above Mach 1.1\n1 entry\n"
    );
    // The next day it is.
    project
        .run_at("anna", LATER, &["list"])
        .success()
        .stdout("FND-001  finding  [due for review] Stall above Mach 1.1\n1 entry\n");
}

#[test]
fn a_how_to_and_a_tool_changed_by_hand_show_the_changed_mark() {
    let project = Project::new();
    anna_writes_howto_and_tool(&project);
    let howto = project.entry_file("HOW-001");
    let text = fs::read_to_string(&howto).expect("the how-to file");
    let changed = text.replace(HOWTO_STEPS, "Changed by hand: export the mesh twice.");
    assert_ne!(changed, text, "the steps are in the file");
    fs::write(&howto, changed).expect("the how-to is changed by hand");

    assert_eq!(
        project.stdout("anna", &["list", "--kind", "howto"]),
        format!(
            "HOW-001  howto  [changed after approval] {HOWTO_TITLE}  — {HOWTO_PURPOSE}  (tested with {HOWTO_TESTED})\n1 entry\n"
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
fn a_tool_whose_script_changed_since_approval_shows_the_file_changed_mark() {
    let project = Project::new();
    project.write(SCRIPT_FILE, SCRIPT);
    let checksum = dossier::filesum::checksum(&project.root().join(SCRIPT_FILE))
        .expect("the checksum of the script");
    let location = format!("location={SCRIPT_FILE}");
    let recorded = format!("checksum={checksum}");
    let purpose = format!("Purpose={TOOL_PURPOSE}");
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
                "--field",
                &location,
                "--field",
                &recorded,
                "--section",
                &purpose,
            ],
        )
        .success();
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "tool"]),
        format!("TOOL-001  tool  {TOOL_TITLE}  — {TOOL_PURPOSE}\n1 entry\n")
    );

    project.write(SCRIPT_FILE, SCRIPT_EDITED);
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "tool"]),
        format!(
            "TOOL-001  tool  [file changed since approval] {TOOL_TITLE}  — {TOOL_PURPOSE}\n1 entry\n"
        )
    );

    // A hand change to the tool's own text as well: both marks, in the order of show.
    let path = project.entry_file("TOOL-001");
    let text = fs::read_to_string(&path).expect("the tool file");
    let changed = text.replace(TOOL_PURPOSE, "An invented script, changed by hand.");
    assert_ne!(changed, text, "the purpose is in the file");
    fs::write(&path, changed).expect("the tool is changed by hand");
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "tool"]),
        format!(
            "TOOL-001  tool  [changed after approval] [file changed since approval] {TOOL_TITLE}  — An invented script, changed by hand.\n1 entry\n"
        )
    );
}

/// Invented text of the how-to of the `not tested with` tests: a how-to for the invented software
/// FlowSim. Its body is written to FLOW_FILE of the project folder, which `howto --from` reads.
const FLOW_TITLE: &str = "Invented flow setup";
const FLOW_PURPOSE: &str = "An invented how-to for the test software FlowSim.";
const FLOW_BODY: &str = "## Purpose\nAn invented how-to for the test software FlowSim.\n## Steps\n1. Invented step one.\n";
const FLOW_FILE: &str = "flow.md";

/// A how-to for FlowSim written as `agent`, with a `Tested with` section made of `tested` when it
/// is not empty. Returns its ID, such as HOW-001.
fn flowsim_howto(project: &Project, agent: &str, tested: &str) -> String {
    let mut body = FLOW_BODY.to_string();
    if !tested.is_empty() {
        body.push_str(&format!("## Tested with\n{tested}\n"));
    }
    project.write(FLOW_FILE, &body);
    let added = project.stdout(
        agent,
        &[
            "howto",
            "--software",
            "FlowSim",
            "--title",
            FLOW_TITLE,
            "--from",
            FLOW_FILE,
        ],
    );
    added
        .split(':')
        .next()
        .expect("the first line starts with the ID")
        .to_string()
}

/// Records a run of `software` in `version` as anna at the fixed clock: a usage record for the
/// task T-001, with no files.
fn run_of(project: &Project, software: &str, version: &str) {
    project
        .run(
            "anna",
            &["used", software, "--version", version, "--task", "T-001"],
        )
        .success();
}

#[test]
fn a_how_to_tested_with_an_older_version_than_its_software_was_used_in_is_marked_in_list() {
    let project = Project::new();
    let id = flowsim_howto(&project, "anna", "FlowSim 2410");
    run_of(&project, "FlowSim", "2502");

    // The usage record is listed too, and it gets no mark: only a how-to can.
    assert_eq!(
        project.stdout("anna", &["list"]),
        format!(
            "{id}  howto  [not tested with 2502] {FLOW_TITLE}  — {FLOW_PURPOSE}  (tested with FlowSim 2410)\nUSE-001  usage  FlowSim 2502 run for T-001\n2 entries\n"
        )
    );
}

#[test]
fn a_how_to_tested_with_the_version_it_was_used_in_is_not_marked_in_list() {
    let project = Project::new();
    let id = flowsim_howto(&project, "anna", "FlowSim 2502");
    run_of(&project, "FlowSim", "2502");

    assert_eq!(
        project.stdout("anna", &["list"]),
        format!(
            "{id}  howto  {FLOW_TITLE}  — {FLOW_PURPOSE}  (tested with FlowSim 2502)\nUSE-001  usage  FlowSim 2502 run for T-001\n2 entries\n"
        )
    );
}

#[test]
fn a_draft_how_to_gets_the_not_tested_mark_after_the_draft_mark_as_in_show() {
    let project = Project::new();
    let id = flowsim_howto(&project, "anna-claude", "FlowSim 2410");
    assert_eq!(id, "HOW-001");
    run_of(&project, "FlowSim", "2502");

    let shown = project.stdout("anna", &["show", &id]);
    assert_eq!(shown.lines().next(), Some("[draft] [not tested with 2502]"));
    assert_eq!(
        project.stdout("anna", &["list", "--drafts"]),
        format!(
            "{id}  howto  [draft] [not tested with 2502] {FLOW_TITLE}  — {FLOW_PURPOSE}  (tested with FlowSim 2410)\nUSE-001  usage  FlowSim 2502 run for T-001\n2 entries\n"
        )
    );
}

/// Invented problems and titles of the investigations of the state tests: INV-001 is solved,
/// INV-002 stays open and INV-003 is given up.
const PROBLEM_ONE: &str = "The density residual grows from iteration 40 on.";
const TITLE_ONE: &str = "Residual grows after iteration 40";
const PROBLEM_TWO: &str = "The mesh export fails on an invented rig.";
const TITLE_TWO: &str = "Mesh export fails";
const PROBLEM_THREE: &str = "The invented flow solver stops at a sonic inlet.";
const TITLE_THREE: &str = "Solver stops at sonic inlet";
const SOLUTION: &str = "The outlet flux had the wrong sign.";
const REASON: &str = "out of time";

/// Opens the investigation `id` as anna with `problem` and `title`; its first line is `<id>:
/// added (open)`.
fn open_investigation(project: &Project, id: &str, problem: &str, title: &str) {
    let stdout = project.stdout("anna", &["problem", problem, "--title", title]);
    assert_eq!(
        stdout.lines().next(),
        Some(format!("{id}: added (open)").as_str())
    );
}

#[test]
fn an_open_investigation_is_marked_open_and_a_solved_one_is_marked_solved() {
    let project = Project::new();
    open_investigation(&project, "INV-001", PROBLEM_ONE, TITLE_ONE);

    assert_eq!(
        project.stdout("anna", &["list"]),
        format!("INV-001  investigation  [open] {TITLE_ONE}\n1 entry\n")
    );

    assert_eq!(
        project.stdout("anna", &["solved", "INV-001", SOLUTION]),
        "INV-001: solved\n"
    );
    assert_eq!(
        project.stdout("anna", &["list"]),
        format!("INV-001  investigation  [solved] {TITLE_ONE}\n1 entry\n")
    );
}

#[test]
fn state_open_lists_only_the_open_investigation_and_each_kind_of_state_is_marked() {
    let project = Project::new();
    open_investigation(&project, "INV-001", PROBLEM_ONE, TITLE_ONE);
    open_investigation(&project, "INV-002", PROBLEM_TWO, TITLE_TWO);
    open_investigation(&project, "INV-003", PROBLEM_THREE, TITLE_THREE);
    assert_eq!(
        project.stdout("anna", &["solved", "INV-001", SOLUTION]),
        "INV-001: solved\n"
    );
    assert_eq!(
        project.stdout("anna", &["gave-up", "INV-003", "--reason", REASON]),
        "INV-003: given up\n"
    );

    assert_eq!(
        project.stdout(
            "anna",
            &["list", "--kind", "investigation", "--state", "open"]
        ),
        format!("INV-002  investigation  [open] {TITLE_TWO}\n1 entry\n")
    );
    assert_eq!(
        project.stdout("anna", &["list", "--kind", "investigation"]),
        format!(
            "INV-001  investigation  [solved] {TITLE_ONE}\nINV-002  investigation  [open] {TITLE_TWO}\nINV-003  investigation  [given up] {TITLE_THREE}\n3 entries\n"
        )
    );
    assert_eq!(
        project.stdout("anna", &["list", "--state", "given up"]),
        format!("INV-003  investigation  [given up] {TITLE_THREE}\n1 entry\n")
    );
}

#[test]
fn topic_filters_list_and_search_by_a_topic_ignoring_case() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "finding",
                "--title",
                "Pump seal leaks at speed",
                "--section",
                "Observation=An invented pump seal leaks above 3000 rpm.",
                "--field",
                "topic=pumps, rig",
            ],
        )
        .success();
    let line = "FND-001  finding  Pump seal leaks at speed\n";

    assert_eq!(
        project.stdout("anna", &["list", "--topic", "rig"]),
        format!("{line}1 entry\n")
    );
    assert_eq!(
        project.stdout("anna", &["list", "--topic", "PUMPS"]),
        format!("{line}1 entry\n")
    );
    assert_eq!(
        project.stdout("anna", &["list", "--topic", "valves"]),
        "0 entries\n"
    );
    assert_eq!(
        project.stdout("anna", &["search", "pump", "--topic", "rig"]),
        "FND-001  Pump seal leaks at speed  — # FND-001 · Pump seal leaks at speed\n1 match\n"
    );
}
