//! Runs the built program for `check`: a clean dossier is ok; each kind of problem is printed on
//! its own line and the command exits with 1, with and without `--json`. An abbreviation that no
//! glossary explains is a warning (R-TERM-3): the command still succeeds and prints it on standard
//! error. Each test runs in its own project folder, with the clock fixed by `common::Project`.

mod common;

use common::Project;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The title of the finding that `titled_finding` records, so that its file is
/// FND-001-mass-flow-oscillates.md.
const TITLE: &str = "Mass flow oscillates";

/// Invented text for a decision, a lesson and a finding.
const DECISION: &str = "Use the transient runs for the last three points.";
const REASON: &str = "Steady runs converged where the transient run already showed rotating stall.";
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const OBSERVATION: &str = "The mass flow still oscillated with half the time step.";

/// Invented observations of findings, for `findings_naming`.
const OBSERVATIONS: [&str; 3] = [
    "The mass flow still oscillated with half the time step.",
    "The outlet temperature drifted after the first hour.",
    "The rotor speed settled only after the third revolution.",
];

/// The fixed clock of the team dossier's runs (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The output of `check` when there is no problem.
const OK: &str = "DOSSIER: ok (0 problems)\n";

/// Records the decision DEC-001 by anna: approved.
fn decide(project: &Project) {
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
}

/// Records a lesson with the statement STATEMENT as `agent`: approved for a person, a draft for
/// an LLM.
fn learn_by(project: &Project, agent: &str) {
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
            "Steady mixing-plane runs overpredict stall margin",
        ],
    );
}

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    fs::read_to_string(project.entry_file(id)).expect("the entry file")
}

/// Writes the entry file with `id` by hand, as a person would edit it.
fn write_entry(project: &Project, id: &str, text: &str) {
    fs::write(project.entry_file(id), text).expect("the entry is changed by hand");
}

/// The path that `check` prints for `file`: relative to the project folder, with `/`.
fn shown(file: &Path) -> String {
    let folder = file
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .expect("the kind folder");
    let name = file
        .file_name()
        .and_then(|name| name.to_str())
        .expect("the file name");
    format!("DOSSIER/{folder}/{name}")
}

/// The line number (1-based) of the first line of `text` that is exactly `line`.
fn line_of(text: &str, line: &str) -> usize {
    text.lines()
        .position(|candidate| candidate == line)
        .expect("the line is in the text")
        + 1
}

/// Runs `check` as anna, where it must find problems: asserts exit code 1 and returns the
/// standard error text.
fn problems(project: &Project) -> String {
    let stderr = project
        .run("anna", &["check"])
        .code(1)
        .get_output()
        .stderr
        .clone();
    String::from_utf8(stderr).expect("the standard error is UTF-8")
}

#[test]
fn a_clean_dossier_with_two_entries_is_ok() {
    let project = Project::new();
    assert_eq!(project.stdout("anna", &["check"]), OK);

    decide(&project);
    project.stdout(
        "ben",
        &["found", OBSERVATION, "--evidence", "Runs 12 and 13."],
    );

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_clean_dossier_is_ok_in_json_too() {
    let project = Project::new();
    decide(&project);

    let stdout = project.stdout("anna", &["check", "--json"]);
    let value: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(value["schema"], "dossier.check.v1");
    assert_eq!(value["ok"], true);
    assert!(value["data"].is_null());
}

#[test]
fn approved_lessons_and_drafts_by_an_llm_are_ok() {
    let project = Project::new();
    learn_by(&project, "anna");
    learn_by(&project, "anna-claude");

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_file_renamed_to_another_id_gives_name() {
    let project = Project::new();
    decide(&project);
    let file = project.entry_file("DEC-001");
    let renamed = file.with_file_name(
        file.file_name()
            .and_then(|name| name.to_str())
            .expect("a file name")
            .replacen("DEC-001", "DEC-009", 1),
    );
    fs::rename(&file, &renamed).expect("the file is renamed");

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 NAME file name does not start with DEC-001-\nDOSSIER: 1 problems\n",
            shown(&renamed)
        )
    );
}

#[test]
fn a_related_field_naming_a_missing_entry_gives_link() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("- history:", "- related: DEC-999\n- history:", 1);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- related: DEC-999");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 LINK related names DEC-999, which does not exist\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn an_approved_lesson_edited_by_hand_gives_changed() {
    let project = Project::new();
    learn_by(&project, "anna");
    let text = text_of(&project, "LES-001");
    let changed = text.replace(
        "overpredict the stall margin",
        "underpredict the stall margin",
    );
    assert_ne!(changed, text);
    write_entry(&project, "LES-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: LES-001 CHANGED approved lesson changed after approval\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("LES-001"))
        )
    );
}

#[test]
fn an_entry_with_the_author_line_deleted_gives_who() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let without = text.replacen("- author: anna\n", "", 1);
    assert_ne!(without, text);
    write_entry(&project, "DEC-001", &without);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 WHO missing field author\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_file_that_does_not_parse_gives_parse_with_no_id() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-005-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        problems(&project),
        "error: DOSSIER/findings/FND-005-broken.md:2: - PARSE no heading\nDOSSIER: 1 problems\n"
    );
}

