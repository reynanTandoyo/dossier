//! Runs the built program for `cleared`: a person clears one rests-on mark that `refute` wrote, on
//! an entry or on the block of a file, and the history keeps the note (R-WRITE-15). An LLM does so
//! with `--said-by <person>` (D57). Each test runs in its own project folder (common::Project), with
//! the clock fixed at 2026-11-20T10:00:00+01:00 unless a test says otherwise.

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::entry::{Entry, Status};
use dossier::paths;
use predicates::str::contains;

/// The fixed clock of `common::Project`: when the assumptions are written and the first refutation.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// The time of the second refutation, a day later.
const LATER: &str = "2026-11-21T09:30:00+01:00";
/// The note of the clearing in most tests.
const NOTE: &str = "the rig measured a flat inlet profile";

/// A project with the scenario of the tests, before any refutation:
/// - data/model.sim is attached to T-042 (role model) and to T-043 (role reference);
/// - DEC-001 is an approved decision of T-042;
/// - ASM-001 is open and affects T-042 and data/model.sim: it marks DEC-001 (approved, of T-042)
///   and both blocks of data/model.sim;
/// - ASM-002 is open and affects DEC-001 only.
fn scenario() -> Project {
    let project = Project::new();
    project.write("data/model.sim", "model\n");
    project.stdout(
        "anna",
        &["attach", "T-042", "data/model.sim", "--role", "model"],
    );
    project.stdout(
        "anna",
        &["attach", "T-043", "data/model.sim", "--role", "reference"],
    );
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
            "--task",
            "T-042",
        ],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform over the span.",
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            "T-042, data/model.sim",
            "--task",
            "T-042",
        ],
    );
    project.stdout(
        "anna",
        &[
            "assume",
            "The outlet pressure is steady over the speedline.",
            "--reason",
            "Two pressure taps only.",
            "--affects",
            "DEC-001",
        ],
    );
    project
}

/// Runs `refute` for `id` as anna at `now`, at a terminal (DOSSIER_ANSWERS is set; refute asks
/// nothing).
fn refute(project: &Project, now: &str, id: &str) -> assert_cmd::assert::Assert {
    project
        .command("anna", now)
        .env("DOSSIER_ANSWERS", "")
        .args(["refute", id, "--by", "anna, from the rig measurements"])
        .assert()
}

/// Runs `cleared` as `agent` with the given target, assumption and note. No terminal is set up
/// (DOSSIER_ANSWERS is removed): cleared is for a person, not for a terminal.
fn cleared(
    project: &Project,
    agent: &str,
    target: &str,
    assumption: &str,
    note: &str,
) -> assert_cmd::assert::Assert {
    project
        .command(agent, NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "cleared",
            target,
            "--assumption",
            assumption,
            "--note",
            note,
        ])
        .assert()
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// The file list of `task`, read and parsed as a block file.
fn list_of(project: &Project, task: &str) -> BlockFile {
    BlockFile::parse(&project.read(&format!("DOSSIER/files/{task}.md"))).expect("the list parses")
}

#[test]
fn cleared_on_an_entry_removes_the_mark_and_keeps_the_note_in_the_history() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);

    cleared(&project, "anna", "DEC-001", "ASM-001", NOTE)
        .success()
        .stdout(contains("DEC-001: cleared ASM-001"));

    let decision = entry_of(&project, "DEC-001");
    assert!(
        decision.field("rests-on").is_none(),
        "the last mark of the entry takes its line away"
    );
    assert_eq!(
        decision.status(),
        Some(Status::Approved),
        "the decision itself is not changed"
    );
    assert_eq!(
        decision.history().last().map(String::as_str),
        Some(
            "2026-11-20T10:00:00+01:00 anna: cleared — ASM-001: the rig measured a flat inlet profile"
        )
    );
    assert!(
        project
            .entry_journal()
            .iter()
            .any(|record| record.id == "DEC-001"
                && record.event == "cleared"
                && record.detail == "ASM-001: the rig measured a flat inlet profile"),
        "the clearing is in the journal"
    );
}

