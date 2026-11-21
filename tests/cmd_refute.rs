//! Runs the built program for `refute`: a person at a terminal finds an approved assumption wrong,
//! and everything that rests on it is marked (R-WRITE-15). Each test runs in its own project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00 unless a test says
//! otherwise.

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::entry::Entry;
use dossier::paths;
use predicates::str::contains;

/// The fixed clock of `common::Project`: when the entries and the assumption are written.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// The time of the second refutation in the tests that refute twice, a day later.
const LATER: &str = "2026-11-21T09:30:00+01:00";

/// What the assumption affects: a task, an approved entry named by its ID, an attached file, a
/// value, a text, and an entry ID that does not exist.
const AFFECTS: &str = "T-042, FND-003, data/model.sim, design.massflow, the inlet profile, FND-099";

/// A usage record that uses the value design.massflow (format.md §15). It is written as a fixture,
/// because no command writes usage records yet.
const USAGE: &str = "<!-- dossier-format: 1 -->
# USE-001 · ExampleSolver 2406 run of speedline point 7

- kind: usage
- status: approved
- created: 2026-11-20T10:00:00+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-20T10:00:00+01:00 by anna
- valid-from: 2026-11-20T10:00:00+01:00
- task: T-042
- software: ExampleSolver
- version: 2406
- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created
- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: approved

## Parameters

massflow = {design.massflow}
";

/// A project with the scenario of the tests, before any refutation:
/// - the value design.massflow is in force;
/// - T-042 has data/model.sim and data/old.sim, and old.sim is detached in its list;
/// - T-043 has data/other.sim and results/run1.csv, which is made from data/model.sim;
/// - DEC-001 (T-042), FND-001 (T-042) and FND-003 (T-043) are approved, FND-002 (T-042) is a draft;
/// - USE-001 (T-042) uses design.massflow;
/// - ASM-001 is open and affects AFFECTS.
fn scenario() -> Project {
    let project = Project::new();
    project.write("data/model.sim", "model\n");
    project.write("data/old.sim", "old model\n");
    project.write("data/other.sim", "other\n");
    project.write("results/run1.csv", "run\n");
    project.stdout(
        "anna",
        &["value", "set", "design.massflow", "12.4 kg/s", "--key"],
    );
    project.stdout(
        "anna",
        &[
            "attach",
            "T-042",
            "data/model.sim",
            "data/old.sim",
            "--role",
            "model",
        ],
    );
    project.stdout(
        "anna",
        &["attach", "T-043", "data/other.sim", "--role", "reference"],
    );
    project.stdout(
        "anna",
        &[
            "attach",
            "T-043",
            "results/run1.csv",
            "--role",
            "result",
            "--from",
            "data/model.sim",
        ],
    );
    // No command detaches a file yet: the block of old.sim is detached by hand, as `detach` will.
    let list = project.read("DOSSIER/files/T-042.md");
    let detached = list.replace(
        "## data/old.sim\n\n",
        "## data/old.sim\n\n- detached: 2026-11-20T10:00:00+01:00 by anna\n",
    );
    assert_ne!(list, detached, "the list has the block of data/old.sim");
    project.write("DOSSIER/files/T-042.md", &detached);

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
            "found",
            "The mass flow still oscillated with half the time step.",
            "--task",
            "T-042",
        ],
    );
    project.stdout(
        "anna",
        &[
            "found",
            "Halving the time step again changed nothing.",
            "--later",
            "--task",
            "T-042",
        ],
    );
    project.stdout(
        "anna",
        &[
            "found",
            "The outlet pressure is stable over the speedline.",
            "--task",
            "T-043",
        ],
    );
    project.write("DOSSIER/usage/USE-001-examplesolver-run.md", USAGE);
    project.stdout(
        "anna",
        &[
            "assume",
            "The inlet profile is uniform over the span.",
            "--reason",
            "No measured profile was delivered.",
            "--affects",
            AFFECTS,
            "--task",
            "T-042",
        ],
    );
    project
}