#[test]
fn an_unknown_status_gives_status() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("- status: approved", "- status: maybe", 1);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- status: maybe");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 STATUS status must be draft, approved, rejected, superseded or retired, not \"maybe\"\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_created_time_that_is_not_a_time_gives_time() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen(
        "- created: 2026-11-20T10:00:00+01:00",
        "- created: yesterday",
        1,
    );
    assert_ne!(changed, text);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- created: yesterday");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 TIME created is not a time: \"yesterday\"\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn two_files_with_one_id_give_duplicate_on_each() {
    let project = Project::new();
    decide(&project);
    let original = project.entry_file("DEC-001");
    let copy = original.with_file_name("DEC-001-copy.md");
    fs::copy(&original, &copy).expect("the entry is copied");

    let stderr = problems(&project);
    let (original, copy) = (shown(&original), shown(&copy));
    assert!(
        stderr.contains(&format!(
            "{original}:2: DEC-001 DUPLICATE the same ID is also in {copy}\n"
        )),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "{copy}:2: DEC-001 DUPLICATE the same ID is also in {original}\n"
        )),
        "{stderr}"
    );
    assert!(stderr.ends_with("DOSSIER: 2 problems\n"), "{stderr}");
}

#[test]
fn an_approved_entry_without_a_required_section_gives_incomplete() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("## Reason\n", "## Reasons\n", 1);
    assert_ne!(changed, text);
    write_entry(&project, "DEC-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: DEC-001 INCOMPLETE missing section Reason\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn problems_fail_with_exit_one_and_refused_in_json() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    write_entry(
        &project,
        "DEC-001",
        &text.replacen("- author: anna\n", "", 1),
    );

    let stdout = project
        .run("anna", &["check", "--json"])
        .code(1)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(stdout).expect("the standard output is UTF-8");
    let value: Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");

    assert_eq!(value["schema"], "dossier.check.v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "REFUSED");
    let message = value["error"]["message"]
        .as_str()
        .expect("the message is text");
    assert!(message.ends_with("DOSSIER: 1 problems"), "{message}");
    assert!(message.contains(" WHO missing field author\n"), "{message}");
}

#[test]
fn well_formed_block_files_are_ok() {
    let project = Project::new();
    project.write(
        "DOSSIER/values.md",
        "<!-- dossier-format: 1 -->\n# Values of P-001\n\n## design.massflow\n\n- value: 12.4\n- unit: kg/s\n- source: docs/invented-spec.pdf\n- key: yes\n- valid-from: 2026-11-20T10:00:00+01:00\n- history: 2026-11-20T10:00:00+01:00 anna: set — 12.4 kg/s\n",
    );
    project.write(
        "DOSSIER/map.md",
        "<!-- dossier-format: 1 -->\n# Map of P-001\n\n## src/solver\n\n- purpose: time integration\n- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: set\n",
    );
    project.write(
        "DOSSIER/exchange.md",
        "<!-- dossier-format: 1 -->\n# Exchange of P-001\n\n## RCV-001\n\n- date: 2026-11-02\n- from: Customer A\n- history: 2026-11-02T10:00:00+01:00 anna: received\n",
    );
    project.write(
        "DOSSIER/files/T-001.md",
        "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## results/run1.csv\n\n- role: result\n- type: file\n- size: 3\n- history: 2026-11-20T10:00:00+01:00 ben: attached\n",
    );

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn block_files_that_do_not_parse_give_parse_with_their_line() {
    let project = Project::new();
    project.write(
        "DOSSIER/values.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );
    project.write(
        "DOSSIER/map.md",
        "<!-- dossier-format: 2 -->\n# Map of P-001\n",
    );
    project.write(
        "DOSSIER/files/T-001.md",
        "<!-- dossier-format: 1 -->\nno heading\n## results/run1.csv\n",
    );
    fs::write(
        project.root().join("DOSSIER/exchange.md"),
        [0xFF, 0xFE, b'#', b' ', b'x'],
    )
    .expect("exchange.md is written");

    assert_eq!(
        problems(&project),
        "error: DOSSIER/exchange.md:1: - PARSE not UTF-8\n\
         DOSSIER/files/T-001.md:2: - PARSE no heading\n\
         DOSSIER/map.md:1: - PARSE this file needs a newer dossier program\n\
         DOSSIER/values.md:2: - PARSE no heading\n\
         DOSSIER: 4 problems\n"
    );
}

#[test]
fn a_rests_on_field_naming_a_missing_assumption_gives_link() {
    let project = Project::new();
    decide(&project);
    let text = text_of(&project, "DEC-001");
    let changed = text.replacen("- history:", "- rests-on: ASM-999\n- history:", 1);
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, "- rests-on: ASM-999");
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 LINK rests-on names ASM-999, which does not exist\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_rests_on_field_of_a_file_block_is_checked_like_an_entry_link() {
    let project = Project::new();
    project.stdout(
        "ben",
        &[
            "assume",
            "The inlet profile is uniform.",
            "--reason",
            "No measured profile was delivered.",
        ],
    );
    let text = "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## results/run1.csv\n\n- role: result\n- type: file\n- rests-on: ASM-001, ASM-999\n- history: 2026-11-20T10:00:00+01:00 ben: attached\n";
    project.write("DOSSIER/files/T-001.md", text);

    let line = line_of(text, "- rests-on: ASM-001, ASM-999");
    assert_eq!(
        problems(&project),
        format!(
            "error: DOSSIER/files/T-001.md:{line}: - LINK results/run1.csv rests-on names ASM-999, which does not exist\nDOSSIER: 1 problems\n"
        )
    );
}

/// An approved how-to written by hand, with its software and its sections Purpose and Steps.
const HOW_TO: &str = "<!-- dossier-format: 1 -->\n# HOW-001 · Warm the solver before a run\n\n- kind: howto\n- status: approved\n- created: 2026-11-20T10:00:00+01:00\n- author: anna\n- approved: 2026-11-20T10:00:00+01:00 by anna\n- valid-from: 2026-11-20T10:00:00+01:00\n- software: Invented Solver\n- history: 2026-11-20T10:00:00+01:00 anna: created\n- history: 2026-11-20T10:00:00+01:00 anna: approved\n\n## Purpose\n\nPrepares an invented solver run.\n\n## Steps\n\nStart the invented run with one iteration.\n";

/// The file of the script that TOOL-001 names: a name with a space and a letter that is not ASCII.
const SCRIPT: &str = "scripts/Netz prüfen.py";

/// The text of an approved tool written by hand, with the field lines `fields` (each ends with a
/// newline) between its software and its history.
fn tool_text(fields: &str) -> String {
    format!(
        "<!-- dossier-format: 1 -->\n# TOOL-001 · Mesh check script\n\n- kind: tool\n- status: approved\n- created: 2026-11-20T10:00:00+01:00\n- author: anna\n- approved: 2026-11-20T10:00:00+01:00 by anna\n- valid-from: 2026-11-20T10:00:00+01:00\n- software: Invented Solver\n{fields}- history: 2026-11-20T10:00:00+01:00 anna: created\n- history: 2026-11-20T10:00:00+01:00 anna: approved\n\n## Purpose\n\nChecks an invented mesh.\n"
    )
}

/// Writes the tool file TOOL-001 with `fields` and returns its text.
fn write_tool(project: &Project, fields: &str) -> String {
    let text = tool_text(fields);
    project.write("DOSSIER/tools/TOOL-001-mesh-check-script.md", &text);
    text
}

/// Writes the script SCRIPT with `text` and returns its checksum, as the program records it.
fn write_script(project: &Project, text: &str) -> String {
    project.write(SCRIPT, text);
    dossier::filesum::checksum(&project.root().join(SCRIPT)).expect("the checksum of the script")
}

/// Records the approval of the entry `id` with the text `text`, as `approve` does for a person at
/// a terminal, which the tests cannot run.
fn record_approval(project: &Project, id: &str, text: &str) {
    let state = project.root().join("DOSSIER").join(".state");
    dossier::approval::record(&state, id, text.as_bytes()).expect("the approval is recorded");
}

#[test]
fn an_entry_whose_review_date_has_passed_gives_review() {
    let project = Project::new();
    decide(&project);
    project.stdout("anna", &["set", "DEC-001", "--field", "review=2026-01-01"]);
    let line = line_of(&text_of(&project, "DEC-001"), "- review: 2026-01-01");

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 REVIEW due for review: review date 2026-01-01 has passed\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn a_review_date_of_today_or_later_is_not_due() {
    let project = Project::new();
    decide(&project);
    for review in ["2026-11-20", "2026-11-21"] {
        let field = format!("review={review}");
        project.stdout("anna", &["set", "DEC-001", "--field", &field]);
        assert_eq!(project.stdout("anna", &["check"]), OK, "review {review}");
    }
}

#[test]
fn a_review_date_that_is_not_a_date_gives_review() {
    let project = Project::new();
    decide(&project);
    project.stdout("anna", &["set", "DEC-001", "--field", "review=soon"]);
    let line = line_of(&text_of(&project, "DEC-001"), "- review: soon");

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: DEC-001 REVIEW review is not a date: \"soon\"\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
}

#[test]
fn an_approved_how_to_edited_by_hand_gives_changed() {
    let project = Project::new();
    project.write(
        "DOSSIER/howtos/HOW-001-warm-the-solver-before-a-run.md",
        HOW_TO,
    );
    record_approval(&project, "HOW-001", HOW_TO);
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let changed = HOW_TO.replace("with one iteration", "with two iterations");
    assert_ne!(changed, HOW_TO);
    write_entry(&project, "HOW-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: HOW-001 CHANGED approved howto changed after approval\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("HOW-001"))
        )
    );
}

#[test]
fn an_approved_tool_edited_by_hand_gives_changed() {
    let project = Project::new();
    let text = write_tool(&project, "");
    record_approval(&project, "TOOL-001", &text);
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let changed = text.replace("Checks an invented mesh.", "Checks an invented grid.");
    assert_ne!(changed, text);
    write_entry(&project, "TOOL-001", &changed);

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: TOOL-001 CHANGED approved tool changed after approval\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("TOOL-001"))
        )
    );
}

