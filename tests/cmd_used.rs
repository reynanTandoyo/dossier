//! Runs the built program for `used`: one run is recorded as a usage record (USE-<n>, approved at
//! once) and its files are attached to the task's file list in the same step: the model with role
//! `model`, and each output with role `result` and made from `<model>@<checksum>`. An output that is
//! attached already is described again with a `reattached` line. A bad value writes nothing. Each
//! test runs in its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::entry::{Entry, Kind, Status};
use dossier::hash::sha256_hex;
use dossier::journal::Record;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented contents of the files of the tests.
const MODEL: &str = "invented model data\n";
const RESULT: &str = "time;value\n0;1\n1;2\n";
const RESULT_LATER: &str = "time;value\n0;3\n";

/// The checksum of some text, as the list and the record keep it (format.md §18).
fn checksum_of(text: &str) -> String {
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// The usage record `id`, parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id))
        .unwrap_or_else(|error| panic!("reading {id}: {error}"));
    Entry::parse(&text).expect("the record parses")
}

/// The file list of task T-001, parsed. Panics when the list does not exist.
fn list_of(project: &Project) -> BlockFile {
    BlockFile::parse(&project.read("DOSSIER/files/T-001.md")).expect("the file list parses")
}

/// The journal records about the file list of task T-001, oldest first.
fn list_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "FILES-T-001")
        .collect()
}

/// The (event, detail) of the journal records of `id`, oldest first.
fn journal_of(project: &Project, id: &str) -> Vec<(String, String)> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == id)
        .map(|record| (record.event, record.detail))
        .collect()
}

/// The number of files in the usage folder: 0 when the folder does not exist.
fn usage_files(project: &Project) -> usize {
    let folder = project.root().join("DOSSIER").join("usage");
    std::fs::read_dir(folder).map_or(0, |entries| entries.flatten().count())
}

/// Writes the invented model and result files of the tests into the project folder.
fn write_files(project: &Project) {
    project.write("sim/m.sim", MODEL);
    project.write("out.csv", RESULT);
}

#[test]
fn a_run_records_a_usage_record_and_attaches_the_model_and_the_output() {
    let project = Project::new();
    write_files(&project);

    let out = project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--model",
            "sim/m.sim",
            "--param",
            "p=1",
            "--output",
            "out.csv",
        ],
    );
    assert_eq!(out.trim_end(), "USE-001: run recorded");

    let entry = entry_of(&project, "USE-001");
    assert_eq!(entry.kind(), Some(Kind::Usage));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.title(), "ExampleSolver 2406 run for T-001");
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(entry.field("software").as_deref(), Some("ExampleSolver"));
    assert_eq!(entry.field("version").as_deref(), Some("2406"));
    assert_eq!(entry.field("task").as_deref(), Some("T-001"));
    assert_eq!(entry.field("model").as_deref(), Some("sim/m.sim"));
    assert_eq!(entry.field("model-checksum"), Some(checksum_of(MODEL)));
    assert_eq!(entry.field("howto"), None);
    assert_eq!(entry.section("Parameters").as_deref(), Some("p = 1"));
    assert_eq!(entry.section("Outputs").as_deref(), Some("out.csv"));
    assert_eq!(entry.section("Deviations"), None);
    assert_eq!(
        entry.history(),
        vec![
            format!("{WHEN} anna: created"),
            format!("{WHEN} anna: approved"),
        ]
    );
    assert_eq!(
        journal_of(&project, "USE-001"),
        vec![("created".to_string(), "approved".to_string())]
    );

    let list = list_of(&project);
    assert_eq!(list.keys(), ["sim/m.sim", "out.csv"]);
    assert_eq!(list.field("sim/m.sim", "role").as_deref(), Some("model"));
    assert_eq!(list.field("sim/m.sim", "made-from"), None);
    assert_eq!(
        list.field("sim/m.sim", "checksum"),
        Some(checksum_of(MODEL))
    );
    assert_eq!(list.field("out.csv", "role").as_deref(), Some("result"));
    assert_eq!(
        list.field("out.csv", "made-from"),
        Some(format!("sim/m.sim@{}", checksum_of(MODEL)))
    );
    assert_eq!(list.field("out.csv", "checksum"), Some(checksum_of(RESULT)));
    assert_eq!(
        list.history("sim/m.sim"),
        vec![format!("{WHEN} anna: attached")]
    );
    assert_eq!(
        list.history("out.csv"),
        vec![format!("{WHEN} anna: attached")]
    );

    let journal = list_journal(&project);
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].event, "attached");
    assert_eq!(journal[0].detail, "sim/m.sim, out.csv");
}

