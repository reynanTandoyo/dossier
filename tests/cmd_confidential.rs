//! Runs the built program for `confidential`: anyone in the project adds a term, an LLM too (D32);
//! the terms are listed with a count; a term already listed is refused in any spelling of its
//! separators; a removed term is added again (R-CONF-1, format.md §24). A person at a terminal
//! removes a term or states that there are none (D32, R-CONF-5). `scan` finds the terms in the team
//! dossier, also those added after the team write (R-CONF-6). `--team` is refused, since the terms
//! belong to the project. Each test runs in its own project folder, with the clock fixed by
//! `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};

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

/// The message of the refusal of `--team` (confidential.rs).
const TEAM_REFUSED: &str = "confidential terms belong to a project; run without --team";

/// Runs `confidential` as `agent` at a terminal: DOSSIER_ANSWERS stands in for the terminal (D32).
/// None of these subcommands asks a question, so the value of the variable does not matter.
fn at_terminal(project: &Project, agent: &str, args: &[&str]) -> assert_cmd::assert::Assert {
    project
        .command(agent, NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(args)
        .assert()
}

#[test]
fn a_person_removes_a_term_by_any_spelling_and_it_is_no_longer_listed() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    project
        .run("ben", &["confidential", "add", "Falcon"])
        .success();
    at_terminal(&project, "anna", &["confidential", "remove", "nt450"])
        .success()
        .stdout(contains("NT-450: removed"));
    let expected = format!(
        "{HEADING}## NT-450\n\n- removed: {NOW} by anna\n- history: {NOW} anna via anna-claude: added\n- history: {NOW} anna: removed\n\n## Falcon\n\n- history: {NOW} ben: added\n\n"
    );
    assert_eq!(terms_text(&project), expected);
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "Falcon\n1 term\n"
    );

    let records = project.entry_journal();
    let last = records.last().expect("the removal is in the journal");
    assert_eq!(last.id, "CONFIDENTIAL");
    assert_eq!(last.event, "removed");
    assert_eq!(last.detail, "NT-450");
    assert_eq!(last.name, "anna");
    assert_eq!(last.person, "anna");
    assert!(!last.llm);
    assert_eq!(last.command, "confidential");
}

#[test]
fn a_term_already_removed_or_never_listed_is_not_found_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    at_terminal(&project, "anna", &["confidential", "remove", "NT-450"]).success();
    let before = terms_text(&project);
    at_terminal(&project, "anna", &["confidential", "remove", "nt-450"])
        .failure()
        .code(1)
        .stderr(contains("NT-450 is already removed"));
    at_terminal(&project, "anna", &["confidential", "remove", "Falcon"])
        .failure()
        .code(1)
        .stderr(contains("Falcon is not a confidential term"));
    assert_eq!(terms_text(&project), before);

    let fresh = Project::new();
    at_terminal(&fresh, "anna", &["confidential", "remove", "NT-450"])
        .failure()
        .code(1)
        .stderr(contains("NT-450 is not a confidential term"));
    assert!(
        !fresh.root().join("DOSSIER/confidential.md").exists(),
        "a refused removal writes no file"
    );
}

#[test]
fn removing_a_term_needs_a_person_at_a_terminal_and_a_name() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    let before = terms_text(&project);
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["confidential", "remove", "NT-450"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains(
            "confidential remove is for a person; anna-claude is an LLM",
        ));
    project
        .run("anna", &["confidential", "remove", "NT-450"])
        .failure()
        .code(1)
        .stderr(contains(
            "this command must be run by a person at a terminal",
        ));
    project
        .run("", &["confidential", "remove", "NT-450"])
        .failure()
        .code(2)
        .stderr(contains("needs a name"));
    assert_eq!(terms_text(&project), before);
}

#[test]
fn none_needs_yes_a_person_and_a_terminal_and_writes_nothing_without_them() {
    let project = Project::new();
    at_terminal(&project, "anna", &["confidential", "none"])
        .failure()
        .code(2)
        .stderr(contains("add --yes"));
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "")
        .args(["confidential", "none", "--yes"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains(
            "confidential none is for a person; anna-claude is an LLM",
        ));
    project
        .run("anna", &["confidential", "none", "--yes"])
        .failure()
        .code(1)
        .stderr(contains(
            "this command must be run by a person at a terminal",
        ));
    assert!(
        !project.root().join("DOSSIER/confidential.md").exists(),
        "nothing is written"
    );
}

