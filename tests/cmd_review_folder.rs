//! Runs the built program for `review-folder`: the tree and the documents of a folder, the review
//! state each document has in imports.md, the count reviewed, the import rules, the examples, a
//! path argument, a folder that the settings describe, and the refusals. Each test runs in its own
//! temporary project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.
//! The times of the test files are set, so the dates shown are the same on every run.

mod common;

use common::Project;
use serde_json::Value;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Invented texts. Their sizes in bytes are their lengths.
const REPORT: &str = "Invented test report: the rig stalls above a tip Mach number of 1.1.";
const SCRIPT: &str = "print('invented test script')\n";
const NOTES: &str = "Invented old meeting notes.";

/// The modification times of the test files: seconds after the Unix epoch. The report is from
/// 2026-11-10, the script from 2026-11-11 (the newest) and the notes from 2026-11-01 (the oldest).
const REPORT_TIME: u64 = 1_794_312_000;
const SCRIPT_TIME: u64 = 1_794_398_400;
const NOTES_TIME: u64 = 1_793_534_400;

/// The import rules, one numbered line each, as the command prints them (R-IMPORT-4).
const RULES: [&str; 7] = [
    "1. Draft only what a later project would use.",
    "2. Prefer the report's conclusions and the reasons behind choices over descriptions of what was done.",
    "3. Record dead ends.",
    "4. Give the page the entry comes from.",
    "5. Set confidence to what the document supports (measured, simulated, experience or guess), and use guess when it only implies something.",
    "6. Write in English, also when the document is not in English.",
    "7. Never copy customer names or product names into the text of a lesson.",
];

/// The example commands, as the command prints them.
const EXAMPLES: [&str; 2] = [
    "dossier review-document <file>",
    "dossier learn <statement> --valid-for <text> --evidence <text> --confidence <level> --from-document <file>",
];

/// imports.md of the invented project: the report is done, the script is being reviewed and the
/// notes hold nothing worth keeping. The keys are the stored paths from the project folder.
const IMPORTS: &str = "<!-- dossier-format: 1 -->\n# Documents of P-001\n\n## docs/Final report.pdf\n\n- state: done\n- history: 2026-11-19T09:00:00+01:00 anna: changed — state done\n\n## tools/fit.py\n\n- state: reviewing\n- history: 2026-11-19T10:00:00+01:00 ben: changed — state reviewing\n\n## old/notes.txt\n\n- state: nothing\n- history: 2026-11-19T11:00:00+01:00 ben: changed — state nothing\n";

/// A path as the command shows it on this system: `/` becomes `\` on Windows.
fn shown(path: &str) -> String {
    if cfg!(windows) {
        path.replace('/', "\\")
    } else {
        path.to_string()
    }
}

/// Sets the modification time of `path` to `seconds` after the Unix epoch.
fn set_time(path: &Path, seconds: u64) {
    let when = SystemTime::UNIX_EPOCH + Duration::from_secs(seconds);
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("open a test file")
        .set_modified(when)
        .expect("set the modification time");
}

/// The local date of `seconds` after the Unix epoch, as the command shows it (format.md §7).
fn local_date(seconds: u64) -> String {
    let when = SystemTime::UNIX_EPOCH + Duration::from_secs(seconds);
    chrono::DateTime::<chrono::Local>::from(when)
        .format("%Y-%m-%d")
        .to_string()
}

/// The lines under the heading `heading` of the standard output, up to the next heading.
fn section(text: &str, heading: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("## ") {
            inside = line == heading;
        } else if inside && !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    lines
}

/// A project folder with a report in docs/, a script in tools/ and old notes in old/, each with
/// its fixed modification time. Nothing is listed in imports.md.
fn project_with_material() -> Project {
    let project = Project::new();
    project.write("docs/Final report.pdf", REPORT);
    project.write("tools/fit.py", SCRIPT);
    project.write("old/notes.txt", NOTES);
    set_time(&project.root().join("docs/Final report.pdf"), REPORT_TIME);
    set_time(&project.root().join("tools/fit.py"), SCRIPT_TIME);
    set_time(&project.root().join("old/notes.txt"), NOTES_TIME);
    project
}