#[test]
fn a_run_by_an_llm_is_recorded_for_its_person_and_an_attached_output_is_described_again() {
    let project = Project::new();
    write_files(&project);
    project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--model",
            "sim/m.sim",
            "--output",
            "out.csv",
        ],
    );

    // The output is rewritten, and the same model is run again by the LLM of anna.
    project.write("out.csv", RESULT_LATER);
    let out = project.stdout(
        "anna-claude",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--model",
            "sim/m.sim",
            "--output",
            "out.csv",
        ],
    );
    assert_eq!(out.trim_end(), "USE-002: run recorded");

    let entry = entry_of(&project, "USE-002");
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.history(),
        vec![
            format!("{WHEN} anna via anna-claude: created"),
            format!("{WHEN} anna via anna-claude: approved"),
        ]
    );

    // The model is attached already: no second `attached` line for it, and the output is listed
    // once, with its new description and a `reattached` line.
    let list = list_of(&project);
    assert_eq!(list.keys(), ["sim/m.sim", "out.csv"]);
    assert_eq!(
        list.history("sim/m.sim"),
        vec![format!("{WHEN} anna: attached")]
    );
    assert_eq!(
        list.field("out.csv", "checksum"),
        Some(checksum_of(RESULT_LATER))
    );
    assert_eq!(
        list.field("out.csv", "made-from"),
        Some(format!("sim/m.sim@{}", checksum_of(MODEL)))
    );
    assert_eq!(
        list.history("out.csv"),
        vec![
            format!("{WHEN} anna: attached"),
            format!("{WHEN} anna via anna-claude: reattached"),
        ]
    );
    let journal = list_journal(&project);
    assert_eq!(journal.len(), 2);
    assert_eq!(journal[1].event, "reattached");
    assert_eq!(journal[1].detail, "out.csv");
}

#[test]
fn a_detached_output_is_attached_again_with_the_run_s_role_and_made_from() {
    let project = Project::new();
    write_files(&project);
    project.stdout(
        "anna",
        &[
            "attach", "T-001", "out.csv", "--role", "report", "--note", "old copy",
        ],
    );
    project.stdout("anna", &["detach", "T-001", "out.csv"]);

    project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--model",
            "sim/m.sim",
            "--output",
            "out.csv",
        ],
    );

    let list = list_of(&project);
    assert_eq!(list.field("out.csv", "detached"), None);
    assert_eq!(list.field("out.csv", "role").as_deref(), Some("result"));
    assert_eq!(list.field("out.csv", "note"), None);
    assert_eq!(
        list.field("out.csv", "made-from"),
        Some(format!("sim/m.sim@{}", checksum_of(MODEL)))
    );
    let history = list.history("out.csv");
    assert_eq!(
        history.last(),
        Some(&format!("{WHEN} anna: attached")),
        "the last line is the attach of the run"
    );
}

#[test]
fn a_run_without_a_model_attaches_its_outputs_without_a_made_from_entry() {
    let project = Project::new();
    write_files(&project);

    project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--output",
            "out.csv",
        ],
    );

    let entry = entry_of(&project, "USE-001");
    assert_eq!(entry.field("model"), None);
    assert_eq!(entry.field("model-checksum"), None);
    assert_eq!(entry.section("Outputs").as_deref(), Some("out.csv"));
    assert_eq!(entry.section("Parameters"), None);

    let list = list_of(&project);
    assert_eq!(list.keys(), ["out.csv"]);
    assert_eq!(list.field("out.csv", "role").as_deref(), Some("result"));
    assert_eq!(list.field("out.csv", "made-from"), None);
}