#[test]
fn a_tool_whose_script_is_unchanged_is_ok() {
    let project = Project::new();
    let checksum = write_script(&project, "print('invented mesh check')\n");
    write_tool(
        &project,
        &format!(
            "- type: script\n- location: {SCRIPT}\n- language: python\n- checksum: {checksum}\n"
        ),
    );

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_tool_whose_script_changed_gives_tool() {
    let project = Project::new();
    let checksum = write_script(&project, "print('invented mesh check')\n");
    let text = write_tool(
        &project,
        &format!(
            "- type: script\n- location: {SCRIPT}\n- language: python\n- checksum: {checksum}\n"
        ),
    );
    write_script(&project, "print('changed mesh check')\n");

    let line = line_of(&text, &format!("- checksum: {checksum}"));
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: TOOL-001 TOOL file changed since its checksum: {SCRIPT}\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("TOOL-001"))
        )
    );
}

#[test]
fn a_tool_whose_script_is_gone_gives_tool() {
    let project = Project::new();
    let checksum = write_script(&project, "print('invented mesh check')\n");
    let text = write_tool(
        &project,
        &format!("- type: script\n- location: {SCRIPT}\n- checksum: {checksum}\n"),
    );
    std::fs::remove_file(project.root().join(SCRIPT)).expect("the script is removed");

    let line = line_of(&text, &format!("- location: {SCRIPT}"));
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: TOOL-001 TOOL file is gone: {SCRIPT}\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("TOOL-001"))
        )
    );
}

