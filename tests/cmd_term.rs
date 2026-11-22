//! Runs the built program for `term`: adding a term to the glossary (the file, its history line and
//! its journal line), the refusals (a term or spelling already there, a blank meaning, no name, no
//! word), looking a term up by another spelling, by part of its name or by its meaning, a lookup
//! without a name, a team term written with `--team` after the confidential check and marked
//! `[team]` in the project, a team that cannot be reached, and the JSON data. Each test runs in its
//! own temporary project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The fixed clock of every run (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The project's glossary after `mass flow` was added as anna (format.md §20).
const MASS_FLOW_FILE: &str = "<!-- dossier-format: 1 -->\n# Glossary of P-001\n\n## mass flow\n\n- meaning: fluid through a section per second\n- symbol: m-dot\n- unit: kg/s\n- also: Massenstrom, mdot\n- history: 2026-11-20T10:00:00+01:00 anna: added\n\n";

/// The line that `term` prints for `mass flow`.
const MASS_FLOW_LINE: &str =
    "mass flow (m-dot, kg/s): fluid through a section per second  [also: Massenstrom, mdot]\n";

/// The line that `term` prints for `pressure ratio` in the project.
const PRESSURE_RATIO_LINE: &str =
    "pressure ratio: outlet pressure divided by inlet pressure  [also: Druckverhältnis]\n";

/// The same line for the team's copy of `pressure ratio`.
const TEAM_PRESSURE_RATIO_LINE: &str =
    "pressure ratio: outlet pressure divided by inlet pressure  [also: Druckverhältnis]  [team]\n";

/// Adds `mass flow` with a symbol, a unit and two other spellings, as anna.
fn add_mass_flow(project: &Project) {
    project.stdout(
        "anna",
        &[
            "term",
            "add",
            "mass flow",
            "fluid through a section per second",
            "--symbol",
            "m-dot",
            "--unit",
            "kg/s",
            "--also",
            "Massenstrom, mdot",
        ],
    );
}

/// Adds `pressure ratio` with one other spelling and no symbol or unit, as anna. `extra` is put
/// before the term's arguments, such as `--team`.
fn add_pressure_ratio(project: &Project, extra: &[&str]) -> assert_cmd::assert::Assert {
    let mut args = vec!["term", "add"];
    args.extend_from_slice(extra);
    args.extend_from_slice(&[
        "pressure ratio",
        "outlet pressure divided by inlet pressure",
        "--also",
        "Druckverhältnis",
    ]);
    project.run("anna", &args)
}

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder, whose
/// name has a space and an umlaut, run as anna. Anna is a member of the team. Returns the folder
/// (it keeps the folder alive) and the DOSSIER folder in it.
fn team_dossier(project: &Project) -> (tempfile::TempDir, PathBuf) {
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
    let dossier = folder.path().join("DOSSIER");
    dossier::config::set_list(&dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    (folder, dossier)
}

/// Links the project to the team dossier at `team`, as `team use` does.
fn link(project: &Project, team: &Path) {
    let path = team.display().to_string();
    project.stdout("anna", &["team", "use", path.as_str()]);
}

/// Writes the project's confidential terms file with the statement that there are none, so that a
/// team write from the project passes the check of R-CONF-5.
fn state_none(project: &Project) {
    project.write(
        "DOSSIER/confidential.md",
        "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## (no confidential terms)\n\n- stated: 2026-11-02T09:00:00+01:00 by anna\n",
    );
}

/// The JSON result of a run as anna with `--json`, parsed.
fn json_of(project: &Project, args: &[&str]) -> Value {
    let mut all = vec!["--json"];
    all.extend_from_slice(args);
    let output = project
        .command("anna", NOW)
        .args(&all)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).expect("the JSON result parses")
}

#[test]
fn a_term_is_added_with_its_fields_its_history_line_and_a_journal_line() {
    let project = Project::new();
    assert_eq!(
        project.stdout(
            "anna",
            &[
                "term",
                "add",
                "mass flow",
                "fluid through a section per second",
                "--symbol",
                "m-dot",
                "--unit",
                "kg/s",
                "--also",
                "Massenstrom, mdot",
            ],
        ),
        "mass flow: added\n"
    );
    assert_eq!(project.read("DOSSIER/glossary.md"), MASS_FLOW_FILE);

    let record = project
        .entry_journal()
        .pop()
        .expect("the term is in the journal");
    assert_eq!(record.id, "GLOSSARY");
    assert_eq!(record.event, "added");
    assert_eq!(record.detail, "mass flow");
    assert_eq!(record.person, "anna");
    assert_eq!(record.command, "term");
}