#[test]
fn none_with_yes_on_an_empty_list_states_there_are_none_and_says_who_and_when() {
    let project = Project::new();
    at_terminal(&project, "anna", &["confidential", "none", "--yes"])
        .success()
        .stdout(contains("(no confidential terms): stated"));
    let expected = format!(
        "{HEADING}## (no confidential terms)\n\n- stated: {NOW} by anna\n- history: {NOW} anna: stated\n\n"
    );
    assert_eq!(terms_text(&project), expected);
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        format!("no confidential terms stated by anna at {NOW}\n")
    );
    let value = list_json(&project, "anna");
    assert_eq!(value["data"]["none_stated"], true);

    let records = project.entry_journal();
    let last = records.last().expect("the statement is in the journal");
    assert_eq!(last.id, "CONFIDENTIAL");
    assert_eq!(last.event, "stated");
    assert_eq!(last.detail, "(no confidential terms)");
}

#[test]
fn none_is_refused_while_a_term_is_listed_and_allowed_once_the_term_is_removed() {
    let project = Project::new();
    project
        .run("ben", &["confidential", "add", "NT-450"])
        .success();
    let before = terms_text(&project);
    at_terminal(&project, "anna", &["confidential", "none", "--yes"])
        .failure()
        .code(1)
        .stderr(contains("remove the terms first"));
    assert_eq!(terms_text(&project), before);

    at_terminal(&project, "anna", &["confidential", "remove", "NT-450"]).success();
    at_terminal(&project, "anna", &["confidential", "none", "--yes"]).success();
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        format!("no confidential terms stated by anna at {NOW}\n")
    );
}

#[test]
fn a_term_differing_only_in_its_separators_is_a_duplicate_as_conf_find_compares_them() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT-450"])
        .success();
    let before = terms_text(&project);
    project
        .run("ben", &["confidential", "add", "NT450"])
        .failure()
        .code(1)
        .stderr(contains("NT-450 is already a confidential term"));
    project
        .run("ben", &["confidential", "add", "n_t 4 5-0"])
        .failure()
        .code(1)
        .stderr(contains("NT-450 is already a confidential term"));
    assert_eq!(terms_text(&project), before);

    project
        .run("anna-claude", &["confidential", "add", "Project Falcon"])
        .success();
    project
        .run("ben", &["confidential", "add", "ProjectFalcon"])
        .failure()
        .code(1)
        .stderr(contains("Project Falcon is already a confidential term"));

    project
        .run("ben", &["confidential", "add", "NT-45"])
        .success()
        .stdout(contains("NT-45: added"));
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "NT-450\nProject Falcon\nNT-45\n3 terms\n"
    );
}

#[test]
fn a_term_written_without_separators_is_the_same_term_as_one_with_them() {
    let project = Project::new();
    project
        .run("anna-claude", &["confidential", "add", "NT450"])
        .success()
        .stdout(contains("NT450: added"));
    project
        .run("ben", &["confidential", "add", "NT-450"])
        .failure()
        .code(1)
        .stderr(contains("NT450 is already a confidential term"));

    at_terminal(&project, "anna", &["confidential", "remove", "NT-450"]).success();
    project
        .run("ben", &["confidential", "add", "NT 450"])
        .success()
        .stdout(contains("NT450: added"));
    let expected = format!(
        "{HEADING}## NT450\n\n- history: {NOW} anna via anna-claude: added\n- history: {NOW} anna: removed\n- history: {NOW} ben: added\n\n"
    );
    assert_eq!(terms_text(&project), expected);
    assert_eq!(
        project.stdout("anna", &["confidential", "list"]),
        "NT450\n1 term\n"
    );
}

#[test]
fn every_subcommand_refuses_the_team_flag_and_writes_nothing() {
    let project = Project::new();
    for args in [
        vec!["confidential", "list", "--team"],
        vec!["confidential", "add", "NT-450", "--team"],
        vec!["confidential", "remove", "NT-450", "--team"],
        vec!["confidential", "none", "--yes", "--team"],
        vec!["confidential", "scan", "--team"],
    ] {
        at_terminal(&project, "anna", &args)
            .failure()
            .code(2)
            .stderr(contains(TEAM_REFUSED));
    }
    assert!(
        !project.root().join("DOSSIER/confidential.md").exists(),
        "a refused command writes no file"
    );
}

