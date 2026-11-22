//! Runs the built program for `confidential add` and `confidential list`: anyone in the project
//! adds a term, an LLM too (D32); the terms are listed with a count; a term already listed is
//! refused in any spelling of its separators; a removed term is added again (R-CONF-1, format.md
//! §24). Each test runs in its own project folder, with the clock fixed by `common::Project`
//! (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The first lines of the terms file of project P-001.
const HEADING: &str = "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n";

/// The terms file of the project, as text.
fn terms_text(project: &Project) -> String {
    project.read("DOSSIER/confidential.md")
}

/// The output of `confidential list --json`, parsed.
fn list_json(project: &Project, agent: &str) -> serde_json::Value {
    let output = project.stdout(agent, &["confidential", "list", "--json"]);
    serde_json::from_str(output.trim()).expect("one line of JSON")
}

#[test]
fn an_llm_adds_a_term_and_the_file_is_as_format_section_24_shows() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success()
        .stdout(contains("NT-450: added"));
    let expected =
        format!("{HEADING}## NT-450\n\n- history: {NOW} anna via anna-claude: added\n\n");
    assert_eq!(terms_text(&project), expected);

    let records = project.entry_journal();
    let last = records.last().expect("the change is in the journal");
    assert_eq!(last.id, "CONFIDENTIAL");
    assert_eq!(last.event, "added");
    assert_eq!(last.detail, "NT-450");
    assert_eq!(last.name, "anna-claude");
    assert_eq!(last.person, "anna");
    assert!(last.llm);
    assert_eq!(last.command, "confidential");
}

#[test]
fn a_person_and_an_llm_add_terms_and_the_list_shows_them_with_their_count() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    project
        .run("ben", &["confidential", "add", "Falcon"])
        .success()
        .stdout(contains("Falcon: added"));
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "NT-450\nFalcon\n2 terms\n"
    );
    let value = list_json(&project, "anna");
    assert_eq!(value["schema"], "dossier.confidential.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"]["terms"],
        serde_json::json!(["NT-450", "Falcon"])
    );
    assert_eq!(value["data"]["none_stated"], false);
    assert!(
        terms_text(&project).contains(&format!("- history: {NOW} ben: added")),
        "the history line names the person who added the term"
    );
}

#[test]
fn a_term_already_listed_is_refused_in_any_spelling_and_the_file_is_unchanged() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    let before = terms_text(&project);
    for spelling in ["nt 450", "NT_450", "  nt-450  ", "Nt-450"] {
        project
            .run("ben", &["confidential", "add", spelling])
            .failure()
            .code(1)
            .stderr(contains("NT-450 is already a confidential term"));
    }
    assert_eq!(terms_text(&project), before);
}

#[test]
fn a_removed_term_is_added_again_by_taking_its_removed_field_away() {
    let project = Project::new();
    project.write(
        "DOSSIER/confidential.md",
        &format!(
            "{HEADING}## Falcon\n\n- removed: 2026-11-04T09:00:00+01:00 by anna\n- history: 2026-11-03T09:00:00+01:00 anna: added\n- history: 2026-11-04T09:00:00+01:00 anna: removed\n\n"
        ),
    );
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "0 terms\n"
    );

    project
        .run("ben", &["confidential", "add", "falcon"])
        .success()
        .stdout(contains("Falcon: added"));
    let expected = format!(
        "{HEADING}## Falcon\n\n- history: 2026-11-03T09:00:00+01:00 anna: added\n- history: 2026-11-04T09:00:00+01:00 anna: removed\n- history: {NOW} ben: added\n\n"
    );
    assert_eq!(terms_text(&project), expected);
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "Falcon\n1 term\n"
    );
}

#[test]
fn a_blank_term_a_term_with_a_line_break_and_the_statement_of_none_are_refused() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "  "])
        .failure()
        .code(1)
        .stderr(contains("the term is empty"));
    project
        .run("anna-claude", &["confidential", "add", "NT\n450"])
        .failure()
        .code(1)
        .stderr(contains("one line"));
    project
        .run(
            "anna-claude",
            &["confidential", "add", "(no confidential terms)"],
        )
        .failure()
        .code(1)
        .stderr(contains("not a term"));
    assert!(
        !project.root().join("DOSSIER/confidential.md").exists(),
        "a refused term writes no file"
    );
}

#[test]
fn adding_a_term_needs_a_name() {
    let project = Project::new();
    project
        .run("", &["confidential", "add", "NT-450"])
        .failure()
        .code(2)
        .stderr(contains("needs a name"));
}

#[test]
fn a_project_with_no_terms_lists_none_and_a_statement_of_none_is_shown_until_a_term_is_added() {
    let project = Project::new();
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "0 terms\n"
    );
    let value = list_json(&project, "anna");
    assert_eq!(value["data"]["terms"], serde_json::json!([]));
    assert_eq!(value["data"]["none_stated"], false);

    project.write(
        "DOSSIER/confidential.md",
        &format!(
            "{HEADING}## (no confidential terms)\n\n- stated: 2026-11-02T09:00:00+01:00 by anna\n\n"
        ),
    );
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "no confidential terms stated by anna at 2026-11-02T09:00:00+01:00\n"
    );
    let value = list_json(&project, "anna");
    assert_eq!(value["data"]["none_stated"], true);

    project
        .run("ben", &["confidential", "add", "NT-450"])
        .success();
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "NT-450\n1 term\n"
    );
}
