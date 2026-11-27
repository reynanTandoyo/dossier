//! Runs the built program for `link`: each of two entries gets the other's ID in its `related`
//! field, once, with one history line; a second link changes nothing. With `--as refines` or
//! `--as contradicts` the IDs go to the fields of that type instead (D61). Each test runs in its own
//! temporary project folder with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use dossier::entry::Entry;
use predicates::str::contains;

/// The fixed clock of `common::Project`, as it appears in history lines.
const WHEN: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for the entries the tests link.
const DECISION: &str = "Use the transient runs for the last three points.";
const REASON: &str = "An invented reason: steady runs converged where the transient run stalled.";
const OBSERVATION: &str = "The mass flow still oscillated by 0.4 % with half the time step.";
const EVIDENCE: &str = "Runs 12 and 13, monitor plots in results.";

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file can be read")
}

/// The value of the field `name` of the entry with `id`, None when it has none.
fn field_of(project: &Project, id: &str, name: &str) -> Option<String> {
    Entry::parse(&text_of(project, id))
        .expect("the file is an entry")
        .field(name)
}

/// The value of the related field of the entry with `id`, None when it has none.
fn related_of(project: &Project, id: &str) -> Option<String> {
    field_of(project, id, "related")
}

/// A decision DEC-001 and a finding FND-001, both written by anna and approved at once.
fn decision_and_finding(project: &Project) {
    project.stdout("anna", &["decide", DECISION, "--reason", REASON]);
    project.stdout("anna", &["found", OBSERVATION, "--evidence", EVIDENCE]);
}

/// The number of lines in the journal, one per change.
fn journal_lines(project: &Project) -> usize {
    project.read("DOSSIER/.state/journal.jsonl").lines().count()
}

#[test]
fn link_adds_each_entry_to_the_others_related_field_and_nothing_else() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision_before = text_of(&project, "DEC-001");
    let finding_before = text_of(&project, "FND-001");

    project
        .run("anna", &["link", "DEC-001", "FND-001"])
        .success()
        .stdout("DEC-001 and FND-001: linked\n");

    // Taking away the new history line and the new field line gives back the old text.
    let decision_after = text_of(&project, "DEC-001");
    let history = format!("- history: {WHEN} anna: changed — related FND-001\n");
    assert_eq!(decision_after.matches(&history).count(), 1);
    assert_eq!(
        decision_after
            .replacen(&history, "", 1)
            .replacen("- related: FND-001\n", "", 1),
        decision_before
    );
    assert_eq!(related_of(&project, "DEC-001").as_deref(), Some("FND-001"));

    let finding_after = text_of(&project, "FND-001");
    let history = format!("- history: {WHEN} anna: changed — related DEC-001\n");
    assert_eq!(finding_after.matches(&history).count(), 1);
    assert_eq!(
        finding_after
            .replacen(&history, "", 1)
            .replacen("- related: DEC-001\n", "", 1),
        finding_before
    );
    assert_eq!(related_of(&project, "FND-001").as_deref(), Some("DEC-001"));

    // The text before the change is kept as the snapshot of each entry.
    assert_eq!(
        project.read("DOSSIER/.state/history/DEC-001/000001.md"),
        decision_before
    );
}

#[test]
fn a_second_link_changes_nothing_in_either_order() {
    let project = Project::new();
    decision_and_finding(&project);
    project.stdout("anna", &["link", "DEC-001", "FND-001"]);
    let decision = text_of(&project, "DEC-001");
    let finding = text_of(&project, "FND-001");
    let journal = journal_lines(&project);

    project
        .run("anna", &["link", "DEC-001", "FND-001"])
        .success()
        .stdout("already linked\n");
    project
        .run("anna", &["link", "FND-001", "DEC-001"])
        .success()
        .stdout("already linked\n");

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(text_of(&project, "FND-001"), finding);
    assert_eq!(journal_lines(&project), journal, "no journal line is added");
    assert!(
        !project
            .root()
            .join("DOSSIER/.state/history/DEC-001/000002.md")
            .exists(),
        "no snapshot is taken for a link that changes nothing"
    );
}