/// The document lines of the three files of `project_with_material`, newest first, with a state.
fn document_lines(report_state: &str, script_state: &str, notes_state: &str) -> Vec<String> {
    vec![
        format!(
            "{}  script  {}  {}  [{script_state}]",
            shown("tools/fit.py"),
            SCRIPT.len(),
            local_date(SCRIPT_TIME)
        ),
        format!(
            "{}  report  {}  {}  [{report_state}]",
            shown("docs/Final report.pdf"),
            REPORT.len(),
            local_date(REPORT_TIME)
        ),
        format!(
            "{}  report  {}  {}  [{notes_state}]",
            shown("old/notes.txt"),
            NOTES.len(),
            local_date(NOTES_TIME)
        ),
    ]
}

#[test]
fn a_report_and_a_script_are_listed_with_kind_size_and_date() {
    let project = project_with_material();
    let text = project.stdout("anna", &["review-folder"]);

    assert!(text.starts_with("# Folder .\n"), "{text}");
    let documents = section(&text, "## Documents");
    let report = format!(
        "{}  report  {}  {}  [not reviewed]",
        shown("docs/Final report.pdf"),
        REPORT.len(),
        local_date(REPORT_TIME)
    );
    let script = format!(
        "{}  script  {}  {}  [not reviewed]",
        shown("tools/fit.py"),
        SCRIPT.len(),
        local_date(SCRIPT_TIME)
    );
    assert!(documents.contains(&report), "{text}");
    assert!(documents.contains(&script), "{text}");
    assert!(
        !text.contains("DOSSIER"),
        "the dossier folder is not listed:\n{text}"
    );
}

#[test]
fn the_documents_are_listed_newest_first() {
    let project = project_with_material();
    let text = project.stdout("anna", &["review-folder"]);

    assert_eq!(
        section(&text, "## Documents"),
        document_lines("not reviewed", "not reviewed", "not reviewed")
    );
}

#[test]
fn the_states_come_from_imports_md_and_the_reviewed_are_counted() {
    let project = project_with_material();
    project.write("DOSSIER/imports.md", IMPORTS);
    let text = project.stdout("anna", &["review-folder"]);

    assert_eq!(
        section(&text, "## Documents"),
        document_lines("done", "reviewing", "nothing")
    );
    assert_eq!(
        section(&text, "## Already reviewed"),
        ["2 of 3 documents reviewed (done or nothing), 1 being reviewed."]
    );
}

#[test]
fn the_tree_lists_to_the_depth_and_the_default_is_three() {
    let project = project_with_material();
    project.write("a/b/c/deep.png", "invented picture");
    let deep = format!("deep.png  {}", "invented picture".len());

    assert_eq!(
        section(
            &project.stdout("anna", &["review-folder", "--depth", "1"]),
            "## Tree"
        ),
        ["a/", "docs/", "old/", "tools/"]
    );

    let default = project.stdout("anna", &["review-folder"]);
    let tree = section(&default, "## Tree");
    assert!(tree.contains(&"    c/".to_string()), "{default}");
    assert!(
        !tree.iter().any(|line| line.contains("deep.png")),
        "{default}"
    );

    let deeper = project.stdout("anna", &["review-folder", "--depth", "4"]);
    assert!(
        section(&deeper, "## Tree").contains(&format!("      {deep}")),
        "{deeper}"
    );
}

#[test]
fn the_import_rules_are_printed_as_a_numbered_list() {
    let project = project_with_material();
    let text = project.stdout("anna", &["review-folder"]);

    assert_eq!(section(&text, "## Import rules"), RULES);
}

#[test]
fn the_examples_are_the_review_and_the_learn_commands() {
    let project = project_with_material();
    let text = project.stdout("anna", &["review-folder"]);

    assert_eq!(section(&text, "## Examples"), EXAMPLES);
}

#[test]
fn a_path_argument_reviews_that_subfolder_with_the_keys_of_the_project() {
    let project = project_with_material();
    project.write("DOSSIER/imports.md", IMPORTS);
    let text = project.stdout("anna", &["review-folder", "docs"]);

    assert!(text.starts_with("# Folder docs\n"), "{text}");
    assert_eq!(
        section(&text, "## Documents"),
        [format!(
            "{}  report  {}  {}  [done]",
            shown("docs/Final report.pdf"),
            REPORT.len(),
            local_date(REPORT_TIME)
        )]
    );
    assert_eq!(
        section(&text, "## Already reviewed"),
        ["1 of 1 documents reviewed (done or nothing), 0 being reviewed."]
    );
    assert!(!text.contains("fit.py"), "{text}");
}

