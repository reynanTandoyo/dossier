//! Runs the built program for `value`: key values are set, shown, proposed, approved and rejected,
//! with who and when in their history (R-VALUE-1 to R-VALUE-3). Each test runs in its own project
//! folder, with the clock fixed by `common::Project` (2026-11-20T10:00:00+01:00).

mod common;

use common::Project;
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The first lines of a values file for project P-001, with one block to be filled in.
const HEADING: &str = "<!-- dossier-format: 1 -->\n# Values of P-001\n\n";

/// The values file of the project, as text.
fn values_text(project: &Project) -> String {
    project.read("DOSSIER/values.md")
}

#[test]
fn a_person_sets_a_value_and_the_file_is_as_format_section_17_shows() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.4 kg/s"])
        .success()
        .stdout(contains("design.massflow: set — 12.4 kg/s"));
    let expected = format!(
        "{HEADING}## design.massflow\n\n- value: 12.4\n- unit: kg/s\n- valid-from: {NOW}\n- history: {NOW} anna: set — 12.4 kg/s\n\n"
    );
    assert_eq!(values_text(&project), expected);

    let records = project.entry_journal();
    let last = records.last().expect("the change is in the journal");
    assert_eq!(last.id, "VALUES");
    assert_eq!(last.event, "set");
    assert_eq!(last.detail, "design.massflow: 12.4 kg/s");
    assert_eq!(last.name, "anna");
    assert_eq!(last.command, "value");

    // The same value again changes nothing.
    project
        .run("anna", &["value", "set", "design.massflow", "12.4 kg/s"])
        .success()
        .stdout(contains("design.massflow: no change"));
    assert_eq!(values_text(&project), expected);
}

#[test]
fn a_value_without_a_unit_or_with_a_bad_name_is_refused_and_no_file_is_written() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.4"])
        .failure()
        .code(1)
        .stderr(contains("has no unit"));
    project
        .run("anna", &["value", "set", "Design.Massflow", "12.4 kg/s"])
        .failure()
        .code(1)
        .stderr(contains("a value name has lower-case letters"));
    assert!(!project.root().join("DOSSIER/values.md").exists());
}

#[test]
fn a_dimensionless_value_has_no_unit_and_says_so() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "value",
                "set",
                "design.gas",
                "air, ideal gas",
                "--dimensionless",
            ],
        )
        .success()
        .stdout(contains("design.gas: set — air, ideal gas"));
    let expected = format!(
        "{HEADING}## design.gas\n\n- value: air, ideal gas\n- dimensionless: yes\n- valid-from: {NOW}\n- history: {NOW} anna: set — air, ideal gas\n\n"
    );
    assert_eq!(values_text(&project), expected);
}

#[test]
fn an_llm_value_is_proposed_and_the_value_in_force_stays() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.1 kg/s"])
        .success();
    project
        .run(
            "anna-claude",
            &["value", "set", "design.massflow", "12.9 kg/s"],
        )
        .success()
        .stdout(contains(
            "design.massflow: proposed — 12.9 kg/s (waiting for a person)",
        ));
    let expected = format!(
        "{HEADING}## design.massflow\n\n- value: 12.1\n- unit: kg/s\n- valid-from: {NOW}\n- proposed: 12.9 kg/s by anna-claude at {NOW}\n- history: {NOW} anna: set — 12.1 kg/s\n- history: {NOW} anna via anna-claude: proposed — 12.9 kg/s\n\n"
    );
    assert_eq!(values_text(&project), expected);
}

#[test]
fn a_person_approves_the_proposal_and_it_comes_into_force() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.1 kg/s"])
        .success();
    project
        .run(
            "anna-claude",
            &["value", "set", "design.massflow", "12.9 kg/s"],
        )
        .success();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["value", "approve", "design.massflow"])
        .assert()
        .success()
        .stdout(contains("design.massflow: approved — 12.9 kg/s"));
    let expected = format!(
        "{HEADING}## design.massflow\n\n- value: 12.9\n- unit: kg/s\n- valid-from: {NOW}\n- history: {NOW} anna: set — 12.1 kg/s\n- history: {NOW} anna via anna-claude: proposed — 12.9 kg/s\n- history: {NOW} anna: set — 12.9 kg/s\n\n"
    );
    assert_eq!(values_text(&project), expected);
}

#[test]
fn a_person_rejects_the_proposal_and_its_history_keeps_it() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.1 kg/s"])
        .success();
    project
        .run(
            "anna-claude",
            &["value", "set", "design.massflow", "12.9 kg/s"],
        )
        .success();
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["value", "reject", "design.massflow"])
        .assert()
        .success()
        .stdout(contains("design.massflow: rejected — 12.9 kg/s"));
    let expected = format!(
        "{HEADING}## design.massflow\n\n- value: 12.1\n- unit: kg/s\n- valid-from: {NOW}\n- history: {NOW} anna: set — 12.1 kg/s\n- history: {NOW} anna via anna-claude: proposed — 12.9 kg/s\n- history: {NOW} anna: rejected — 12.9 kg/s\n\n"
    );
    assert_eq!(values_text(&project), expected);
}

