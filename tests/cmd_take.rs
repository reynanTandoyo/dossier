//! Runs the built program for `take`: a file is recorded as taken in taken.md with its size, time,
//! checksum, task and who took it; a file another person holds is refused unless `--force`, which
//! records the take as forced; a file the caller holds already is left as it is. Each test runs in
//! its own temporary project folder with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::blockfile::BlockFile;
use dossier::hash::sha256_hex;
use dossier::journal::Record;
use dossier::paths::shown;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history and journal lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// The stored path of the taken file of most tests.
const PATH: &str = "runs/case-a.dat";

/// Another file of the tests.
const OTHER: &str = "runs/case-b.dat";

/// Invented contents of the files of the tests.
const DATA: &str = "invented simulation data\n";

/// The text of the file of taken files of the project.
fn taken_text(project: &Project) -> String {
    project.read("DOSSIER/taken.md")
}

/// The file of taken files, parsed.
fn taken_of(project: &Project) -> BlockFile {
    BlockFile::parse(&taken_text(project)).expect("the taken files parse")
}

/// The checksum of some text, as the record holds it (format.md §18).
fn checksum_of(text: &str) -> String {
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// The journal records of the taken files, oldest first.
fn taken_journal(project: &Project) -> Vec<Record> {
    project
        .entry_journal()
        .into_iter()
        .filter(|record| record.id == "TAKEN")
        .collect()
}

#[test]
fn take_records_who_task_size_time_and_checksum_of_the_file() {
    let project = Project::new();
    project.write(PATH, DATA);
    let output = project.stdout("anna", &["take", PATH, "--task", "T-001"]);
    assert_eq!(
        output,
        format!(
            "taken for T-001: 1 file\n  {}  {} bytes  {}\n",
            shown(PATH),
            DATA.len(),
            checksum_of(DATA)
        )
    );

    let file = taken_of(&project);
    assert_eq!(file.keys(), [PATH]);
    assert_eq!(
        file.field(PATH, "taken").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by anna")
    );
    assert_eq!(file.field(PATH, "task").as_deref(), Some("T-001"));
    assert_eq!(file.field(PATH, "size"), Some(DATA.len().to_string()));
    let modified = file.field(PATH, "modified").expect("a modification time");
    assert!(dossier::time::valid(&modified), "{modified}");
    assert_eq!(file.field(PATH, "checksum"), Some(checksum_of(DATA)));
    assert!(file.field(PATH, "forced").is_none());
    assert_eq!(
        file.history(PATH),
        [format!("{WHEN} anna: taken — T-001 {}", checksum_of(DATA))]
    );

    let journal = taken_journal(&project);
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].name, "anna");
    assert!(!journal[0].llm);
    assert_eq!(journal[0].event, "taken");
    assert_eq!(journal[0].detail, PATH);
}

#[test]
fn a_file_another_person_holds_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project.write(PATH, DATA);
    project.write(OTHER, DATA);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);
    let before = taken_text(&project);

    project
        .run("ben", &["take", OTHER, PATH, "--task", "T-002"])
        .code(1)
        .stderr(contains(format!(
            "{} is taken by anna for T-001 since {WHEN}; ask them, or add --force",
            shown(PATH)
        )));

    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
    assert_eq!(taken_journal(&project).len(), 1);
}

#[test]
fn ben_with_force_takes_the_file_and_the_block_is_marked_forced() {
    let project = Project::new();
    project.write(PATH, DATA);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);

    let output = project.stdout("ben", &["take", PATH, "--task", "T-002", "--force"]);
    assert_eq!(
        output,
        format!(
            "taken for T-002: 1 file\n  {}  {} bytes  {}\n",
            shown(PATH),
            DATA.len(),
            checksum_of(DATA)
        )
    );

    let file = taken_of(&project);
    assert_eq!(
        file.field(PATH, "taken").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by ben")
    );
    assert_eq!(file.field(PATH, "task").as_deref(), Some("T-002"));
    assert_eq!(file.field(PATH, "forced").as_deref(), Some("yes"));
    assert_eq!(
        file.history(PATH),
        [
            format!("{WHEN} anna: taken — T-001 {}", checksum_of(DATA)),
            format!(
                "{WHEN} ben: taken by force — T-002 {}, from anna",
                checksum_of(DATA)
            ),
        ]
    );

    let journal = taken_journal(&project);
    assert_eq!(journal.len(), 2);
    assert_eq!(journal[1].name, "ben");
    assert_eq!(journal[1].event, "taken");
}

#[test]
fn taking_a_file_twice_changes_nothing() {
    let project = Project::new();
    project.write(PATH, DATA);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);
    let before = taken_text(&project);

    let output = project.stdout("anna", &["take", PATH, "--task", "T-001"]);
    assert_eq!(
        output,
        format!(
            "taken for T-001: 1 file\n  {}  already taken by you\n",
            shown(PATH)
        )
    );
    assert_eq!(taken_text(&project), before, "taken.md is unchanged");
    assert_eq!(taken_journal(&project).len(), 1);
}