/// Runs `refute` as `agent` at `now`, at a terminal (DOSSIER_ANSWERS is set; refute asks nothing).
fn refute(
    project: &Project,
    agent: &str,
    now: &str,
    id: &str,
    by: &str,
) -> assert_cmd::assert::Assert {
    project
        .command(agent, now)
        .env("DOSSIER_ANSWERS", "")
        .args(["refute", id, "--by", by])
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
fn refute_marks_everything_that_rests_on_the_assumption_and_prints_it() {
    let project = scenario();
    refute(
        &project,
        "anna",
        NOW,
        "ASM-001",
        "anna, from the rig measurements",
    )
    .success()
    .stdout(contains("ASM-001: refuted"))
    .stdout(contains("DEC-001: rests on ASM-001"))
    .stdout(contains("FND-001: rests on ASM-001"))
    .stdout(contains("FND-003: rests on ASM-001"))
    .stdout(contains("USE-001: rests on ASM-001"))
    .stdout(contains(format!(
        "T-042 file {}: rests on ASM-001",
        paths::shown("data/model.sim")
    )))
    .stdout(contains(format!(
        "T-043 file {}: rests on ASM-001",
        paths::shown("results/run1.csv")
    )))
    .stdout(contains("values named: design.massflow (used by USE-001)"))
    .stdout(contains("not recognised: the inlet profile"))
    .stderr(contains("no entry FND-099"));

    let assumption = entry_of(&project, "ASM-001");
    assert_eq!(assumption.field("state").as_deref(), Some("wrong"));
    assert_eq!(
        assumption.field("refuted").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by anna")
    );
    assert_eq!(assumption.field("valid-until").as_deref(), Some(NOW));
    assert_eq!(
        assumption.section("Confirmation").as_deref(),
        Some("anna, from the rig measurements")
    );
    assert_eq!(
        assumption.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna: refuted")
    );

    assert_eq!(entry_of(&project, "DEC-001").list("rests-on"), ["ASM-001"]);
    assert_eq!(entry_of(&project, "FND-001").list("rests-on"), ["ASM-001"]);
    assert_eq!(
        entry_of(&project, "FND-003").list("rests-on"),
        ["ASM-001"],
        "a named entry is marked whatever its task"
    );
    assert_eq!(entry_of(&project, "USE-001").list("rests-on"), ["ASM-001"]);
    assert!(
        entry_of(&project, "FND-002").list("rests-on").is_empty(),
        "a draft of the task is not marked"
    );
    assert_eq!(
        entry_of(&project, "DEC-001")
            .history()
            .last()
            .map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna: changed — rests on ASM-001")
    );

    let model = list_of(&project, "T-042");
    assert_eq!(
        model.field("data/model.sim", "rests-on").as_deref(),
        Some("ASM-001")
    );
    assert_eq!(
        model.field("data/old.sim", "rests-on"),
        None,
        "a detached block of the task is not marked"
    );
    assert_eq!(
        model.history("data/model.sim").last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna: changed — rests on ASM-001")
    );
    let results = list_of(&project, "T-043");
    assert_eq!(
        results.field("results/run1.csv", "rests-on").as_deref(),
        Some("ASM-001"),
        "a file made from a path named is marked"
    );
    assert_eq!(results.field("data/other.sim", "rests-on"), None);

    let journal = project.entry_journal();
    assert!(
        journal
            .iter()
            .any(|record| record.id == "ASM-001" && record.event == "refuted"),
        "the refutation is in the journal"
    );
    assert!(
        journal.iter().any(|record| record.id == "FILES-T-042"
            && record.event == "changed"
            && record.detail == "data/model.sim: changed — rests on ASM-001"),
        "the list of T-042 is in the journal with the block it changed"
    );
}

#[test]
fn refuting_the_same_assumption_twice_is_refused_and_changes_nothing() {
    let project = scenario();
    refute(&project, "anna", NOW, "ASM-001", "anna").success();
    let assumption = std::fs::read(project.entry_file("ASM-001")).expect("the entry is read");
    let model = project.read("DOSSIER/files/T-042.md");

    refute(&project, "anna", LATER, "ASM-001", "anna")
        .failure()
        .code(1)
        .stderr(contains("ASM-001 is already wrong"));

    assert_eq!(
        std::fs::read(project.entry_file("ASM-001")).expect("the entry is read"),
        assumption
    );
    assert_eq!(project.read("DOSSIER/files/T-042.md"), model);
}

#[test]
fn without_a_terminal_nothing_is_asked_and_nothing_is_written() {
    let project = scenario();
    let assumption = std::fs::read(project.entry_file("ASM-001")).expect("the entry is read");
    project
        .command("anna", NOW)
        .env_remove("DOSSIER_ANSWERS")
        .args(["refute", "ASM-001", "--by", "anna"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));

    assert_eq!(
        std::fs::read(project.entry_file("ASM-001")).expect("the entry is read"),
        assumption
    );
    assert!(entry_of(&project, "DEC-001").list("rests-on").is_empty());
}

#[test]
fn an_llm_is_refused_even_at_a_terminal() {
    let project = scenario();
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["refute", "ASM-001", "--by", "anna"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("refute is for a person; anna-claude is an LLM"));
    assert!(entry_of(&project, "ASM-001").field("refuted").is_none());
}

#[test]
fn only_an_assumption_can_be_refuted_and_an_unknown_one_is_not_found() {
    let project = scenario();
    refute(&project, "anna", NOW, "DEC-001", "anna")
        .failure()
        .code(1)
        .stderr(contains(
            "refute is for an assumption; DEC-001 is a decision",
        ));
    assert!(entry_of(&project, "DEC-001").list("rests-on").is_empty());

    refute(&project, "anna", NOW, "ASM-009", "anna")
        .failure()
        .code(1)
        .stderr(contains("no entry ASM-009"));
}

#[test]
fn by_is_required_and_must_say_who_and_how() {
    let project = scenario();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["refute", "ASM-001"])
        .assert()
        .failure()
        .code(2);

    refute(&project, "anna", NOW, "ASM-001", "   ")
        .failure()
        .code(1)
        .stderr(contains("must not be empty"));
    assert!(entry_of(&project, "ASM-001").field("refuted").is_none());
}