#[test]
fn cleared_on_a_path_removes_the_mark_of_every_block_of_that_path() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    for task in ["T-042", "T-043"] {
        assert_eq!(
            list_of(&project, task)
                .field("data/model.sim", "rests-on")
                .as_deref(),
            Some("ASM-001"),
            "{task} marks data/model.sim after refute"
        );
    }

    let shown = paths::shown("data/model.sim");
    cleared(&project, "anna", "data/model.sim", "ASM-001", NOTE)
        .success()
        .stdout(contains(format!("T-042 file {shown}: cleared ASM-001")))
        .stdout(contains(format!("T-043 file {shown}: cleared ASM-001")));

    for task in ["T-042", "T-043"] {
        let list = list_of(&project, task);
        assert_eq!(
            list.field("data/model.sim", "rests-on"),
            None,
            "{task}: the mark of the block is removed"
        );
        assert_eq!(
            list.history("data/model.sim").last().map(String::as_str),
            Some(
                "2026-11-20T10:00:00+01:00 anna: cleared — ASM-001: the rig measured a flat inlet profile"
            ),
            "{task}: the block keeps the history line"
        );
    }
    assert_eq!(
        entry_of(&project, "DEC-001").list("rests-on"),
        ["ASM-001"],
        "clearing a file leaves the entry's own mark"
    );

    let journal = project.entry_journal();
    assert!(
        journal.iter().any(|record| record.id == "FILES-T-042"
            && record.event == "cleared"
            && record.detail
                == "data/model.sim: cleared — ASM-001: the rig measured a flat inlet profile"),
        "the list of T-042 is in the journal with the block it changed"
    );
}

#[test]
fn clearing_one_mark_keeps_the_others_and_any_spelling_of_the_ids_works() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    refute(&project, LATER, "ASM-002").success();
    assert_eq!(
        entry_of(&project, "DEC-001").list("rests-on"),
        ["ASM-001", "ASM-002"]
    );

    cleared(&project, "anna", "P-001:DEC-001", "asm-001", NOTE)
        .success()
        .stdout(contains("DEC-001: cleared ASM-001"));
    assert_eq!(
        entry_of(&project, "DEC-001").list("rests-on"),
        ["ASM-002"],
        "the other mark stays"
    );

    cleared(
        &project,
        "anna",
        "DEC-001",
        "ASM-002",
        "the second pressure tap agrees",
    )
    .success();
    assert!(
        entry_of(&project, "DEC-001").field("rests-on").is_none(),
        "the last mark is gone"
    );
}

#[test]
fn a_mark_that_is_not_there_is_not_found_and_changes_nothing() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    let decision = std::fs::read(project.entry_file("DEC-001")).expect("the entry is read");
    let list = project.read("DOSSIER/files/T-042.md");

    cleared(&project, "anna", "DEC-001", "ASM-002", NOTE)
        .failure()
        .code(1)
        .stderr(contains("DEC-001 has no mark for ASM-002"));
    cleared(&project, "anna", "data/model.sim", "ASM-002", NOTE)
        .failure()
        .code(1)
        .stderr(contains(format!(
            "{} has no mark for ASM-002",
            paths::shown("data/model.sim")
        )));
    cleared(&project, "anna", "DEC-009", "ASM-001", NOTE)
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));

    assert_eq!(
        std::fs::read(project.entry_file("DEC-001")).expect("the entry is read"),
        decision
    );
    assert_eq!(project.read("DOSSIER/files/T-042.md"), list);
}

#[test]
fn only_a_person_clears_and_the_assumption_and_the_note_are_checked() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    let decision = std::fs::read(project.entry_file("DEC-001")).expect("the entry is read");

    cleared(&project, "anna-claude", "DEC-001", "ASM-001", NOTE)
        .failure()
        .code(1)
        .stderr(contains("--said-by anna"));
    cleared(&project, "anna", "DEC-001", "DEC-001", NOTE)
        .failure()
        .code(1)
        .stderr(contains("--assumption names an assumption"));
    cleared(&project, "anna", "DEC-001", "ASM-001", "   ")
        .failure()
        .code(1)
        .stderr(contains("must not be empty"));
    cleared(&project, "anna", "DEC-001", "ASM-001", "one\ntwo")
        .failure()
        .code(1)
        .stderr(contains("the --note text is one line"));
    project
        .command("anna", NOW)
        .args(["cleared", "DEC-001", "--assumption", "ASM-001"])
        .assert()
        .failure()
        .code(2);

    assert_eq!(
        std::fs::read(project.entry_file("DEC-001")).expect("the entry is read"),
        decision,
        "a refused change writes nothing"
    );
    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);
}