#[test]
fn a_second_term_is_added_after_the_first_and_keeps_its_bytes() {
    let project = Project::new();
    add_mass_flow(&project);
    add_pressure_ratio(&project, &[])
        .success()
        .stdout("pressure ratio: added\n");
    let text = project.read("DOSSIER/glossary.md");
    assert!(text.starts_with(MASS_FLOW_FILE), "the first block is kept");
    assert!(text.ends_with(
        "## pressure ratio\n\n- meaning: outlet pressure divided by inlet pressure\n- also: Druckverhältnis\n- history: 2026-11-20T10:00:00+01:00 anna: added\n\n"
    ));
}

#[test]
fn a_word_finds_a_term_by_another_spelling_by_part_of_its_name_or_by_its_meaning() {
    let project = Project::new();
    add_mass_flow(&project);
    add_pressure_ratio(&project, &[]).success();

    // The other spellings, in any case.
    assert_eq!(
        project.stdout("anna", &["term", "Massenstrom"]),
        MASS_FLOW_LINE
    );
    assert_eq!(project.stdout("anna", &["term", "MDOT"]), MASS_FLOW_LINE);
    assert_eq!(
        project.stdout("anna", &["term", "DRUCKVERHÄLTNIS"]),
        PRESSURE_RATIO_LINE
    );
    // Part of the term's name.
    assert_eq!(project.stdout("anna", &["term", "flow"]), MASS_FLOW_LINE);
    // A word of the meaning.
    assert_eq!(project.stdout("anna", &["term", "section"]), MASS_FLOW_LINE);
    // Two terms, sorted by term ignoring case.
    assert_eq!(
        project.stdout("anna", &["term", "ratio"]),
        PRESSURE_RATIO_LINE
    );
    assert_eq!(
        project.stdout("anna", &["term", "pressure"]),
        PRESSURE_RATIO_LINE
    );
    assert_eq!(
        project.stdout("anna", &["term", "i"]),
        format!("{MASS_FLOW_LINE}{PRESSURE_RATIO_LINE}")
    );
    // Nothing found is not an error.
    assert_eq!(
        project.stdout("anna", &["term", "enthalpy"]),
        "no term matches enthalpy\n"
    );
}

#[test]
fn a_term_or_a_spelling_already_in_the_glossary_is_refused_and_nothing_is_written() {
    let project = Project::new();
    add_mass_flow(&project);
    let before = project.read("DOSSIER/glossary.md");

    // The term itself, in another case.
    project
        .run("anna", &["term", "add", "Mass Flow", "another meaning"])
        .failure()
        .code(1)
        .stderr(contains("Mass Flow is already a term of the glossary"));
    // A term that is another term's other spelling.
    project
        .run("anna", &["term", "add", "Massenstrom", "a flow"])
        .failure()
        .code(1)
        .stderr(contains("Massenstrom is already a spelling of mass flow"));
    // A new term with an other spelling that is taken.
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "enthalpy",
                "heat content",
                "--also",
                "Enthalpie, mdot",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("mdot is already a spelling of mass flow"));

    assert_eq!(project.read("DOSSIER/glossary.md"), before);
    assert_eq!(
        project.stdout("anna", &["term", "enthalpy"]),
        "no term matches enthalpy\n"
    );
}

#[test]
fn a_term_needs_a_name_a_meaning_and_a_word_to_look_up() {
    let project = Project::new();
    // A write needs a name in the settings (Code::Usage, exit 2).
    project
        .command("", NOW)
        .args(["term", "add", "flow", "fluid"])
        .assert()
        .code(2)
        .stderr(contains("--agent"));
    // A blank meaning is refused (Code::BadValue).
    project
        .run("anna", &["term", "add", "flow", "   "])
        .failure()
        .code(1)
        .stderr(contains("the meaning is empty"));
    // Neither a word nor `add`, or a blank word, is a usage error.
    project.run("anna", &["term"]).failure().code(2);
    project.run("anna", &["term", "  "]).failure().code(2);

    assert!(!project.root().join("DOSSIER/glossary.md").exists());
}

#[test]
fn a_word_is_looked_up_without_a_name_and_with_a_name_that_is_not_in_the_settings() {
    let project = Project::new();
    add_mass_flow(&project);
    // A lookup reads the glossary, so the operating-system user or an unknown name is no reason to
    // refuse it.
    project
        .command("", NOW)
        .args(["term", "Massenstrom"])
        .assert()
        .success()
        .stdout(MASS_FLOW_LINE);
    project
        .run("carl", &["term", "mdot"])
        .success()
        .stdout(MASS_FLOW_LINE);
}

