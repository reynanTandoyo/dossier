//! Runs the built program for `review-document` and `reviewed-document`: an old document is marked
//! as being reviewed and then as done, or as holding nothing worth keeping (R-IMPORT-3), and the
//! import rules are printed for the LLM (R-IMPORT-4). Each test runs in its own project folder,
//! with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The document of most tests: an invented file in the project folder.
const DOC: &str = "docs/spec-rev1.pdf";

/// The first lines of imports.md for project P-001.
const HEADING: &str = "<!-- dossier-format: 1 -->\n# Documents of P-001\n\n";

/// The text of imports.md of the project.
fn imports_text(project: &Project) -> String {
    project.read("DOSSIER/imports.md")
}

/// A stored path (with /) as the text output shows it: with the separator of this system.
fn shown(stored: &str) -> String {
    if cfg!(windows) {
        stored.replace('/', "\\")
    } else {
        stored.to_string()
    }
}

#[test]
fn review_then_reviewed_marks_the_document_done() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("anna-claude", &["review-document", DOC])
        .success()
        .stdout(contains(format!("{}: reviewing", shown(DOC))))
        .stdout(contains("- Draft only what a later project would use."))
        .stdout(contains("draft with: --from-document docs/spec-rev1.pdf"));
    let reviewing = format!(
        "{HEADING}## docs/spec-rev1.pdf\n\n- state: reviewing\n- history: {NOW} anna via anna-claude: reviewing\n\n"
    );
    assert_eq!(imports_text(&project), reviewing);

    project
        .run("anna", &["reviewed-document", DOC])
        .success()
        .stdout(contains(format!("{}: done", shown(DOC))));
    let done = format!(
        "{HEADING}## docs/spec-rev1.pdf\n\n- state: done\n- history: {NOW} anna via anna-claude: reviewing\n- history: {NOW} anna: reviewed\n\n"
    );
    assert_eq!(imports_text(&project), done);

    let records = project.entry_journal();
    let last = records.last().expect("the change is in the journal");
    assert_eq!(last.id, "IMPORTS");
    assert_eq!(last.event, "reviewed");
    assert_eq!(last.detail, "docs/spec-rev1.pdf");
    assert_eq!(last.name, "anna");
    assert_eq!(last.command, "reviewed-document");
}

#[test]
fn reviewed_document_with_nothing_marks_it_as_holding_nothing() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("anna-claude", &["review-document", DOC])
        .success();
    project
        .run("anna", &["reviewed-document", DOC, "--nothing"])
        .success()
        .stdout(contains(format!("{}: nothing", shown(DOC))));
    let expected = format!(
        "{HEADING}## docs/spec-rev1.pdf\n\n- state: nothing\n- history: {NOW} anna via anna-claude: reviewing\n- history: {NOW} anna: nothing\n\n"
    );
    assert_eq!(imports_text(&project), expected);

    let records = project.entry_journal();
    let last = records.last().expect("the change is in the journal");
    assert_eq!(last.event, "nothing");
}

#[test]
fn reviewing_a_done_document_again_keeps_its_history() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("anna-claude", &["review-document", DOC])
        .success();
    project.run("anna", &["reviewed-document", DOC]).success();
    project
        .run("anna-claude", &["review-document", DOC])
        .success();
    let expected = format!(
        "{HEADING}## docs/spec-rev1.pdf\n\n- state: reviewing\n- history: {NOW} anna via anna-claude: reviewing\n- history: {NOW} anna: reviewed\n- history: {NOW} anna via anna-claude: reviewing\n\n"
    );
    assert_eq!(imports_text(&project), expected);
}

#[test]
fn a_document_is_added_to_a_file_that_already_lists_others_and_they_stay_as_they_are() {
    let project = Project::new();
    let existing = format!(
        "{HEADING}## old/minutes.txt\n\n- state: nothing\n- history: 2026-11-06T11:20:00+01:00 ben via ben-codex: nothing\n\n"
    );
    project.write("DOSSIER/imports.md", &existing);
    project.write(DOC, "invented report");
    project
        .run("anna-claude", &["review-document", DOC])
        .success();
    let expected = format!(
        "{existing}## docs/spec-rev1.pdf\n\n- state: reviewing\n- history: {NOW} anna via anna-claude: reviewing\n\n"
    );
    assert_eq!(imports_text(&project), expected);
}

#[test]
fn a_document_with_a_space_and_a_non_ascii_letter_works_and_is_quoted_in_the_hint() {
    let project = Project::new();
    let name = "docs/Prüfstand notes.docx";
    project.write(name, "invented notes");
    project
        .run("anna-claude", &["review-document", name])
        .success()
        .stdout(contains(
            "draft with: --from-document \"docs/Prüfstand notes.docx\"",
        ));
    project
        .run("anna", &["reviewed-document", name])
        .success()
        .stdout(contains(format!("{}: done", shown(name))));
    let text = imports_text(&project);
    assert!(text.contains("## docs/Prüfstand notes.docx\n\n- state: done\n"));
}

#[test]
fn reviewed_document_refuses_a_document_that_is_not_listed_and_writes_nothing() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("anna", &["reviewed-document", DOC])
        .failure()
        .code(1)
        .stderr(contains("is not being reviewed"));
    assert!(!project.root().join("DOSSIER/imports.md").exists());
}

#[test]
fn review_document_refuses_a_file_that_does_not_exist_and_writes_nothing() {
    let project = Project::new();
    project
        .run("anna-claude", &["review-document", "docs/missing.pdf"])
        .failure()
        .code(1)
        .stderr(contains("no document at docs/missing.pdf"));
    assert!(!project.root().join("DOSSIER/imports.md").exists());
}

#[test]
fn a_document_outside_the_project_folder_is_refused() {
    let project = Project::new();
    project
        .run("anna-claude", &["review-document", "../outside.pdf"])
        .failure()
        .code(1)
        .stderr(contains("the path is outside the project folder"));
    assert!(!project.root().join("DOSSIER/imports.md").exists());
}

#[test]
fn review_document_without_a_name_is_a_usage_error() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("", &["review-document", DOC])
        .failure()
        .code(2)
        .stderr(contains("needs a name"));
}

#[test]
fn a_dry_run_writes_nothing() {
    let project = Project::new();
    project.write(DOC, "invented report");
    project
        .run("anna-claude", &["--dry-run", "review-document", DOC])
        .success()
        .stderr(contains("dry run: would write"));
    assert!(!project.root().join("DOSSIER/imports.md").exists());
}

#[test]
fn review_document_as_json_gives_the_state_and_the_rules() {
    let project = Project::new();
    project.write(DOC, "invented report");
    let out = project
        .run("anna-claude", &["--json", "review-document", DOC])
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&out).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.review-document.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["document"], DOC);
    assert_eq!(value["data"]["state"], "reviewing");
    assert_eq!(value["data"]["rules"].as_array().map(Vec::len), Some(7));
}