#[test]
fn the_options_are_written_into_the_record_and_the_how_to_must_exist() {
    let project = Project::new();
    write_files(&project);
    project.stdout(
        "anna",
        &["found", "Steady runs overpredict near the stability limit"],
    );

    let out = project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "t-001",
            "--howto",
            "fnd-001",
            "--title",
            "Speedline point 7",
            "--deviations",
            "Coarser mesh on the rotor.",
            "--note",
            "Second try of the day.",
        ],
    );
    assert_eq!(out.trim_end(), "USE-001: run recorded");

    let entry = entry_of(&project, "USE-001");
    assert_eq!(entry.title(), "Speedline point 7");
    assert_eq!(entry.field("task").as_deref(), Some("T-001"));
    assert_eq!(entry.field("howto").as_deref(), Some("FND-001"));
    assert_eq!(
        entry.section("Deviations").as_deref(),
        Some("Coarser mesh on the rotor.")
    );
    assert_eq!(
        entry.section("Notes").as_deref(),
        Some("Second try of the day.")
    );
    assert_eq!(entry.section("Parameters"), None);
    assert_eq!(entry.section("Outputs"), None);

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--howto",
                "LES-099",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no entry LES-099"));
    assert_eq!(usage_files(&project), 1);
}

#[test]
fn a_missing_model_or_a_bad_value_is_refused_and_nothing_is_written() {
    let project = Project::new();
    write_files(&project);

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--model",
                "sim/missing.sim",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("sim/missing.sim does not exist"));

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--param",
                "p",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("name=value"));

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--output",
                "../outside.csv",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("outside the project folder"));

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "bad task",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("a task is written like T-042"));

    // Nothing was given out or written: no record, no file list, no journal line.
    assert_eq!(usage_files(&project), 0);
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());
    assert!(journal_of(&project, "USE-001").is_empty());
    assert!(list_journal(&project).is_empty());
}

#[test]
fn a_run_needs_a_name_and_the_version_and_the_task() {
    let project = Project::new();
    write_files(&project);

    project
        .run(
            "",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
            ],
        )
        .failure()
        .code(2)
        .stderr(contains("--agent"));

    project
        .run("anna", &["used", "ExampleSolver", "--task", "T-001"])
        .failure()
        .code(2);
    assert_eq!(usage_files(&project), 0);
}

#[test]
fn json_gives_the_id_in_the_envelope() {
    let project = Project::new();
    write_files(&project);

    let out = project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--output",
            "out.csv",
            "--json",
        ],
    );
    let value: serde_json::Value = serde_json::from_str(out.trim_end()).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.used.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "USE-001");
    assert_eq!(value["warnings"], serde_json::json!([]));
}

#[test]
fn a_missing_output_is_refused_before_an_id_is_given_out() {
    let project = Project::new();
    write_files(&project);

    project
        .run(
            "anna",
            &[
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--output",
                "missing.csv",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("missing.csv does not exist"));
    assert_eq!(usage_files(&project), 0);
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());

    // The number was not used up: the next run is USE-001.
    let out = project.stdout(
        "anna",
        &[
            "used",
            "ExampleSolver",
            "--version",
            "2406",
            "--task",
            "T-001",
            "--output",
            "out.csv",
        ],
    );
    assert_eq!(out.trim_end(), "USE-001: run recorded");
}

#[test]
fn a_dry_run_writes_nothing() {
    let project = Project::new();
    write_files(&project);

    project
        .run(
            "anna",
            &[
                "--dry-run",
                "used",
                "ExampleSolver",
                "--version",
                "2406",
                "--task",
                "T-001",
                "--model",
                "sim/m.sim",
                "--output",
                "out.csv",
            ],
        )
        .success()
        .stdout(contains("USE-NEW: run recorded"));
    assert_eq!(usage_files(&project), 0);
    assert!(!project.root().join("DOSSIER/files/T-001.md").exists());
    assert!(journal_of(&project, "USE-NEW").is_empty());
}

#[test]
fn a_changed_model_that_is_attached_already_is_described_again() {
    let project = Project::new();
    write_files(&project);
    let run = [
        "used",
        "ExampleSolver",
        "--version",
        "2406",
        "--task",
        "T-001",
        "--model",
        "sim/m.sim",
        "--output",
        "out.csv",
    ];
    project.stdout("anna", &run);
    project.write("sim/m.sim", "invented model data, second version\n");

    project.stdout("anna", &run);

    let list = list_of(&project);
    assert_eq!(
        list.field("sim/m.sim", "checksum"),
        Some(checksum_of("invented model data, second version\n"))
    );
    assert!(
        list.history("sim/m.sim")
            .iter()
            .any(|line| line.ends_with("reattached"))
    );
    let entry = entry_of(&project, "USE-002");
    assert_eq!(
        entry.field("model-checksum"),
        list.field("sim/m.sim", "checksum")
    );
}