#[test]
fn a_team_term_is_added_with_team_and_found_by_the_project_marked_team() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    state_none(&project);
    add_mass_flow(&project);

    add_pressure_ratio(&project, &["--team"])
        .success()
        .stdout("pressure ratio: added\n");
    // The team's glossary has the block; the project's glossary keeps only its own term.
    assert_eq!(
        fs::read_to_string(team.join("glossary.md")).expect("the team glossary"),
        "<!-- dossier-format: 1 -->\n# Glossary of TEAM\n\n## pressure ratio\n\n- meaning: outlet pressure divided by inlet pressure\n- also: Druckverhältnis\n- history: 2026-11-20T10:00:00+01:00 anna: added\n\n"
    );
    assert_eq!(project.read("DOSSIER/glossary.md"), MASS_FLOW_FILE);

    // The project finds the team's term by its other spelling, marked team.
    assert_eq!(
        project.stdout("anna", &["term", "Druckverhältnis"]),
        TEAM_PRESSURE_RATIO_LINE
    );
    // With --team only the team's terms are searched.
    assert_eq!(
        project.stdout("anna", &["--team", "term", "mdot"]),
        "no term matches mdot\n"
    );
    assert_eq!(
        project.stdout("anna", &["--team", "term", "ratio"]),
        TEAM_PRESSURE_RATIO_LINE
    );
    // A team term that is already in the team glossary is refused.
    add_pressure_ratio(&project, &["--team"])
        .failure()
        .code(1)
        .stderr(contains("pressure ratio is already a term of the glossary"));
}

#[test]
fn a_team_write_from_a_project_is_checked_against_its_confidential_terms() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);

    // The project has no confidential terms and has not stated there are none: refused (R-CONF-5).
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "--team",
                "pressure ratio",
                "a ratio of pressures",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no confidential terms yet"));
    assert!(!team.join("glossary.md").exists());

    // A confidential term in the meaning is refused, with its line (R-CONF-2).
    project.write(
        "DOSSIER/confidential.md",
        "<!-- dossier-format: 1 -->\n# Confidential terms of P-001\n\n## NT-450\n\n- history: 2026-11-02T09:00:00+01:00 anna: added\n",
    );
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "--team",
                "pressure ratio",
                "the nt 450 stage ratio",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("confidential terms found: NT-450 (line 2)"));
    assert!(!team.join("glossary.md").exists());

    // A term that is clean is written.
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "--team",
                "pressure ratio",
                "a ratio of pressures",
            ],
        )
        .success()
        .stdout("pressure ratio: added\n");
    assert!(team.join("glossary.md").exists());
}

#[test]
fn a_project_that_stated_there_are_no_confidential_terms_may_write_to_the_team() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    state_none(&project);
    project
        .run(
            "anna",
            &[
                "term",
                "add",
                "--team",
                "pressure ratio",
                "a ratio of pressures",
            ],
        )
        .success()
        .stdout("pressure ratio: added\n");
}

#[test]
fn json_gives_the_term_it_added_and_the_terms_a_lookup_found() {
    let project = Project::new();
    let added = json_of(
        &project,
        &[
            "term",
            "add",
            "mass flow",
            "fluid through a section per second",
            "--unit",
            "kg/s",
            "--also",
            "Massenstrom, mdot",
        ],
    );
    assert_eq!(added["schema"], "dossier.term.v1");
    assert_eq!(added["ok"], true);
    assert_eq!(added["data"]["term"], "mass flow");
    assert_eq!(added["data"]["status"], "added");
    assert_eq!(added["data"]["team"], false);
    assert_eq!(added["data"]["symbol"], Value::Null);
    assert_eq!(added["data"]["unit"], "kg/s");
    assert_eq!(
        added["data"]["also"],
        serde_json::json!(["Massenstrom", "mdot"])
    );

    let found = json_of(&project, &["term", "MDOT"]);
    assert_eq!(found["data"]["word"], "MDOT");
    assert_eq!(found["data"]["count"], 1);
    let term = &found["data"]["terms"][0];
    assert_eq!(term["term"], "mass flow");
    assert_eq!(term["meaning"], "fluid through a section per second");
    assert_eq!(term["symbol"], Value::Null);
    assert_eq!(term["team"], false);

    let none = json_of(&project, &["term", "enthalpy"]);
    assert_eq!(none["data"]["count"], 0);
    assert_eq!(none["data"]["terms"], serde_json::json!([]));
}

#[test]
fn a_team_that_cannot_be_reached_is_a_warning_and_the_project_terms_are_still_found() {
    let project = Project::new();
    add_mass_flow(&project);
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        "\"NOWHERE/DOSSIER\"",
    )
    .expect("set the team path");
    project
        .run("anna", &["term", "Massenstrom"])
        .success()
        .stdout(MASS_FLOW_LINE)
        .stderr(contains("the team dossier cannot be reached"));
}