#[test]
fn linking_an_entry_to_itself_fails_and_writes_nothing() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision = text_of(&project, "DEC-001");

    for pair in [
        ["DEC-001", "DEC-001"],
        ["dec-001", "DEC-001"],
        ["P-001:DEC-001", "DEC-001"],
    ] {
        project
            .run("anna", &["link", pair[0], pair[1]])
            .failure()
            .code(1)
            .stderr(contains("an entry cannot be linked to itself"));
    }

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(related_of(&project, "DEC-001"), None);
}

#[test]
fn existing_related_entries_are_kept_and_the_new_one_follows_them() {
    let project = Project::new();
    decision_and_finding(&project);
    project.stdout(
        "anna",
        &[
            "found",
            "A second observation, invented.",
            "--evidence",
            EVIDENCE,
        ],
    );
    project.stdout("anna", &["link", "DEC-001", "FND-001"]);
    let finding_first = text_of(&project, "FND-001");

    project
        .run("anna", &["link", "DEC-001", "FND-002"])
        .success()
        .stdout("DEC-001 and FND-002: linked\n");

    assert_eq!(
        related_of(&project, "DEC-001").as_deref(),
        Some("FND-001, FND-002")
    );
    assert_eq!(related_of(&project, "FND-002").as_deref(), Some("DEC-001"));
    assert_eq!(
        text_of(&project, "FND-001"),
        finding_first,
        "an entry that is not linked by this command keeps its bytes"
    );
}

#[test]
fn only_the_entry_that_does_not_name_the_other_changes() {
    let project = Project::new();
    decision_and_finding(&project);
    // The decision names the finding by hand; the finding does not name the decision yet.
    project.stdout("anna", &["set", "DEC-001", "--field", "related=FND-001"]);
    let decision = text_of(&project, "DEC-001");

    project
        .run("anna", &["link", "DEC-001", "FND-001"])
        .success()
        .stdout("DEC-001 and FND-001: linked\n");

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(related_of(&project, "FND-001").as_deref(), Some("DEC-001"));
}

#[test]
fn an_llm_links_and_the_history_names_the_person_and_the_llm() {
    let project = Project::new();
    decision_and_finding(&project);

    project
        .run("anna-claude", &["link", "DEC-001", "FND-001"])
        .success()
        .stdout("DEC-001 and FND-001: linked\n");

    let history = format!("- history: {WHEN} anna via anna-claude: changed — related FND-001\n");
    assert_eq!(text_of(&project, "DEC-001").matches(&history).count(), 1);
}

#[test]
fn a_missing_entry_is_refused_and_nothing_changes() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision = text_of(&project, "DEC-001");
    let finding = text_of(&project, "FND-001");

    project
        .run("anna", &["link", "DEC-001", "DEC-009"])
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(text_of(&project, "FND-001"), finding);
}

#[test]
fn a_dry_run_prints_what_would_be_written_and_changes_nothing() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision = text_of(&project, "DEC-001");
    let finding = text_of(&project, "FND-001");

    project
        .run("anna", &["link", "DEC-001", "FND-001", "--dry-run"])
        .success()
        .stdout("DEC-001 and FND-001: linked\n")
        .stderr(contains("dry run: would write"));

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(text_of(&project, "FND-001"), finding);
}

#[test]
fn link_without_a_name_needs_one_and_writes_nothing() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision = text_of(&project, "DEC-001");

    project
        .run("", &["link", "DEC-001", "FND-001"])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));

    assert_eq!(text_of(&project, "DEC-001"), decision);
}

