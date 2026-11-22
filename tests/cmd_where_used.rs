//! Runs the built program for `where-used`: the uses of a software in the team's usage index,
//! newest first, the narrowing by --analysis and --machine, the JSON form, the team dossier itself,
//! and the refusals. The team dossier is made in a folder of its own, as anna, and the project is
//! linked to it with `team use`; the usage index is written by each test, block by block
//! (format.md §20). Each test runs in its own temporary folders, with the clock fixed.

mod common;

use common::Project;
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of these tests.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A team dossier made by `init --team-dossier --code TEAM` in its own temporary folder (its name
/// has a space and an umlaut), run as anna, with anna as its member. The project is linked to it
/// with `team use`. Returns the folder (it keeps the folder alive) and the team's DOSSIER folder.
fn linked_team(project: &Project) -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let team = folder.path().join("DOSSIER");
    dossier::config::set_list(&team, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    project
        .run("anna", &["team", "use", &team.display().to_string()])
        .success();
    (folder, team)
}

/// Writes the team's usage index: the blocks in the order given, each with its fields, as
/// format.md §20 writes them.
fn write_index(team: &Path, blocks: &[(&str, &[(&str, &str)])]) {
    let mut text = "<!-- dossier-format: 1 -->\n# Usage of TEAM\n\n".to_string();
    for (key, fields) in blocks {
        text.push_str(&format!("## {key}\n\n"));
        for (name, value) in fields.iter() {
            text.push_str(&format!("- {name}: {value}\n"));
        }
        text.push('\n');
    }
    std::fs::write(team.join("usage-index.md"), text).expect("the usage index");
}

/// The usage index of most tests: two uses of ExampleSolver in two projects, the older one in
/// P-001, and one use of OtherTool that has no analysis or machine.
fn sample_index(team: &Path) {
    write_index(
        team,
        &[
            (
                "P-001:USE-001",
                &[
                    ("date", "2026-11-12"),
                    ("code", "P-001"),
                    ("software", "ExampleSolver"),
                    ("version", "2406"),
                    ("analysis", "steady RANS, speedline"),
                    ("machine", "axial compressor"),
                    ("person", "anna"),
                ],
            ),
            (
                "P-002:USE-003",
                &[
                    ("date", "2026-11-18"),
                    ("code", "P-002"),
                    ("software", "ExampleSolver"),
                    ("version", "2411"),
                    ("analysis", "transient RANS"),
                    ("machine", "axial compressor"),
                    ("howto", "HOW-001"),
                    ("person", "ben"),
                ],
            ),
            (
                "P-002:USE-004",
                &[
                    ("date", "2026-11-19"),
                    ("code", "P-002"),
                    ("software", "OtherTool"),
                    ("version", "1.0"),
                    ("person", "ben"),
                ],
            ),
        ],
    );
}

/// The two uses of ExampleSolver of `sample_index`, newest first, as `where-used` prints them.
const TWO_USES: &str = "2026-11-18  P-002  ExampleSolver 2411  transient RANS  axial compressor  ben  howto HOW-001\n2026-11-12  P-001  ExampleSolver 2406  steady RANS, speedline  axial compressor  anna\n2 uses\n";

#[test]
fn the_uses_of_a_software_are_listed_newest_first_across_projects() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    assert_eq!(
        project.stdout("anna", &["where-used", "ExampleSolver"]),
        TWO_USES
    );
}

#[test]
fn the_software_is_matched_ignoring_case_and_a_missing_value_is_a_dash() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    assert_eq!(
        project.stdout("anna", &["where-used", "examplesolver"]),
        TWO_USES
    );
    assert_eq!(
        project.stdout("anna", &["where-used", "otherTOOL"]),
        "2026-11-19  P-002  OtherTool 1.0  -  -  ben\n1 use\n"
    );
}

#[test]
fn analysis_narrows_the_uses_and_ignores_case() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    assert_eq!(
        project.stdout(
            "anna",
            &["where-used", "ExampleSolver", "--analysis", "SPEEDLINE"]
        ),
        "2026-11-12  P-001  ExampleSolver 2406  steady RANS, speedline  axial compressor  anna\n1 use\n"
    );
    assert_eq!(
        project.stdout(
            "anna",
            &["where-used", "ExampleSolver", "--analysis", "transient"]
        ),
        "2026-11-18  P-002  ExampleSolver 2411  transient RANS  axial compressor  ben  howto HOW-001\n1 use\n"
    );
    assert_eq!(
        project.stdout(
            "anna",
            &["where-used", "ExampleSolver", "--analysis", "turbine"]
        ),
        "0 uses\n"
    );
}