#[test]
fn a_tool_with_an_absolute_location_is_checked_too() {
    let project = Project::new();
    let checksum = write_script(&project, "print('invented mesh check')\n");
    let location = project.root().join(SCRIPT).display().to_string();
    let text = write_tool(
        &project,
        &format!("- type: script\n- location: {location}\n- checksum: {checksum}\n"),
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);

    write_script(&project, "print('changed mesh check')\n");
    let line = line_of(&text, &format!("- checksum: {checksum}"));
    assert_eq!(
        problems(&project),
        format!(
            "error: {}:{line}: TOOL-001 TOOL file changed since its checksum: {location}\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("TOOL-001"))
        )
    );
}

#[test]
fn a_tool_whose_location_is_a_command_name_is_not_checked() {
    let project = Project::new();
    write_tool(&project, "- type: command\n- location: ansys-cfx\n");

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

/// Records a finding by ben with the observation `observation` and the evidence `evidence`.
fn finding(project: &Project, observation: &str, evidence: &str) {
    project.stdout("ben", &["found", observation, "--evidence", evidence]);
}

/// Records the first `count` findings (FND-001 on) by ben, each with the evidence "Run <n> used
/// the invented <word> setup.", so that `word` appears in `count` entries.
fn findings_naming(project: &Project, word: &str, count: usize) {
    for (number, observation) in OBSERVATIONS.iter().take(count).enumerate() {
        let evidence = format!("Run {} used the invented {word} setup.", 12 + number);
        finding(project, observation, &evidence);
    }
}

/// Runs `check` as anna, where it must succeed: asserts exit code 0 and returns the standard error
/// text, which holds the warnings.
fn warnings(project: &Project) -> String {
    let stderr = project
        .run("anna", &["check"])
        .code(0)
        .get_output()
        .stderr
        .clone();
    String::from_utf8(stderr).expect("the standard error is UTF-8")
}

/// The line number (1-based) of the first line of `text` that contains `word`.
fn line_containing(text: &str, word: &str) -> usize {
    text.lines()
        .position(|line| line.contains(word))
        .expect("the word is in the text")
        + 1
}

/// The message of the TERM warning of CFD, which the first entry FND-001 holds on `line`.
fn cfd_warning(file: &str, line: usize) -> String {
    format!(
        "{file}:{line}: FND-001 TERM CFD appears in 3 entries and is in no glossary; add it with: dossier term add CFD \"<meaning>\""
    )
}

#[test]
fn an_abbreviation_in_three_entries_and_no_glossary_is_a_warning() {
    let project = Project::new();
    findings_naming(&project, "CFD", 3);
    let file = project.entry_file("FND-001");
    let line = line_containing(&text_of(&project, "FND-001"), "CFD");

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(
        warnings(&project),
        format!("warning: {}\n", cfd_warning(&shown(&file), line))
    );
}

#[test]
fn an_abbreviation_in_two_entries_is_not_reported() {
    let project = Project::new();
    findings_naming(&project, "CFD", 2);

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn an_abbreviation_that_the_project_glossary_explains_is_not_reported() {
    let project = Project::new();
    project.stdout(
        "anna",
        &["term", "add", "CFD", "computational fluid dynamics"],
    );
    findings_naming(&project, "CFD", 3);

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn a_symbol_or_another_spelling_of_a_term_is_not_reported_whatever_its_case() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "term",
            "add",
            "computational fluid dynamics",
            "fluid flow solved on a mesh",
            "--symbol",
            "CFD",
        ],
    );
    project.stdout(
        "anna",
        &[
            "term",
            "add",
            "finite element analysis",
            "stress and strain solved on a mesh",
            "--also",
            "fea",
        ],
    );
    findings_naming(&project, "CFD", 3);
    findings_naming(&project, "FEA", 3);

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn a_term_of_the_team_glossary_is_not_reported() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    state_none(&project);
    findings_naming(&project, "CFD", 3);
    assert!(
        warnings(&project).contains(" TERM CFD appears in 3 entries"),
        "the team has no CFD term yet"
    );

    project.stdout(
        "anna",
        &[
            "term",
            "add",
            "--team",
            "CFD",
            "computational fluid dynamics",
        ],
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn references_to_entries_and_exchange_records_are_not_abbreviations() {
    let project = Project::new();
    for observation in OBSERVATIONS {
        finding(
            &project,
            observation,
            "See LES-001 and RCV-001 for the invented data.",
        );
    }

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn an_abbreviation_is_counted_in_entries_not_in_occurrences() {
    let project = Project::new();
    for observation in &OBSERVATIONS[..2] {
        finding(
            &project,
            observation,
            "The invented CFD run, and the CFD run again.",
        );
    }

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn an_abbreviation_in_a_field_is_not_counted() {
    let project = Project::new();
    for observation in OBSERVATIONS {
        finding(
            &project,
            observation,
            "Run 12 used the invented solver setup.",
        );
    }
    for id in ["FND-001", "FND-002", "FND-003"] {
        let text = text_of(&project, id);
        let changed = text.replacen("- history:", "- software: CFD\n- history:", 1);
        assert_ne!(changed, text);
        write_entry(&project, id, &changed);
    }

    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert_eq!(warnings(&project), "");
}

#[test]
fn a_problem_is_reported_with_the_warnings_of_the_glossary() {
    let project = Project::new();
    findings_naming(&project, "CFD", 3);
    let file = project.entry_file("FND-001");
    let line = line_containing(&text_of(&project, "FND-001"), "CFD");
    let text = text_of(&project, "FND-002");
    write_entry(
        &project,
        "FND-002",
        &text.replacen("- author: ben\n", "", 1),
    );

    assert_eq!(
        problems(&project),
        format!(
            "error: {}:1: FND-002 WHO missing field author\nwarning: {}\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("FND-002")),
            cfd_warning(&shown(&file), line)
        )
    );
}

#[test]
fn a_glossary_that_does_not_parse_gives_parse_and_no_term_warning() {
    let project = Project::new();
    findings_naming(&project, "CFD", 3);
    project.write(
        "DOSSIER/glossary.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        problems(&project),
        "error: DOSSIER/glossary.md:2: - PARSE no heading\nDOSSIER: 1 problems\n"
    );
}

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder, whose
/// name has a space and an umlaut, run as anna. Anna is a member of the team. Returns the folder
/// (it keeps the folder alive) and the DOSSIER folder in it.
fn team_dossier(project: &Project) -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--code", "TEAM", "--team-dossier"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    (folder, dossier)
}

/// Links the project to the team dossier at `team`, as `team use` does.
fn link(project: &Project, team: &Path) {
    let path = team.display().to_string();
    project.stdout("anna", &["team", "use", path.as_str()]);
}

/// Writes the project's confidential terms file with the statement that there are none, so that a
/// team write from the project passes the check of R-CONF-5.
fn state_none(project: &Project) {
    project.write(
        "DOSSIER/confidential.md",
        "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## (no confidential terms)\n\n- stated: 2026-11-02T09:00:00+01:00 by anna\n",
    );
}

/// An invented password line that `check` reports without its value.
const CREDENTIAL_LINE: &str = "password: hunter2secret";

/// The value of CREDENTIAL_LINE. `check` must never print it.
const CREDENTIAL_VALUE: &str = "hunter2secret";

/// The end of a SECRET line for a password line (format.md §27, design.md section 13).
const SECRET_MESSAGE: &str = "looks like a credential (password); remove it, or a person allows it with --allow-credential-like on the command that writes it";

#[test]
fn a_password_line_added_by_hand_to_an_entry_gives_secret_without_its_value() {
    let project = Project::new();
    decide(&project);
    let changed = format!("{}{CREDENTIAL_LINE}\n", text_of(&project, "DEC-001"));
    write_entry(&project, "DEC-001", &changed);

    let line = line_of(&changed, CREDENTIAL_LINE);
    let stderr = problems(&project);
    assert_eq!(
        stderr,
        format!(
            "error: {}:{line}: DEC-001 SECRET {SECRET_MESSAGE}\nDOSSIER: 1 problems\n",
            shown(&project.entry_file("DEC-001"))
        )
    );
    assert!(!stderr.contains(CREDENTIAL_VALUE), "{stderr}");
}

#[test]
fn a_password_line_that_a_person_allowed_is_not_reported() {
    let project = Project::new();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["found", CREDENTIAL_LINE, "--allow-credential-like"])
        .assert()
        .success();
    decide(&project);
    let changed = format!("{}{CREDENTIAL_LINE}\n", text_of(&project, "DEC-001"));
    write_entry(&project, "DEC-001", &changed);

    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn a_password_line_in_a_file_block_gives_secret_with_no_id() {
    let project = Project::new();
    let text = format!(
        "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## results/run1.csv\n\n- role: result\n- type: file\n{CREDENTIAL_LINE}\n- history: 2026-11-20T10:00:00+01:00 ben: attached\n"
    );
    project.write("DOSSIER/files/T-001.md", &text);

    let line = line_of(&text, CREDENTIAL_LINE);
    let stderr = problems(&project);
    assert_eq!(
        stderr,
        format!(
            "error: DOSSIER/files/T-001.md:{line}: - SECRET {SECRET_MESSAGE}\nDOSSIER: 1 problems\n"
        )
    );
    assert!(!stderr.contains(CREDENTIAL_VALUE), "{stderr}");
}

/// Records the finding FND-001 by ben, titled TITLE.
fn titled_finding(project: &Project) {
    project.stdout(
        "ben",
        &[
            "found",
            OBSERVATION,
            "--evidence",
            "Runs 12 and 13.",
            "--title",
            TITLE,
        ],
    );
}

/// Renames the file of `id` by hand to `name`, in the same folder, and returns the new path.
fn rename_by_hand(project: &Project, id: &str, name: &str) -> PathBuf {
    let file = project.entry_file(id);
    let renamed = file.with_file_name(name);
    fs::rename(&file, &renamed).expect("the file is renamed");
    renamed
}

#[test]
fn check_fix_renames_a_file_back_to_its_id_and_title() {
    let project = Project::new();
    titled_finding(&project);
    let wrong = rename_by_hand(&project, "FND-001", "FND-001-wrong.md");

    // A plain check reports nothing here (the ID is right) and changes nothing.
    assert_eq!(project.stdout("anna", &["check"]), OK);
    assert!(wrong.exists());

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let right = project.entry_file("FND-001");
    assert_eq!(
        stdout,
        format!(
            "fixed: {} renamed to {}\n{}",
            shown(&wrong),
            shown(&right),
            OK
        )
    );
    assert!(!wrong.exists());
    assert_eq!(
        right.file_name().and_then(|name| name.to_str()),
        Some("FND-001-mass-flow-oscillates.md")
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_records_the_repair_in_the_history_and_the_journal() {
    let project = Project::new();
    titled_finding(&project);
    rename_by_hand(&project, "FND-001", "FND-001-wrong.md");

    project.stdout("anna", &["check", "--fix"]);

    let text = text_of(&project, "FND-001");
    assert!(
        text.contains("- history: 2026-11-20T10:00:00+01:00 anna: fixed — file name\n"),
        "{text}"
    );
    let records = project.entry_journal();
    let last = records.last().expect("the repair is in the journal");
    assert_eq!(last.id, "FND-001");
    assert_eq!(last.name, "anna");
    assert_eq!(last.command, "check");
    assert_eq!(last.event, "fixed");
    assert_eq!(last.detail, "file name");
}

#[test]
fn check_fix_with_a_dry_run_renames_nothing() {
    let project = Project::new();
    titled_finding(&project);
    let wrong = rename_by_hand(&project, "FND-001", "FND-001-wrong.md");
    let right = project
        .root()
        .join("DOSSIER/findings/FND-001-mass-flow-oscillates.md");

    let output = project
        .run("anna", &["check", "--fix", "--dry-run"])
        .code(0)
        .get_output()
        .clone();

    assert_eq!(
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        OK
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("the standard error is UTF-8"),
        format!(
            "warning: dry run: would rename {} to {}\n",
            shown(&wrong),
            shown(&right)
        )
    );
    assert!(wrong.exists());
    assert!(!right.exists());
}

#[test]
fn check_fix_without_a_name_is_a_usage_error_and_changes_nothing() {
    let project = Project::new();
    titled_finding(&project);
    let wrong = rename_by_hand(&project, "FND-001", "FND-001-wrong.md");

    let stderr = project
        .run("", &["check", "--fix"])
        .code(2)
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(stderr).expect("the standard error is UTF-8");
    assert!(stderr.contains("needs a name"), "{stderr}");
    assert!(wrong.exists());
}

#[test]
fn check_fix_leaves_a_file_alone_when_its_right_name_is_taken() {
    let project = Project::new();
    titled_finding(&project);
    let wrong = rename_by_hand(&project, "FND-001", "FND-001-wrong.md");
    project.write(
        "DOSSIER/findings/FND-001-mass-flow-oscillates.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    let stderr = project
        .run("anna", &["check", "--fix"])
        .code(1)
        .get_output()
        .stderr
        .clone();
    assert_eq!(
        String::from_utf8(stderr).expect("the standard error is UTF-8"),
        "error: DOSSIER/findings/FND-001-mass-flow-oscillates.md:2: - PARSE no heading\nDOSSIER: 1 problems\n"
    );
    assert!(wrong.exists());
}

#[test]
fn check_fix_renames_an_approved_lesson_without_a_changed_mark() {
    let project = Project::new();
    learn_by(&project, "anna");
    let wrong = rename_by_hand(&project, "LES-001", "LES-001-wrong.md");

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let right = project.entry_file("LES-001");
    assert_eq!(
        stdout,
        format!(
            "fixed: {} renamed to {}\n{}",
            shown(&wrong),
            shown(&right),
            OK
        )
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

/// The format line of version 1 (format.md §3).
const FORMAT_LINE: &str = "<!-- dossier-format: 1 -->";

/// Removes the format line of the entry `id` by hand, keeping every other byte, and returns the
/// text it had before.
fn remove_format_line_by_hand(project: &Project, id: &str) -> String {
    let original = text_of(project, id);
    let without = original.replacen(&format!("{FORMAT_LINE}\n"), "", 1);
    assert_ne!(without, original, "the format line is in the entry");
    write_entry(project, id, &without);
    original
}

#[test]
fn check_fix_puts_back_a_format_line_removed_by_hand() {
    let project = Project::new();
    decide(&project);
    let original = remove_format_line_by_hand(&project, "DEC-001");
    // A file without a format line is read as version 1, so a plain check is still ok.
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let file = project.entry_file("DEC-001");
    assert_eq!(stdout, format!("{}: format line added\n{OK}", shown(&file)));

    // The only other change is the history line of the repair; every other byte is as it was.
    let text = text_of(&project, "DEC-001");
    let history = format!("- history: {NOW} anna: fixed — format line\n");
    assert!(
        text.starts_with(&format!("{FORMAT_LINE}\n# DEC-001")),
        "{text}"
    );
    assert_eq!(text.replacen(&history, "", 1), original);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_records_the_format_line_in_the_history_and_the_journal() {
    let project = Project::new();
    decide(&project);
    remove_format_line_by_hand(&project, "DEC-001");

    project.stdout("anna", &["check", "--fix"]);

    let text = text_of(&project, "DEC-001");
    assert!(
        text.contains("- history: 2026-11-20T10:00:00+01:00 anna: fixed — format line\n"),
        "{text}"
    );
    let records = project.entry_journal();
    let last = records.last().expect("the repair is in the journal");
    assert_eq!(last.id, "DEC-001");
    assert_eq!(last.name, "anna");
    assert_eq!(last.command, "check");
    assert_eq!(last.event, "fixed");
    assert_eq!(last.detail, "format line");
}

#[test]
fn check_fix_gives_a_crlf_format_line_to_an_entry_saved_with_crlf() {
    let project = Project::new();
    decide(&project);
    let crlf = text_of(&project, "DEC-001").replace('\n', "\r\n");
    write_entry(
        &project,
        "DEC-001",
        &crlf.replacen(&format!("{FORMAT_LINE}\r\n"), "", 1),
    );

    project.stdout("anna", &["check", "--fix"]);

    let text = text_of(&project, "DEC-001");
    assert!(
        text.starts_with(&format!("{FORMAT_LINE}\r\n# DEC-001")),
        "{text}"
    );
    assert!(
        !text.replace("\r\n", "").contains('\n'),
        "a bare LF in {text:?}"
    );
    let history = format!("- history: {NOW} anna: fixed — format line\r\n");
    assert_eq!(text.replacen(&history, "", 1), crlf);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_keeps_a_byte_order_mark_at_the_very_start() {
    let project = Project::new();
    decide(&project);
    let original = remove_format_line_by_hand(&project, "DEC-001");
    write_entry(
        &project,
        "DEC-001",
        &format!("\u{feff}{}", text_of(&project, "DEC-001")),
    );

    project.stdout("anna", &["check", "--fix"]);

    let text = text_of(&project, "DEC-001");
    let history = format!("- history: {NOW} anna: fixed — format line\n");
    assert!(
        text.starts_with(&format!("\u{feff}{FORMAT_LINE}\n# DEC-001")),
        "{text}"
    );
    assert_eq!(text.matches('\u{feff}').count(), 1, "{text}");
    assert_eq!(
        text.replacen(&history, "", 1),
        format!("\u{feff}{original}")
    );
}

#[test]
fn check_fix_with_a_dry_run_adds_no_format_line() {
    let project = Project::new();
    decide(&project);
    remove_format_line_by_hand(&project, "DEC-001");
    let without = text_of(&project, "DEC-001");
    let file = project.entry_file("DEC-001");

    let output = project
        .run("anna", &["check", "--fix", "--dry-run"])
        .code(0)
        .get_output()
        .clone();

    assert_eq!(
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        OK
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("the standard error is UTF-8"),
        format!("warning: dry run: would write {}\n", shown(&file))
    );
    assert_eq!(text_of(&project, "DEC-001"), without);
}

#[test]
fn check_fix_renames_and_adds_a_format_line_in_one_repair() {
    let project = Project::new();
    titled_finding(&project);
    remove_format_line_by_hand(&project, "FND-001");
    let wrong = rename_by_hand(&project, "FND-001", "FND-001-wrong.md");

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let right = project.entry_file("FND-001");
    assert_eq!(
        stdout,
        format!(
            "fixed: {} renamed to {}\n{}: format line added\n{OK}",
            shown(&wrong),
            shown(&right),
            shown(&right)
        )
    );
    assert!(!wrong.exists());

    let text = text_of(&project, "FND-001");
    assert!(
        text.starts_with(&format!("{FORMAT_LINE}\n# FND-001")),
        "{text}"
    );
    assert!(text.contains(&format!("- history: {NOW} anna: fixed — file name\n")));
    assert!(text.contains(&format!("- history: {NOW} anna: fixed — format line\n")));
    let records = project.entry_journal();
    let last = records.last().expect("the repair is in the journal");
    assert_eq!(last.id, "FND-001");
    assert_eq!(last.event, "fixed");
    assert_eq!(last.detail, "file name, format line");
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

/// Moves the line that starts with `moved` of the entry `id` by hand to just after the line that
/// starts with `anchor`, each line whole with its own ending. Returns the text before the move.
fn move_line_after(project: &Project, id: &str, moved: &str, anchor: &str) -> String {
    let original = text_of(project, id);
    let mut lines: Vec<String> = original.split_inclusive('\n').map(str::to_string).collect();
    let from = lines
        .iter()
        .position(|line| line.starts_with(moved))
        .expect("the moved line is in the entry");
    let line = lines.remove(from);
    let to = lines
        .iter()
        .position(|line| line.starts_with(anchor))
        .expect("the anchor line is in the entry");
    lines.insert(to + 1, line);
    write_entry(project, id, &lines.concat());
    original
}

/// The history line that `check --fix` adds for fields put in order, as anna at the fixed time.
fn field_order_history() -> String {
    format!("- history: {NOW} anna: fixed — field order\n")
}

#[test]
fn check_fix_puts_back_a_field_moved_by_hand_below_the_one_after_it() {
    let project = Project::new();
    decide(&project);
    let original = move_line_after(&project, "DEC-001", "- created:", "- author:");
    // The order of the fields is not a problem for `check`: it reports nothing.
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let file = project.entry_file("DEC-001");
    assert_eq!(
        stdout,
        format!("{}: fields put in order\n{OK}", shown(&file))
    );

    // The only other change is the history line of the repair; every other line is as it was.
    let text = text_of(&project, "DEC-001");
    assert!(text.contains(&field_order_history()), "{text}");
    assert_eq!(text.replacen(&field_order_history(), "", 1), original);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_keeps_an_unknown_field_at_its_place_among_the_known_ones() {
    let project = Project::new();
    decide(&project);
    let canonical = text_of(&project, "DEC-001");
    let with_unknown = canonical.replacen(
        "- status: approved\n",
        "- status: approved\n- zeta: kept by hand\n",
        1,
    );
    write_entry(&project, "DEC-001", &with_unknown);
    move_line_after(&project, "DEC-001", "- created:", "- author:");

    project.stdout("anna", &["check", "--fix"]);

    // The known fields are in order again; the unknown key stays between `status` and the rest.
    let text = text_of(&project, "DEC-001");
    let expected = canonical.replacen(
        "- status: approved\n",
        "- status: approved\n- zeta: kept by hand\n",
        1,
    );
    assert_eq!(text.replacen(&field_order_history(), "", 1), expected);
}

#[test]
fn check_fix_keeps_the_history_lines_where_they_are() {
    let project = Project::new();
    decide(&project);
    let canonical = text_of(&project, "DEC-001");
    let created = format!("- history: {NOW} anna: created\n");
    // The first history line is moved up under `kind` by hand, and `created` below `author`.
    let moved_up = |text: &str| {
        text.replacen(&created, "", 1).replacen(
            "- kind: decision\n",
            &format!("- kind: decision\n{created}"),
            1,
        )
    };
    write_entry(&project, "DEC-001", &moved_up(&canonical));
    move_line_after(&project, "DEC-001", "- created:", "- author:");

    project.stdout("anna", &["check", "--fix"]);

    // The history line keeps its text and its place; only the fields around it moved back.
    let text = text_of(&project, "DEC-001");
    assert_eq!(
        text.replacen(&field_order_history(), "", 1),
        moved_up(&canonical)
    );
}

#[test]
fn check_fix_does_not_write_an_entry_whose_fields_are_in_order() {
    let project = Project::new();
    decide(&project);
    let before = text_of(&project, "DEC-001");
    let records = project.entry_journal().len();

    assert_eq!(project.stdout("anna", &["check", "--fix"]), OK);

    assert_eq!(text_of(&project, "DEC-001"), before);
    assert_eq!(project.entry_journal().len(), records);
}

#[test]
fn check_fix_keeps_the_crlf_endings_of_moved_fields() {
    let project = Project::new();
    decide(&project);
    let crlf = text_of(&project, "DEC-001").replace('\n', "\r\n");
    write_entry(&project, "DEC-001", &crlf);
    move_line_after(&project, "DEC-001", "- created:", "- author:");

    project.stdout("anna", &["check", "--fix"]);

    let text = text_of(&project, "DEC-001");
    assert!(
        !text.replace("\r\n", "").contains('\n'),
        "a bare LF in {text:?}"
    );
    let history = field_order_history().replace('\n', "\r\n");
    assert_eq!(text.replacen(&history, "", 1), crlf);
}

#[test]
fn check_fix_puts_back_the_fields_of_an_approved_lesson_and_it_is_unchanged_again() {
    let project = Project::new();
    learn_by(&project, "anna");
    move_line_after(&project, "LES-001", "- created:", "- author:");
    // The approved text is no longer the text that was approved, so `check` reports it.
    let stderr = problems(&project);
    assert!(
        stderr.contains("LES-001 CHANGED approved lesson changed after approval"),
        "{stderr}"
    );

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let file = project.entry_file("LES-001");
    assert_eq!(
        stdout,
        format!("{}: fields put in order\n{OK}", shown(&file))
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_puts_in_order_the_approval_fields_an_approved_llm_lesson_got_after_its_tags() {
    let project = Project::new();
    project.stdout(
        "ben-codex",
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
            "Steady mixing-plane runs overpredict stall margin",
            "--tags",
            "rig",
        ],
    );
    // Approval adds `approved` and `valid-from` before the history lines, after the tags. Neither
    // is part of the approval checksum, so the order can be put right without a change.
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let file = project.entry_file("LES-001");
    assert_eq!(
        stdout,
        format!("{}: fields put in order\n{OK}", shown(&file))
    );
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_leaves_an_approved_entry_in_its_order_when_the_new_order_would_change_it() {
    let project = Project::new();
    project.stdout(
        "ben-codex",
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
            "Steady mixing-plane runs overpredict stall margin",
            "--tags",
            "rig",
        ],
    );
    // `machine` is written after `tags`, which is out of order; the approval records that text.
    project.stdout("ben-codex", &["set", "LES-001", "--field", "machine=lab"]);
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();
    let before = text_of(&project, "LES-001");

    // Putting `machine` before `tags` would make the approved lesson read as changed: it stays.
    let output = project
        .run("anna", &["check", "--fix"])
        .code(0)
        .get_output()
        .clone();
    assert_eq!(
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        OK
    );
    let stderr = String::from_utf8(output.stderr).expect("the standard error is UTF-8");
    assert!(
        stderr.contains(
            "fields not put in order, because the approved entry would then show as changed"
        ),
        "{stderr}"
    );
    assert_eq!(text_of(&project, "LES-001"), before);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_leaves_an_approved_lesson_without_a_format_line_as_it_is() {
    let project = Project::new();
    learn_by(&project, "anna");
    // The approval is recorded for the text without a format line, so adding one would change it.
    remove_format_line_by_hand(&project, "LES-001");
    record_approval(&project, "LES-001", &text_of(&project, "LES-001"));
    let without = text_of(&project, "LES-001");
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let output = project
        .run("anna", &["check", "--fix"])
        .code(0)
        .get_output()
        .clone();
    assert_eq!(
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        OK
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("the standard error is UTF-8"),
        format!(
            "warning: {}: format line not added, because the approved entry would then show as changed\n",
            shown(&project.entry_file("LES-001"))
        )
    );
    assert_eq!(text_of(&project, "LES-001"), without);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_leaves_an_approved_lesson_with_both_repairs_as_it_is() {
    let project = Project::new();
    learn_by(&project, "anna");
    // The fields are out of order and there is no format line; the approval records that text.
    move_line_after(&project, "LES-001", "- created:", "- author:");
    remove_format_line_by_hand(&project, "LES-001");
    record_approval(&project, "LES-001", &text_of(&project, "LES-001"));
    let before = text_of(&project, "LES-001");
    assert_eq!(project.stdout("anna", &["check"]), OK);

    let output = project
        .run("anna", &["check", "--fix"])
        .code(0)
        .get_output()
        .clone();
    assert_eq!(
        String::from_utf8(output.stdout).expect("the standard output is UTF-8"),
        OK
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("the standard error is UTF-8"),
        format!(
            "warning: {}: fields not put in order and format line not added, because the approved entry would then show as changed\n",
            shown(&project.entry_file("LES-001"))
        )
    );
    assert_eq!(text_of(&project, "LES-001"), before);
    assert_eq!(project.stdout("anna", &["check"]), OK);
}

#[test]
fn check_fix_puts_back_the_format_line_of_an_approved_lesson_and_it_is_unchanged_again() {
    let project = Project::new();
    learn_by(&project, "anna");
    remove_format_line_by_hand(&project, "LES-001");
    // The approved text had the format line, so the text without it reads as changed.
    assert!(
        problems(&project).contains("LES-001 CHANGED approved lesson changed after approval"),
        "the entry reads as changed"
    );

    let stdout = project.stdout("anna", &["check", "--fix"]);
    let file = project.entry_file("LES-001");
    assert_eq!(stdout, format!("{}: format line added\n{OK}", shown(&file)));
    assert_eq!(project.stdout("anna", &["check"]), OK);
}