#[test]
fn refines_gives_the_narrowing_entry_refines_and_the_other_refined_by() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision_before = text_of(&project, "DEC-001");
    let finding_before = text_of(&project, "FND-001");

    project
        .run("anna", &["link", "DEC-001", "FND-001", "--as", "refines"])
        .success()
        .stdout("DEC-001 refines FND-001\n");

    // Taking away the new history line and the new field line gives back the old text.
    let decision_after = text_of(&project, "DEC-001");
    let history = format!("- history: {WHEN} anna: changed — refines FND-001\n");
    assert_eq!(decision_after.matches(&history).count(), 1);
    assert_eq!(
        decision_after
            .replacen(&history, "", 1)
            .replacen("- refines: FND-001\n", "", 1),
        decision_before
    );
    assert_eq!(
        field_of(&project, "DEC-001", "refines").as_deref(),
        Some("FND-001")
    );

    let finding_after = text_of(&project, "FND-001");
    let history = format!("- history: {WHEN} anna: changed — refined by DEC-001\n");
    assert_eq!(finding_after.matches(&history).count(), 1);
    assert_eq!(
        finding_after
            .replacen(&history, "", 1)
            .replacen("- refined-by: DEC-001\n", "", 1),
        finding_before
    );
    assert_eq!(
        field_of(&project, "FND-001", "refined-by").as_deref(),
        Some("DEC-001")
    );
}

#[test]
fn contradicts_names_the_other_entry_in_both_files() {
    let project = Project::new();
    decision_and_finding(&project);

    project
        .run(
            "anna",
            &["link", "DEC-001", "FND-001", "--as", "contradicts"],
        )
        .success()
        .stdout("DEC-001 contradicts FND-001\n");

    assert_eq!(
        field_of(&project, "DEC-001", "contradicts").as_deref(),
        Some("FND-001")
    );
    assert_eq!(
        field_of(&project, "FND-001", "contradicts").as_deref(),
        Some("DEC-001")
    );
    let to_finding = format!("- history: {WHEN} anna: changed — contradicts FND-001\n");
    let to_decision = format!("- history: {WHEN} anna: changed — contradicts DEC-001\n");
    assert_eq!(text_of(&project, "DEC-001").matches(&to_finding).count(), 1);
    assert_eq!(
        text_of(&project, "FND-001").matches(&to_decision).count(),
        1
    );
}

#[test]
fn a_typed_link_that_is_already_there_prints_already_linked_and_changes_nothing() {
    let project = Project::new();
    decision_and_finding(&project);
    project.stdout("anna", &["link", "DEC-001", "FND-001", "--as", "refines"]);
    let decision = text_of(&project, "DEC-001");
    let finding = text_of(&project, "FND-001");
    let journal = journal_lines(&project);

    project
        .run("anna", &["link", "DEC-001", "FND-001", "--as", "refines"])
        .success()
        .stdout("already linked\n");

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(text_of(&project, "FND-001"), finding);
    assert_eq!(journal_lines(&project), journal, "no journal line is added");
}

#[test]
fn an_unknown_type_is_refused_and_nothing_is_written() {
    let project = Project::new();
    decision_and_finding(&project);
    let decision = text_of(&project, "DEC-001");
    let finding = text_of(&project, "FND-001");
    let journal = journal_lines(&project);

    project
        .run(
            "anna",
            &["link", "DEC-001", "FND-001", "--as", "supersedes"],
        )
        .failure()
        .code(1)
        .stderr(contains("--as takes refines or contradicts"));

    assert_eq!(text_of(&project, "DEC-001"), decision);
    assert_eq!(text_of(&project, "FND-001"), finding);
    assert_eq!(journal_lines(&project), journal);
}

#[test]
fn a_plain_link_after_a_contradiction_still_adds_the_related_field() {
    let project = Project::new();
    decision_and_finding(&project);
    project.stdout(
        "anna",
        &["link", "DEC-001", "FND-001", "--as", "contradicts"],
    );

    project
        .run("anna", &["link", "DEC-001", "FND-001"])
        .success()
        .stdout("DEC-001 and FND-001: linked\n");

    assert_eq!(related_of(&project, "DEC-001").as_deref(), Some("FND-001"));
    assert_eq!(related_of(&project, "FND-001").as_deref(), Some("DEC-001"));
    assert_eq!(
        field_of(&project, "DEC-001", "contradicts").as_deref(),
        Some("FND-001")
    );
}