#[test]
fn json_gives_the_same_lists_as_the_text() {
    let project = scenario();
    let stdout = project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["--json", "refute", "ASM-001", "--by", "anna"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(stdout).expect("the output is UTF-8");
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");

    assert_eq!(envelope["schema"], "dossier.refute.v1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["id"], "ASM-001");
    assert_eq!(envelope["data"]["state"], "wrong");
    assert_eq!(
        envelope["data"]["entries"],
        serde_json::json!(["DEC-001", "FND-001", "USE-001", "FND-003"])
    );
    assert_eq!(
        envelope["data"]["files"],
        serde_json::json!([
            {"task": "T-042", "path": "data/model.sim"},
            {"task": "T-043", "path": "results/run1.csv"},
        ])
    );
    assert_eq!(
        envelope["data"]["values"],
        serde_json::json!([{"name": "design.massflow", "used_by": ["USE-001"]}])
    );
    assert_eq!(
        envelope["data"]["not_recognised"],
        serde_json::json!(["the inlet profile"])
    );
    let warnings = envelope["warnings"].to_string();
    assert!(warnings.contains("no entry FND-099"), "{warnings}");
}

#[test]
fn a_refusal_in_json_names_its_code() {
    let project = scenario();
    refute(&project, "anna", NOW, "ASM-001", "anna").success();
    let stdout = project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "")
        .args(["--json", "refute", "ASM-001", "--by", "anna"])
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(stdout).expect("the output is UTF-8");
    let envelope: serde_json::Value = serde_json::from_str(text.trim()).expect("one JSON line");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "REFUSED");
}