#[test]
fn with_a_linked_team_the_flag_is_still_refused_and_the_team_dossier_gets_no_terms() {
    let project = Project::new();
    let team_dir = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(team_dir.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let team = team_dir.path().join("DOSSIER");
    dossier::config::set_list(&team, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    project
        .run("anna", &["team", "use", &team.display().to_string()])
        .success();

    project
        .run("anna-claude", &["confidential", "add", "NT-450", "--team"])
        .failure()
        .code(2)
        .stderr(contains(TEAM_REFUSED));
    assert!(
        !team.join("confidential.md").exists(),
        "the team dossier gets no terms"
    );
    assert!(!project.root().join("DOSSIER/confidential.md").exists());
}

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder, whose name
/// has a space and an umlaut, with anna as its member, and the project linked to it by `team use`.
/// Returns the folder (it keeps the folder alive) and the DOSSIER folder in it.
fn linked_team(project: &Project) -> (tempfile::TempDir, PathBuf) {
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
    let team = folder.path().join("DOSSIER");
    dossier::config::set_list(&team, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    let path = team.display().to_string();
    project
        .run("anna", &["team", "use", path.as_str()])
        .success();
    (folder, team)
}

/// The 1-based number of the first line of `text` that contains `needle`.
fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.contains(needle))
        .map(|index| index + 1)
        .expect("the text holds the needle")
}

/// The name of the one file in `folder` whose name starts with `prefix`.
fn file_named(folder: &Path, prefix: &str) -> String {
    let names: Vec<String> = fs::read_dir(folder)
        .expect("the folder can be read")
        .map(|entry| entry.expect("a folder entry").file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| name.starts_with(prefix))
        .collect();
    assert_eq!(names.len(), 1, "one file starts with {prefix}");
    names.into_iter().next().expect("one file")
}

#[test]
fn a_term_added_after_a_team_write_is_found_in_the_team_lesson_and_glossary() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    // No term is listed yet and a person states there are none, so the team writes pass (R-CONF-5).
    at_terminal(&project, "anna", &["confidential", "none", "--yes"]).success();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs converge early near stall at stage nt 999",
                "--title",
                "Steady runs converge early",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Comparison of speedlines, report section 4.2",
                "--confidence",
                "measured",
                "--team",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "pressure ratio",
                "outlet pressure divided by inlet pressure, see NT999",
                "--team",
            ],
        )
        .success();
    // The term is added later, so the team already holds both texts.
    project
        .run("anna", &["confidential", "add", "NT-999"])
        .success();

    let lesson = file_named(&team.join("lessons"), "LES-");
    let lesson_text =
        fs::read_to_string(team.join("lessons").join(&lesson)).expect("the team lesson");
    let glossary_text = fs::read_to_string(team.join("glossary.md")).expect("the team glossary");
    let lesson_hit = format!(
        "DOSSIER/lessons/{lesson}:{}: NT-999",
        line_of(&lesson_text, "nt 999")
    );
    let glossary_hit = format!(
        "DOSSIER/glossary.md:{}: NT-999",
        line_of(&glossary_text, "NT999")
    );
    project
        .run("anna", &["confidential", "scan"])
        .failure()
        .code(1)
        .stderr(contains(format!(
            "error: {lesson_hit}\n{glossary_hit}\n2 matches"
        )));
}

#[test]
fn a_team_with_no_term_in_it_gives_zero_matches() {
    let project = Project::new();
    let (_folder, _team) = linked_team(&project);
    project
        .run("anna", &["confidential", "add", "NT-999"])
        .success();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs converge early near stall",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Comparison of speedlines, report section 4.2",
                "--confidence",
                "measured",
                "--team",
            ],
        )
        .success();
    assert_eq!(
        project.stdout("anna", &["confidential", "scan"]),
        "0 matches\n"
    );
}

#[test]
fn a_scan_needs_terms_or_a_statement_of_none_before_it_reports_zero_matches() {
    let project = Project::new();
    let (_folder, _team) = linked_team(&project);
    project
        .run("anna", &["confidential", "scan"])
        .failure()
        .code(1)
        .stderr(contains("this project has no confidential terms yet"));

    at_terminal(&project, "anna", &["confidential", "none", "--yes"]).success();
    assert_eq!(
        project.stdout("anna", &["confidential", "scan"]),
        "0 matches\n"
    );
}

#[test]
fn a_scan_without_a_team_dossier_says_the_project_has_none() {
    let project = Project::new();
    project
        .run("anna", &["confidential", "add", "NT-999"])
        .success();
    project
        .run("anna", &["confidential", "scan"])
        .failure()
        .code(4)
        .stderr(contains("this project has no team dossier"));
}
