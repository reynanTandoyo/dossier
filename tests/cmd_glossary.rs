//! Runs the built program for `glossary`: the project's terms sorted ignoring case, the team's
//! terms marked `[team]`, the line format with empty parts left out, the count, the JSON data, a
//! team dossier that cannot be reached, a run in the team dossier itself, and `--team`. Each test
//! runs in its own temporary project folder (common::Project), with the clock fixed at
//! 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// The fixed clock of every run (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A glossary file of invented terms in the format of format.md section 20. Each block is a term
/// with its fields as `(name, value)` pairs, in the order given. `code` is the file's title.
fn glossary(code: &str, blocks: &[(&str, &[(&str, &str)])]) -> String {
    let mut text = format!("<!-- dossier-format: 1 -->\n# Glossary of {code}\n");
    for (term, fields) in blocks {
        text.push_str(&format!("\n## {term}\n\n"));
        for (name, value) in fields.iter() {
            text.push_str(&format!("- {name}: {value}\n"));
        }
    }
    text
}

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder, whose
/// name has a space and an umlaut, run as anna. Returns the folder (it keeps the folder alive)
/// and the DOSSIER folder in it.
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
    (folder, dossier)
}

/// Makes the project name the team dossier at `stored` in its settings, as a team path is kept.
fn link(project: &Project, stored: &str) {
    let settings = project.root().join("DOSSIER");
    dossier::config::set_value(&settings, "team", &format!("\"{stored}\""))
        .expect("set the team path");
}

/// The team dossier's path as the project stores it: absolute, with / for \.
fn stored(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// The project's glossary: a term with a symbol, a unit, a meaning and another spelling, a term
/// with only a meaning, and a term with nothing but its name. Invented texts.
fn project_terms() -> String {
    glossary(
        "P-001",
        &[
            (
                "pressure ratio",
                &[
                    ("meaning", "outlet pressure divided by inlet pressure"),
                    ("also", "Druckverhältnis"),
                ],
            ),
            (
                "Mass flow",
                &[
                    ("meaning", "fluid through a section per second"),
                    ("symbol", "m-dot"),
                    ("unit", "kg/s"),
                    ("also", "Massenstrom, mdot"),
                ],
            ),
        ],
    )
}

#[test]
fn two_project_terms_are_listed_sorted_ignoring_case() {
    let project = Project::new();
    project.write("DOSSIER/glossary.md", &project_terms());
    assert_eq!(
        project.stdout("anna", &["glossary"]),
        "Mass flow (m-dot, kg/s): fluid through a section per second  [also: Massenstrom, mdot]\n\
         pressure ratio: outlet pressure divided by inlet pressure  [also: Druckverhältnis]\n\
         2 terms\n"
    );
}

#[test]
fn terms_sort_ignoring_case_not_by_the_bytes_of_their_names() {
    let project = Project::new();
    // Byte order would put "Mass flow" first; ignoring case, "alpha" comes first.
    project.write(
        "DOSSIER/glossary.md",
        &glossary(
            "P-001",
            &[
                ("pressure ratio", &[("meaning", "a ratio of pressures")]),
                ("Mass flow", &[("meaning", "a flow of mass")]),
                ("alpha", &[("meaning", "a first letter")]),
            ],
        ),
    );
    assert_eq!(
        project.stdout("anna", &["glossary"]),
        "alpha: a first letter\n\
         Mass flow: a flow of mass\n\
         pressure ratio: a ratio of pressures\n\
         3 terms\n"
    );
}

#[test]
fn team_terms_are_listed_with_the_project_terms_and_marked_team() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    fs::write(
        team.join("glossary.md"),
        glossary(
            "TEAM",
            &[
                (
                    "Druckverhältnis",
                    &[
                        ("symbol", "pi"),
                        ("meaning", "outlet pressure divided by inlet pressure"),
                        ("also", "pressure ratio"),
                    ],
                ),
                ("mass flow", &[("unit", "kg/s")]),
            ],
        ),
    )
    .expect("write the team glossary");
    project.write(
        "DOSSIER/glossary.md",
        &glossary(
            "P-001",
            &[
                ("flow rate", &[("meaning", "volume of fluid per second")]),
                (
                    "Mass flow",
                    &[("meaning", "fluid through a section per second")],
                ),
            ],
        ),
    );
    link(&project, &stored(&team));

    // Equal spellings: the project's term comes before the team's, as it is listed first.
    assert_eq!(
        project.stdout("anna", &["glossary"]),
        "Druckverhältnis (pi): outlet pressure divided by inlet pressure  [also: pressure ratio]  [team]\n\
         flow rate: volume of fluid per second\n\
         Mass flow: fluid through a section per second\n\
         mass flow (kg/s)  [team]\n\
         4 terms\n"
    );
}