#[test]
fn a_file_already_held_is_reported_and_the_new_one_is_taken() {
    let project = Project::new();
    project.write(PATH, DATA);
    let second = "second file\n";
    project.write(OTHER, second);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);

    let output = project.stdout("anna", &["take", PATH, OTHER, "--task", "T-001"]);
    assert_eq!(
        output,
        format!(
            "taken for T-001: 2 files\n  {}  already taken by you\n  {}  {} bytes  {}\n",
            shown(PATH),
            shown(OTHER),
            second.len(),
            checksum_of(second)
        )
    );
    assert_eq!(taken_of(&project).keys(), [PATH, OTHER]);
    let journal = taken_journal(&project);
    assert_eq!(journal.len(), 2);
    assert_eq!(journal[1].detail, OTHER, "only the file taken now is named");
}

#[test]
fn a_missing_file_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &["take", "runs/missing.dat", "--task", "T-001"])
        .code(1)
        .stderr(contains(format!(
            "{} does not exist",
            shown("runs/missing.dat")
        )));
    assert!(!project.root().join("DOSSIER/taken.md").exists());
    assert!(taken_journal(&project).is_empty());
}

#[test]
fn a_folder_is_refused_because_take_takes_files() {
    let project = Project::new();
    project.write("runs/folder/inside.dat", DATA);
    project
        .run("anna", &["take", "runs/folder", "--task", "T-001"])
        .code(1)
        .stderr(contains(format!(
            "{} is not a file; take takes files",
            shown("runs/folder")
        )));
    assert!(!project.root().join("DOSSIER/taken.md").exists());
}

#[test]
fn a_path_with_a_space_and_a_non_ascii_letter_is_taken_like_any_other() {
    let project = Project::new();
    let path = "Läufe/Lauf 01 — Entwurf.dat";
    project.write(path, DATA);
    let output = project.stdout("anna", &["take", path, "--task", "T-001"]);
    assert!(output.contains(&shown(path)), "{output}");

    let file = taken_of(&project);
    assert_eq!(file.keys(), [path]);
    assert_eq!(file.field(path, "checksum"), Some(checksum_of(DATA)));
}

#[test]
fn an_llm_takes_for_its_person() {
    let project = Project::new();
    project.write(PATH, DATA);
    project.stdout("anna - claude", &["take", PATH, "--task", "T-001"]);

    let file = taken_of(&project);
    assert_eq!(
        file.field(PATH, "taken").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by anna")
    );
    assert_eq!(
        file.history(PATH),
        [format!(
            "{WHEN} anna via anna - claude: taken — T-001 {}",
            checksum_of(DATA)
        )]
    );
    let journal = taken_journal(&project);
    assert_eq!(journal[0].name, "anna - claude");
    assert!(journal[0].llm);
}

#[test]
fn a_returned_file_is_taken_again_and_keeps_its_history() {
    let project = Project::new();
    project.write(PATH, DATA);
    project.stdout("anna", &["take", PATH, "--task", "T-001"]);
    // What `return` will do when it ends a take: the block stays, its `taken` field goes.
    let without_taken: String = taken_text(&project)
        .split_inclusive('\n')
        .filter(|line| !line.starts_with("- taken:"))
        .collect();
    project.write("DOSSIER/taken.md", &without_taken);

    project.stdout("ben", &["take", PATH, "--task", "T-002"]);

    let file = taken_of(&project);
    assert_eq!(file.keys(), [PATH]);
    assert_eq!(
        file.field(PATH, "taken").as_deref(),
        Some("2026-11-20T10:00:00+01:00 by ben")
    );
    assert_eq!(file.field(PATH, "task").as_deref(), Some("T-002"));
    assert!(file.field(PATH, "forced").is_none());
    assert_eq!(
        file.history(PATH),
        [
            format!("{WHEN} anna: taken — T-001 {}", checksum_of(DATA)),
            format!("{WHEN} ben: taken — T-002 {}", checksum_of(DATA)),
        ]
    );
}

#[test]
fn json_output_gives_the_task_and_each_file() {
    let project = Project::new();
    project.write(PATH, DATA);
    let output = project.stdout("anna", &["take", PATH, "--task", "T-001", "--json"]);
    let value: serde_json::Value =
        serde_json::from_str(output.trim_end()).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.take.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["task"], "T-001");
    assert_eq!(value["data"]["files"][0]["path"], PATH);
    assert_eq!(value["data"]["files"][0]["size"], DATA.len());
    assert_eq!(value["data"]["files"][0]["checksum"], checksum_of(DATA));
    assert_eq!(value["data"]["files"][0]["already"], false);
}

// Windows cannot create a file whose name has `?`, so the file is made on Unix only.
#[cfg(unix)]
#[test]
fn a_name_windows_cannot_have_is_a_warning() {
    let project = Project::new();
    let path = "runs/what?.dat";
    project.write(path, DATA);
    project
        .run("anna", &["take", path, "--task", "T-001"])
        .success()
        .stderr(contains(format!(
            "{} cannot be copied to a Windows PC",
            shown(path)
        )));
}

#[test]
fn paths_that_differ_only_in_case_are_a_warning() {
    let project = Project::new();
    project.write("runs/case.dat", DATA);
    project.write("runs/CASE.dat", DATA);
    project
        .run(
            "anna",
            &["take", "runs/case.dat", "runs/CASE.dat", "--task", "T-001"],
        )
        .success()
        .stderr(contains("differ only in upper and lower case"));
}