#[test]
fn machine_narrows_the_uses_and_both_filters_must_match() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    assert_eq!(
        project.stdout(
            "anna",
            &["where-used", "ExampleSolver", "--machine", "COMPRESSOR"]
        ),
        TWO_USES
    );
    assert_eq!(
        project.stdout(
            "anna",
            &["where-used", "ExampleSolver", "--machine", "pump"]
        ),
        "0 uses\n"
    );
    assert_eq!(
        project.stdout(
            "anna",
            &[
                "where-used",
                "ExampleSolver",
                "--analysis",
                "steady",
                "--machine",
                "compressor"
            ]
        ),
        "2026-11-12  P-001  ExampleSolver 2406  steady RANS, speedline  axial compressor  anna\n1 use\n"
    );
}

#[test]
fn a_use_later_in_the_index_on_the_same_day_comes_first() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    write_index(
        &team,
        &[
            (
                "P-001:USE-001",
                &[
                    ("date", "2026-11-12"),
                    ("code", "P-001"),
                    ("software", "ExampleSolver"),
                    ("version", "2406"),
                    ("analysis", "first run"),
                    ("person", "anna"),
                ],
            ),
            (
                "P-001:USE-002",
                &[
                    ("date", "2026-11-12"),
                    ("code", "P-001"),
                    ("software", "ExampleSolver"),
                    ("version", "2406"),
                    ("analysis", "second run"),
                    ("person", "anna"),
                ],
            ),
        ],
    );

    let text = project.stdout("anna", &["where-used", "ExampleSolver"]);
    let second = text.find("second run").expect("the second run is listed");
    let first = text.find("first run").expect("the first run is listed");
    assert!(second < first, "the later block comes first:\n{text}");
    assert!(text.ends_with("2 uses\n"));
}

#[test]
fn json_gives_each_use_with_its_key_and_null_where_the_block_has_none() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    let value: serde_json::Value =
        serde_json::from_str(&project.stdout("anna", &["where-used", "OtherTool", "--json"]))
            .expect("the JSON parses");
    assert_eq!(value["schema"], "dossier.where-used.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], serde_json::json!([]));
    let uses = value["data"].as_array().expect("one object per use");
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0]["key"], "P-002:USE-004");
    assert_eq!(uses[0]["date"], "2026-11-19");
    assert_eq!(uses[0]["software"], "OtherTool");
    assert_eq!(uses[0]["version"], "1.0");
    assert_eq!(uses[0]["analysis"], serde_json::Value::Null);
    assert_eq!(uses[0]["machine"], serde_json::Value::Null);
    assert_eq!(uses[0]["howto"], serde_json::Value::Null);
    assert_eq!(uses[0]["person"], "ben");
}

#[test]
fn run_in_the_team_dossier_reads_the_same_index() {
    let project = Project::new();
    let (folder, team) = linked_team(&project);
    sample_index(&team);

    let text = project
        .command("anna", NOW)
        .current_dir(folder.path())
        .args(["where-used", "ExampleSolver"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(String::from_utf8(text).expect("UTF-8"), TWO_USES);
}

#[test]
fn a_team_without_a_usage_index_has_no_uses() {
    let project = Project::new();
    let (_folder, _team) = linked_team(&project);

    assert_eq!(
        project.stdout("anna", &["where-used", "ExampleSolver"]),
        "0 uses\n"
    );
}

#[test]
fn a_project_without_a_team_is_refused() {
    let project = Project::new();

    project
        .run("anna", &["where-used", "ExampleSolver"])
        .code(4)
        .stdout("")
        .stderr(contains("no team dossier"));
}

#[test]
fn a_team_that_cannot_be_reached_is_refused_with_its_path() {
    let project = Project::new();
    let missing = project.root().join("missing").join("DOSSIER");
    let stored = missing.display().to_string().replace('\\', "/");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{stored}\""),
    )
    .expect("the team path is set");

    project
        .run("anna", &["where-used", "ExampleSolver"])
        .code(4)
        .stderr(contains("the team dossier cannot be reached"));
}

#[test]
fn a_blank_software_or_text_is_refused() {
    let project = Project::new();
    let (_folder, team) = linked_team(&project);
    sample_index(&team);

    project
        .run("anna", &["where-used", "   "])
        .code(1)
        .stderr(contains("the software must not be empty"));
    project
        .run("anna", &["where-used", "ExampleSolver", "--machine", " "])
        .code(1)
        .stderr(contains("--machine must not be empty"));
}
