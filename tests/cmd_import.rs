//! Runs the built program for `import status`: the documents found in the project folder (or the
//! folder the settings describe) with their review state in imports.md, the imported entries that
//! wait for approval, are approved or are promoted, the JSON data, a broken entry file as a
//! warning, and the refusal of `--team`. Each test runs in its own temporary project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// Invented texts. The reviewed document has a space and a non-ASCII letter in its name.
const DONE_DOC: &str = "docs/Prüfstand report.pdf";
const OTHER_DOC: &str = "docs/old notes.txt";
const REPORT: &str = "Invented test report: the rig stalls above a tip Mach number of 1.1.";
const NOTES: &str = "Invented old meeting notes.";
const STATEMENT: &str = "Invented: the rig stalls above a tip Mach number of 1.1.";
const OWN_STATEMENT: &str = "Invented: a coarse mesh overpredicts the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented test report, section 2.";

/// The standard output of `import status` in the folder `folder`, one line each, as the command
/// prints it: the counts of the documents, then the counts of the imported entries.
fn status_text(folder: &str, lines: [usize; 8]) -> String {
    let [
        documents,
        not_reviewed,
        reviewing,
        done,
        nothing,
        drafts,
        approved,
        promoted,
    ] = lines;
    [
        format!("Import status of P-001 (folder {folder})"),
        format!("Documents found: {documents}"),
        format!("  not reviewed: {not_reviewed}"),
        format!("  being reviewed: {reviewing}"),
        format!("  done: {done}"),
        format!("  nothing to take: {nothing}"),
        "Imported entries:".to_string(),
        format!("  drafts waiting for approval: {drafts}"),
        format!("  approved: {approved}"),
        format!("  promoted to the team: {promoted}"),
    ]
    .join("\n")
        + "\n"
}

/// Marks `document` as reviewed: review-document by the LLM, then reviewed-document by anna.
fn review_done(project: &Project, document: &str) {
    project
        .run("anna-claude", &["review-document", document])
        .success();
    project
        .run("anna", &["reviewed-document", document])
        .success();
}

/// Drafts a lesson from `document` as anna-claude (an imported draft).
fn draft_from(project: &Project, statement: &str, document: &str) {
    project
        .run(
            "anna-claude",
            &[
                "learn",
                statement,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
                "--from-document",
                document,
            ],
        )
        .success();
}

/// The invented project of most tests: DONE_DOC is reviewed and done, OTHER_DOC is not reviewed,
/// and one lesson is drafted from DONE_DOC (LES-001) and one lesson is written without a document
/// (LES-002), so one imported draft waits for approval.
fn project_with_one_done_and_one_draft() -> Project {
    let project = Project::new();
    project.write(DONE_DOC, REPORT);
    project.write(OTHER_DOC, NOTES);
    review_done(&project, DONE_DOC);
    draft_from(&project, STATEMENT, DONE_DOC);
    project
        .run(
            "anna-claude",
            &[
                "learn",
                OWN_STATEMENT,
                "--valid-for",
                VALID_FOR,
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
            ],
        )
        .success();
    project
}

#[test]
fn one_document_done_and_one_imported_draft_are_counted() {
    let project = project_with_one_done_and_one_draft();

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text(".", [2, 1, 0, 1, 0, 1, 0, 0])
    );
}

#[test]
fn json_gives_the_same_numbers_and_the_schema() {
    let project = project_with_one_done_and_one_draft();

    let stdout = project.stdout("anna", &["import", "status", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("the JSON parses");

    assert_eq!(value["schema"], "dossier.import.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!({
            "folder": ".",
            "documents": 2,
            "not_reviewed": 1,
            "reviewing": 0,
            "done": 1,
            "nothing": 0,
            "imported_drafts": 1,
            "imported_approved": 0,
            "imported_promoted": 0,
        })
    );
}

#[test]
fn an_approved_imported_lesson_is_approved_and_promoted_when_its_field_says_so() {
    let project = project_with_one_done_and_one_draft();
    project
        .command("anna", "2026-11-20T10:00:00+01:00")
        .env("DOSSIER_ANSWERS", "a")
        .args(["approve", "LES-001"])
        .assert()
        .success();
    project
        .run(
            "anna",
            &["set", "LES-001", "--field", "promoted=team:LES-001"],
        )
        .success();

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text(".", [2, 1, 0, 1, 0, 0, 1, 1])
    );
}

#[test]
fn documents_being_reviewed_and_with_nothing_to_take_are_counted() {
    let project = Project::new();
    project.write(DONE_DOC, REPORT);
    project.write(OTHER_DOC, NOTES);
    project.write("docs/third.pdf", REPORT);
    project
        .run("anna-claude", &["review-document", DONE_DOC])
        .success();
    project
        .run("anna-claude", &["review-document", OTHER_DOC])
        .success();
    project
        .run("anna", &["reviewed-document", OTHER_DOC, "--nothing"])
        .success();

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text(".", [3, 1, 1, 0, 1, 0, 0, 0])
    );
}

#[test]
fn a_project_with_no_documents_and_no_entries_has_only_zeros() {
    let project = Project::new();

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text(".", [0; 8])
    );
}

#[test]
fn the_folder_the_settings_describe_is_the_one_counted() {
    let project = Project::new();
    project.write("archive/Final report.pdf", REPORT);
    project.write("archive/minutes.txt", NOTES);
    project.write("docs/outside.pdf", REPORT);
    let settings = project.read("DOSSIER/dossier.toml");
    let described = settings.replacen(
        "code = \"P-001\"\n",
        "code = \"P-001\"\ndescribes = \"archive\"\n",
        1,
    );
    assert_ne!(settings, described, "the settings name the folder");
    project.write("DOSSIER/dossier.toml", &described);
    // The keys of imports.md are paths from the described folder, as review-folder reads them.
    project.write(
        "DOSSIER/imports.md",
        "<!-- dossier-format: 1 -->\n# Documents of P-001\n\n## Final report.pdf\n\n- state: done\n- history: 2026-11-19T09:00:00+01:00 anna: changed — state done\n",
    );

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text("archive", [2, 1, 0, 1, 0, 0, 0, 0])
    );
}

#[test]
fn a_broken_entry_file_gives_a_warning_and_the_counts_still_work() {
    let project = project_with_one_done_and_one_draft();
    project.write(
        "DOSSIER/lessons/LES-009-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        project.stdout("anna", &["import", "status"]),
        status_text(".", [2, 1, 0, 1, 0, 1, 0, 0])
    );
    project
        .run("anna", &["import", "status"])
        .success()
        .stderr(contains(
            "warning: DOSSIER/lessons/LES-009-broken.md: line 2: no heading",
        ));
}

#[test]
fn team_is_refused_because_the_imports_belong_to_the_project() {
    let project = project_with_one_done_and_one_draft();

    project
        .run("anna", &["--team", "import", "status"])
        .failure()
        .stderr(contains("import belongs to a project; run without --team"));
}