#[test]
fn a_line_leaves_out_the_parts_that_are_empty() {
    let project = Project::new();
    project.write(
        "DOSSIER/glossary.md",
        &glossary(
            "P-001",
            &[
                ("heat flux", &[("symbol", "q")]),
                ("speed", &[("unit", "m/s")]),
                ("stall", &[("meaning", "a flow separation on the blade")]),
                ("tip", &[]),
            ],
        ),
    );
    assert_eq!(
        project.stdout("anna", &["glossary"]),
        "heat flux (q)\n\
         speed (m/s)\n\
         stall: a flow separation on the blade\n\
         tip\n\
         4 terms\n"
    );
}

#[test]
fn no_terms_lists_zero_terms_and_one_term_is_singular() {
    let project = Project::new();
    assert_eq!(project.stdout("anna", &["glossary"]), "0 terms\n");

    project.write(
        "DOSSIER/glossary.md",
        &glossary(
            "P-001",
            &[("tip", &[("meaning", "the outer end of a blade")])],
        ),
    );
    assert_eq!(
        project.stdout("anna", &["glossary"]),
        "tip: the outer end of a blade\n1 term\n"
    );
}

#[test]
fn json_gives_the_count_and_each_term_with_null_where_a_value_is_missing() {
    let project = Project::new();
    project.write("DOSSIER/glossary.md", &project_terms());
    let stdout = project.stdout("anna", &["glossary", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("the JSON parses");
    assert_eq!(value["schema"], "dossier.glossary.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(value["data"]["count"], 2);
    assert_eq!(
        value["data"]["terms"],
        json!([
            {
                "term": "Mass flow",
                "meaning": "fluid through a section per second",
                "symbol": "m-dot",
                "unit": "kg/s",
                "also": ["Massenstrom", "mdot"],
                "team": false
            },
            {
                "term": "pressure ratio",
                "meaning": "outlet pressure divided by inlet pressure",
                "symbol": null,
                "unit": null,
                "also": ["Druckverhältnis"],
                "team": false
            }
        ])
    );
}

#[test]
fn an_unreachable_team_dossier_gives_a_warning_and_the_project_terms() {
    let project = Project::new();
    project.write(
        "DOSSIER/glossary.md",
        &glossary(
            "P-001",
            &[("flow rate", &[("meaning", "volume of fluid per second")])],
        ),
    );
    link(&project, "NOWHERE/DOSSIER");
    project
        .command("anna", NOW)
        .arg("glossary")
        .assert()
        .success()
        .stdout("flow rate: volume of fluid per second\n1 term\n")
        .stderr(contains(
            "warning: NOWHERE/DOSSIER: the team dossier cannot be reached",
        ));
}

#[test]
fn run_in_the_team_dossier_lists_its_own_terms() {
    let project = Project::new();
    let (folder, team) = team_dossier(&project);
    fs::write(
        team.join("glossary.md"),
        glossary(
            "TEAM",
            &[
                (
                    "pressure ratio",
                    &[("meaning", "outlet pressure divided by inlet pressure")],
                ),
                ("mass flow", &[("unit", "kg/s")]),
            ],
        ),
    )
    .expect("write the team glossary");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .arg("glossary")
        .assert()
        .success()
        .stdout(
            "mass flow (kg/s)  [team]\n\
             pressure ratio: outlet pressure divided by inlet pressure  [team]\n\
             2 terms\n",
        );
}

#[test]
fn team_flag_from_a_project_lists_only_the_team_terms() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    fs::write(
        team.join("glossary.md"),
        glossary("TEAM", &[("mass flow", &[("unit", "kg/s")])]),
    )
    .expect("write the team glossary");
    project.write("DOSSIER/glossary.md", &project_terms());
    link(&project, &stored(&team));
    project
        .command("anna", NOW)
        .args(["glossary", "--team"])
        .assert()
        .success()
        .stdout("mass flow (kg/s)  [team]\n1 term\n");
}