#[test]
fn approving_and_rejecting_need_a_person_at_a_terminal_and_a_proposal() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.1 kg/s"])
        .success();
    project
        .run(
            "anna-claude",
            &["value", "set", "design.massflow", "12.9 kg/s"],
        )
        .success();
    project
        .command("anna-claude", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["value", "approve", "design.massflow"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains(
            "value approve is for a person; anna-claude is an LLM",
        ));
    project
        .run("anna", &["value", "approve", "design.massflow"])
        .failure()
        .code(1)
        .stderr(contains(
            "this command must be run by a person at a terminal",
        ));
    project
        .command("anna", NOW)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["value", "approve", "design.speed"])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("design.speed has no proposed value"));
}

#[test]
fn said_by_a_person_brings_an_llm_value_into_force_and_an_unknown_person_is_refused() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &[
                "value",
                "set",
                "design.massflow",
                "13.0 kg/s",
                "--said-by",
                "anna",
            ],
        )
        .success()
        .stdout(contains("design.massflow: set — 13.0 kg/s"));
    let text = values_text(&project);
    assert!(text.contains("- value: 13.0\n"), "{text}");
    assert!(!text.contains("proposed"), "{text}");
    assert!(
        text.contains(&format!(
            "- history: {NOW} anna via anna-claude: set — 13.0 kg/s"
        )),
        "{text}"
    );

    project
        .run(
            "anna-claude",
            &[
                "value",
                "set",
                "design.massflow",
                "14.0 kg/s",
                "--said-by",
                "carl",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("carl is not one"));
}

#[test]
fn an_attached_source_brings_an_llm_value_into_force_and_a_detached_one_does_not() {
    let project = Project::new();
    project.write(
        "DOSSIER/files/T-001.md",
        "<!-- dossier-format: 1 -->\n# Files of T-001\n\n## data/spec.pdf\n\n- role: reference\n\n## data/old.csv\n\n- role: reference\n- detached: 2026-11-20T09:00:00+01:00 by anna\n",
    );
    project
        .run(
            "anna-claude",
            &[
                "value",
                "set",
                "design.massflow",
                "12.6 kg/s",
                "--source",
                "data/spec.pdf",
            ],
        )
        .success()
        .stdout(contains("design.massflow: set — 12.6 kg/s"));
    let text = values_text(&project);
    assert!(text.contains("- source: data/spec.pdf\n"), "{text}");

    project
        .run(
            "anna-claude",
            &[
                "value",
                "set",
                "design.speed",
                "3900 rpm",
                "--source",
                "data/old.csv",
            ],
        )
        .success()
        .stdout(contains("design.speed: proposed — 3900 rpm"));
    assert!(
        values_text(&project).contains(&format!("- proposed: 3900 rpm by anna-claude at {NOW}\n"))
    );
}

#[test]
fn show_gives_the_fields_and_the_history_without_a_name_and_a_missing_value_is_not_found() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.4 kg/s"])
        .success();
    project
        .run("", &["value", "show", "design.massflow"])
        .success()
        .stdout(contains("  value: 12.4"))
        .stdout(contains(format!("  history: {NOW} anna: set — 12.4 kg/s")));
    project
        .run("", &["value", "show", "design.none"])
        .failure()
        .code(1)
        .stderr(contains("no value named design.none"));
}

#[test]
fn json_output_names_the_command_and_the_status() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["--json", "value", "set", "design.massflow", "12.4 kg/s"],
        )
        .success()
        .stdout(contains("\"schema\":\"dossier.value.v1\""))
        .stdout(contains("\"status\":\"set\""));
}

#[test]
fn a_changed_value_names_the_usage_records_and_the_entries_that_use_or_mention_it() {
    let project = Project::new();
    project
        .run("anna", &["value", "set", "design.massflow", "12.4 kg/s"])
        .success();
    project
        .run(
            "anna",
            &[
                "decide",
                "The inlet mass flow is design.massflow.",
                "--reason",
                "An invented reason for the test.",
            ],
        )
        .success();
    project.write(
        "DOSSIER/usage/USE-001-speedline-run.md",
        "<!-- dossier-format: 1 -->\n# USE-001 · Speedline run\n\n- kind: usage\n- status: approved\n- created: 2026-11-20T10:00:00+01:00\n- author: anna\n- via: anna-claude\n- approved: 2026-11-20T10:00:00+01:00 by anna\n- valid-from: 2026-11-20T10:00:00+01:00\n- task: T-001\n- software: ExampleSolver\n- version: 2406\n- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created\n- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: approved\n\n## Parameters\n\nmassflow = {design.massflow}\n",
    );
    let output = project
        .run("anna", &["value", "set", "design.massflow", "13.0 kg/s"])
        .success()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).expect("the standard error is UTF-8");
    assert!(stderr.contains("warning: used by USE-001"), "{stderr}");
    assert!(stderr.contains("warning: named in DEC-001"), "{stderr}");
    assert!(!stderr.contains("named in USE-001"), "{stderr}");
}