#[test]
fn a_folder_with_spaces_and_non_ascii_letters_works() {
    let project = Project::new();
    project.write("Prüf stand/Ergebnis ü.pdf", REPORT);
    set_time(
        &project.root().join("Prüf stand").join("Ergebnis ü.pdf"),
        REPORT_TIME,
    );
    let text = project.stdout("anna", &["review-folder", "Prüf stand"]);

    assert!(text.starts_with("# Folder Prüf stand\n"), "{text}");
    assert_eq!(
        section(&text, "## Documents"),
        [format!(
            "{}  report  {}  {}  [not reviewed]",
            shown("Prüf stand/Ergebnis ü.pdf"),
            REPORT.len(),
            local_date(REPORT_TIME)
        )]
    );
}

#[test]
fn the_folder_the_settings_describe_is_the_default() {
    let project = Project::new();
    let old = tempfile::tempdir().expect("a folder for the old project");
    std::fs::create_dir_all(old.path().join("docs")).expect("create docs");
    std::fs::write(old.path().join("docs").join("Old report.docx"), REPORT)
        .expect("write the old report");
    set_time(
        &old.path().join("docs").join("Old report.docx"),
        REPORT_TIME,
    );

    // `init --in` writes describes after the code line, before the tables (format.md §26).
    let described = old.path().to_string_lossy().replace('\\', "/");
    let settings = project.read("DOSSIER/dossier.toml");
    let settings = settings.replacen(
        "code = \"P-001\"\n",
        &format!("code = \"P-001\"\ndescribes = \"{described}\"\n"),
        1,
    );
    project.write("DOSSIER/dossier.toml", &settings);

    let text = project.stdout("anna", &["review-folder"]);
    assert!(
        text.starts_with(&format!("# Folder {}\n", old.path().display())),
        "{text}"
    );
    assert_eq!(
        section(&text, "## Documents"),
        [format!(
            "{}  report  {}  {}  [not reviewed]",
            shown("docs/Old report.docx"),
            REPORT.len(),
            local_date(REPORT_TIME)
        )]
    );
}

#[test]
fn json_gives_the_documents_their_states_and_the_counts() {
    let project = project_with_material();
    project.write("DOSSIER/imports.md", IMPORTS);
    let output = project.stdout("anna", &["--json", "review-folder"]);
    let value: Value = serde_json::from_str(output.trim()).expect("one JSON line");

    assert_eq!(value["schema"], "dossier.review-folder.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["reviewed"], 2);
    assert_eq!(value["data"]["being_reviewed"], 1);
    let documents = value["data"]["documents"]
        .as_array()
        .expect("a list of documents");
    let report = documents
        .iter()
        .find(|document| document["stored"] == "docs/Final report.pdf")
        .expect("the report is listed");
    assert_eq!(report["kind"], "report");
    assert_eq!(report["size"], REPORT.len());
    assert_eq!(report["state"], "done");
    let notes = documents
        .iter()
        .find(|document| document["stored"] == "old/notes.txt")
        .expect("the notes are listed");
    assert_eq!(notes["state"], "nothing");
    assert_eq!(value["data"]["rules"].as_array().map(Vec::len), Some(7));
}

#[test]
fn a_path_outside_the_project_is_refused() {
    let project = project_with_material();
    project
        .run("anna", &["review-folder", ".."])
        .failure()
        .code(1)
        .stderr(predicates::str::contains("outside"));
}

#[test]
fn a_missing_folder_is_an_io_error_that_names_it() {
    let project = project_with_material();
    project
        .run("anna", &["review-folder", "missing"])
        .failure()
        .code(4)
        .stderr(predicates::str::contains("missing"));
}

#[test]
fn reading_changes_nothing_and_needs_no_name() {
    let project = project_with_material();
    project.write("DOSSIER/imports.md", IMPORTS);
    let journal = project.read("DOSSIER/.state/journal.jsonl");
    let settings = project.read("DOSSIER/dossier.toml");

    project.run("", &["review-folder"]).success();

    assert_eq!(project.read("DOSSIER/.state/journal.jsonl"), journal);
    assert_eq!(project.read("DOSSIER/dossier.toml"), settings);
    assert_eq!(project.read("DOSSIER/imports.md"), IMPORTS);
}