#[test]
fn an_llm_without_said_by_is_refused_and_nothing_is_written() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    let decision = std::fs::read(project.entry_file("DEC-001")).expect("the entry is read");

    project
        .command("anna-claude", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "cleared",
            "DEC-001",
            "--assumption",
            "ASM-001",
            "--note",
            NOTE,
        ])
        .assert()
        .code(1)
        .stderr(contains("--said-by anna"));

    assert_eq!(
        std::fs::read(project.entry_file("DEC-001")).expect("the entry is read"),
        decision,
        "a refused change writes nothing"
    );
    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);
}

#[test]
fn an_llm_with_said_by_clears_its_mark_without_a_terminal_or_a_question() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    // No answers are prepared and no terminal is attached, so nothing can be asked: the mark is
    // cleared as the person decided (D57).
    project
        .command("anna-claude", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "cleared",
            "DEC-001",
            "--assumption",
            "ASM-001",
            "--note",
            NOTE,
            "--said-by",
            "anna",
        ])
        .assert()
        .success()
        .stdout("DEC-001: cleared ASM-001\n")
        .stderr("");

    let decision = entry_of(&project, "DEC-001");
    assert!(
        decision.field("rests-on").is_none(),
        "the last mark of the entry takes its line away"
    );
    assert_eq!(
        decision.history().last().map(String::as_str),
        Some(
            "2026-11-20T10:00:00+01:00 anna via anna-claude: cleared — ASM-001: the rig measured a flat inlet profile"
        )
    );
    let journal = project.entry_journal();
    let newest = journal.last().expect("the clearing is journalled");
    assert_eq!(newest.id, "DEC-001");
    assert_eq!(newest.event, "cleared");
    assert_eq!(newest.person, "anna");
    assert_eq!(newest.name, "anna-claude");
    assert!(newest.llm);
}

#[test]
fn an_llm_with_said_by_for_another_person_is_refused_and_nothing_is_written() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    let decision = std::fs::read(project.entry_file("DEC-001")).expect("the entry is read");

    project
        .command("anna-claude", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "cleared",
            "DEC-001",
            "--assumption",
            "ASM-001",
            "--note",
            NOTE,
            "--said-by",
            "ben",
        ])
        .assert()
        .code(1)
        .stderr(contains("acts for anna only"));

    assert_eq!(
        std::fs::read(project.entry_file("DEC-001")).expect("the entry is read"),
        decision,
        "a refused change writes nothing"
    );
    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);
}

#[test]
fn a_worker_session_clears_nothing_even_with_said_by() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();
    let decision = std::fs::read(project.entry_file("DEC-001")).expect("the entry is read");

    project
        .command("anna-claude", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .env("DOSSIER_ROLE", "worker")
        .args([
            "cleared",
            "DEC-001",
            "--assumption",
            "ASM-001",
            "--note",
            NOTE,
            "--said-by",
            "anna",
        ])
        .assert()
        .code(1)
        .stderr(contains("a worker decides nothing"));

    assert_eq!(
        std::fs::read(project.entry_file("DEC-001")).expect("the entry is read"),
        decision,
        "a refused change writes nothing"
    );
    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);
}

#[test]
fn json_gives_the_same_lists_as_the_text() {
    let project = scenario();
    refute(&project, NOW, "ASM-001").success();

    let stdout = project
        .command("anna", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args([
            "--json",
            "cleared",
            "data/model.sim",
            "--assumption",
            "ASM-001",
            "--note",
            NOTE,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(stdout).expect("the output is UTF-8");
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");

    assert_eq!(envelope["schema"], "dossier.cleared.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["assumption"], "ASM-001");
    assert_eq!(envelope["data"]["entries"], serde_json::json!([]));
    assert_eq!(
        envelope["data"]["files"],
        serde_json::json!([
            {"task": "T-042", "path": "data/model.sim"},
            {"task": "T-043", "path": "data/model.sim"},
        ])
    );
}
