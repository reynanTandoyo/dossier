//! Runs the built program for `search`: a word in a title is found, every word must be present,
//! drafts are found only with `--drafts` and are marked, `--kind` narrows, title matches come
//! first, `--full` prints whole entries, and no match prints `0 matches`. A project with a team
//! dossier also finds the team's entries, marked `team:` before their ID, sorted in with the
//! project's own. A German word finds the English entries through the glossaries of the project
//! and the team (R-TERM-4). A word also matches a word of the same stem, so `replaced` finds
//! `replacement`. When more than one word is typed, the entries that hold only some of them are
//! listed after the full matches, and JSON marks each result `all` or `some`. Each test runs in its
//! own temporary project with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;
use std::fs;

/// The fixed clock of the runs, as `common::Project` uses it.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// A later time than the fixed clock, for entries that must sort after the others.
const LATER: &str = "2026-11-21T09:00:00+01:00";

/// The IDs at the start of the result lines of `stdout`, in order. A result line has two spaces
/// after its ID; the count line has none.
fn result_ids(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.contains("  "))
        .filter_map(|line| line.split("  ").next())
        .map(str::to_string)
        .collect()
}

#[test]
fn a_word_in_the_title_is_found_whatever_its_case() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Steady runs converged where the transient run already showed stall.",
            ],
        )
        .success();

    let expected = "DEC-001  Use transient runs for the last three speedline points  — \
        # DEC-001 · Use transient runs for the last three speedline points\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "speedline"]), expected);
    assert_eq!(project.stdout("", &["search", "SPEEDLINE"]), expected);
}

#[test]
fn two_words_must_both_be_present() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run(
            "anna",
            &["found", "Mixing-plane runs converge at high back pressure"],
        )
        .success();

    let both = project.stdout("", &["search", "transient", "stall"]);
    assert_eq!(result_ids(&both), ["DEC-001"]);
    assert!(both.ends_with("1 match\n"), "{both}");

    // Each entry holds one of the two words, so both are listed after the line `some of the words:`.
    let some = project.stdout("", &["search", "transient", "mixing"]);
    let expected = "some of the words:\n\
        DEC-001  Use transient runs for the last three speedline points\n\
        FND-001  Mixing-plane runs converge at high back pressure\n\
        0 matches, 2 with some of the words\n";
    assert_eq!(some, expected);
}

/// Two entries for the words `valve rotor`: DEC-001 holds both in its title, FND-001 holds only
/// `rotor`, in its title and a day later.
fn valve_and_rotor(project: &Project) {
    project
        .run(
            "anna",
            &[
                "decide",
                "Valve rotor clearance is set at the cold stop",
                "--reason",
                "Set the clearance cold before the first hot run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Rotor speed drifts after restart"],
        )
        .success();
}

#[test]
fn an_entry_with_only_some_of_the_words_is_listed_after_the_full_matches() {
    let project = Project::new();
    valve_and_rotor(&project);

    let expected = "DEC-001  Valve rotor clearance is set at the cold stop  — \
        # DEC-001 · Valve rotor clearance is set at the cold stop\n\
        some of the words:\n\
        FND-001  Rotor speed drifts after restart\n\
        1 match, 1 with some of the words\n";
    assert_eq!(project.stdout("", &["search", "valve", "rotor"]), expected);

    // With --full only the full match is printed whole, and the count is of the full matches.
    let first = fs::read_to_string(project.entry_file("DEC-001")).expect("the decision file");
    assert_eq!(
        project.stdout("", &["search", "valve", "rotor", "--full"]),
        format!("{first}1 match\n")
    );
}

#[test]
fn no_entry_with_every_word_says_so_and_lists_the_partial_ones() {
    let project = Project::new();
    project
        .run("anna", &["found", "Rotor speed drifts after restart"])
        .success();

    let expected = "some of the words:\n\
        FND-001  Rotor speed drifts after restart\n\
        0 matches, 1 with some of the words\n";
    assert_eq!(project.stdout("", &["search", "rotor", "nozzle"]), expected);
}

#[test]
fn one_typed_word_never_lists_partial_results() {
    let project = Project::new();
    valve_and_rotor(&project);
    project
        .run("anna", &["found", "Nozzle wear was measured"])
        .success();

    let stdout = project.stdout("", &["search", "rotor"]);
    assert!(!stdout.contains("some of the words"), "{stdout}");
    assert!(stdout.ends_with("2 matches\n"), "{stdout}");

    let json = project.stdout("", &["--json", "search", "rotor"]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("one JSON line");
    let hits = value["data"].as_array().expect("the results");
    assert_eq!(hits.len(), 2, "{json}");
    assert!(hits.iter().all(|hit| hit["match"] == "all"), "{json}");
}

#[test]
fn json_marks_each_result_all_or_some_with_the_full_matches_first() {
    let project = Project::new();
    valve_and_rotor(&project);

    let stdout = project.stdout("", &["--json", "search", "valve", "rotor"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON line");
    let hits = value["data"].as_array().expect("the results");
    assert_eq!(hits.len(), 2, "{stdout}");
    assert_eq!(hits[0]["id"], "DEC-001");
    assert_eq!(hits[0]["match"], "all");
    assert_eq!(hits[1]["id"], "FND-001");
    assert_eq!(hits[1]["match"], "some");
    assert_eq!(hits[1]["line"], "Rotor speed drifts after restart");
}

#[test]
fn a_draft_with_some_of_the_words_is_listed_only_with_drafts() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Valve rotor clearance is set at the cold stop",
                "--reason",
                "Set the clearance cold before the first hot run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Rotor speed drifts after restart", "--later"],
        )
        .success();

    let approved = "DEC-001  Valve rotor clearance is set at the cold stop  — \
        # DEC-001 · Valve rotor clearance is set at the cold stop\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "valve", "rotor"]), approved);

    let expected = "DEC-001  Valve rotor clearance is set at the cold stop  — \
        # DEC-001 · Valve rotor clearance is set at the cold stop\n\
        some of the words:\n\
        FND-001  [draft] Rotor speed drifts after restart\n\
        1 match, 1 with some of the words\n";
    assert_eq!(
        project.stdout("", &["search", "valve", "rotor", "--drafts"]),
        expected
    );
}

#[test]
fn a_draft_is_found_only_with_drafts_and_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                "Compressor stall seen in the monitor plot",
                "--later",
            ],
        )
        .success();

    project
        .run("", &["search", "stall"])
        .success()
        .stdout("0 matches\n");

    let expected = "FND-001  [draft] Compressor stall seen in the monitor plot  — \
        # FND-001 · Compressor stall seen in the monitor plot\n\
        1 match\n";
    assert_eq!(
        project.stdout("", &["search", "stall", "--drafts"]),
        expected
    );
}

#[test]
fn kind_narrows_the_results() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Keep the stall margin check in every speedline",
                "--reason",
                "The margin is the design limit.",
            ],
        )
        .success();
    project
        .run("anna", &["found", "Stall started at the lowest mass flow"])
        .success();

    let all = project.stdout("", &["search", "stall"]);
    assert_eq!(result_ids(&all).len(), 2, "{all}");

    let decisions = project.stdout("", &["search", "stall", "--kind", "decision"]);
    assert_eq!(result_ids(&decisions), ["DEC-001"]);

    let findings = project.stdout("", &["search", "stall", "--kind", "finding"]);
    assert_eq!(result_ids(&findings), ["FND-001"]);
}

#[test]
fn title_matches_come_first_then_tags_then_the_rest_newest_first() {
    let project = Project::new();
    // A title match, the older one first.
    project
        .run(
            "anna",
            &["found", "Bearing temperature drifted during the first run"],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Bearing noise rose in the second run"],
        )
        .success();
    // A word in a tag field only.
    project
        .run_at(
            "anna",
            "2026-11-21T10:00:00+01:00",
            &[
                "found",
                "Rig vibration stayed low",
                "--machine",
                "bearing rig",
            ],
        )
        .success();
    // A word in the text only, and the newest of all.
    project
        .run_at(
            "anna",
            "2026-11-21T11:00:00+01:00",
            &[
                "assume",
                "Seal leakage is negligible",
                "--reason",
                "The bearing housing is sealed",
            ],
        )
        .success();

    let stdout = project.stdout("", &["search", "bearing"]);
    assert_eq!(
        result_ids(&stdout),
        ["FND-002", "FND-001", "FND-003", "ASM-001"],
        "{stdout}"
    );
    assert!(stdout.ends_with("4 matches\n"), "{stdout}");
}

#[test]
fn full_prints_whole_entries_separated_by_a_line_of_dashes() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &[
                "found",
                "Speedline monitor plots agree with the transient run",
            ],
        )
        .success();

    let first = fs::read_to_string(project.entry_file("FND-001")).expect("the finding file");
    let second = fs::read_to_string(project.entry_file("DEC-001")).expect("the decision file");
    let expected = format!("{first}---\n{second}2 matches\n");
    assert_eq!(
        project.stdout("", &["search", "speedline", "--full"]),
        expected
    );
}

#[test]
fn no_match_prints_zero_matches_and_exits_zero() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["found", "The mass flow still oscillated by 0.4 %"],
        )
        .success();

    project
        .run("", &["search", "zebra"])
        .success()
        .code(0)
        .stdout("0 matches\n");
    project
        .run("", &["search", "zebra", "--full"])
        .success()
        .stdout("0 matches\n");
}

#[test]
fn a_blank_word_is_refused_as_a_usage_error() {
    let project = Project::new();
    project
        .run("", &["search", "   "])
        .code(2)
        .stderr(contains("needs at least one word that is not blank"));
}

#[test]
fn json_gives_the_id_title_status_and_line_of_each_result() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use transient runs for the last three speedline points",
                "--reason",
                "Rotating stall was visible in the transient run.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &["found", "Speedline plot kept for review", "--later"],
        )
        .success();

    let stdout = project.stdout("", &["--json", "search", "speedline", "--drafts"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.search.v1");
    assert_eq!(value["ok"], true);
    let expected = serde_json::json!([
        {
            "id": "FND-001",
            "title": "Speedline plot kept for review",
            "status": "draft",
            "line": "# FND-001 · Speedline plot kept for review",
            "match": "all",
            "place": "project",
        },
        {
            "id": "DEC-001",
            "title": "Use transient runs for the last three speedline points",
            "status": "approved",
            "line": "# DEC-001 · Use transient runs for the last three speedline points",
            "match": "all",
            "place": "project",
        },
    ]);
    assert_eq!(value["data"], expected);
}

#[test]
fn an_approved_lesson_changed_by_hand_is_marked() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Report section 4.2",
                "--confidence",
                "measured",
            ],
        )
        .success();

    let heading = "LES-001  Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin";
    assert_eq!(
        project.stdout("", &["search", "stall"]),
        format!("{heading}\n1 match\n")
    );

    let path = project.entry_file("LES-001");
    let mut text = fs::read_to_string(&path).expect("the lesson file");
    text.push_str("\nChecked again by hand.\n");
    fs::write(&path, text).expect("the lesson file is changed");

    let marked = "LES-001  [changed after approval] Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "stall"]), marked);
}

/// A team dossier in a folder of its own, with a space and a non-ASCII letter in its name. It is
/// made with `init --team-dossier`, anna is its only member, and the project names it as its team.
/// Returns the folder that holds the team's DOSSIER: the test keeps it alive.
fn linked_team(project: &Project) -> tempfile::TempDir {
    let team = tempfile::Builder::new()
        .prefix("Team Ärger ")
        .tempdir()
        .expect("a temporary team folder");
    project
        .command("anna", NOW)
        .current_dir(team.path())
        .args(["init", "--code", "TEAM", "--team-dossier"])
        .assert()
        .success();
    let team_dossier = team.path().join("DOSSIER");
    dossier::config::set_list(&team_dossier, "members", &["anna".to_string()])
        .expect("anna is a member of the team");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{}\"", stored_path(&team_dossier)),
    )
    .expect("the project names its team");
    team
}

/// A path as the settings store it: with / for every separator. The caller adds the quotes.
fn stored_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// A lesson that anna learns in the team dossier at `now`, with the statement given. The team's
/// folder is the working folder, so the command finds the team's DOSSIER.
fn learn_in_team(team: &tempfile::TempDir, project: &Project, now: &str, statement: &str) {
    project
        .command("anna", now)
        .current_dir(team.path())
        .args([
            "learn",
            statement,
            "--valid-for",
            "Transonic front stages",
            "--evidence",
            "Report section 4.2",
            "--confidence",
            "measured",
        ])
        .assert()
        .success();
}

#[test]
fn a_team_lesson_is_found_from_the_project_with_the_team_mark() {
    let project = Project::new();
    let team = linked_team(&project);
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the stall margin",
    );

    let heading = "team:LES-001  Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin";
    assert_eq!(
        project.stdout("", &["search", "stall"]),
        format!("{heading}\n1 match\n")
    );
}

#[test]
fn json_gives_the_place_of_each_result() {
    let project = Project::new();
    let team = linked_team(&project);
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the stall margin",
    );
    project
        .run("anna", &["found", "Stall seen at the lowest mass flow"])
        .success();

    let stdout = project.stdout("", &["--json", "search", "stall"]);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("one JSON line");
    let expected = serde_json::json!([
        {
            "id": "FND-001",
            "title": "Stall seen at the lowest mass flow",
            "status": "approved",
            "line": "# FND-001 · Stall seen at the lowest mass flow",
            "match": "all",
            "place": "project",
        },
        {
            "id": "LES-001",
            "title": "Steady runs overpredict the stall margin",
            "status": "approved",
            "line": "# LES-001 · Steady runs overpredict the stall margin",
            "match": "all",
            "place": "team",
        },
    ]);
    assert_eq!(value["data"], expected);
}

#[test]
fn team_results_are_sorted_in_with_the_project_ones() {
    let project = Project::new();
    let team = linked_team(&project);
    // The project's LES-001 and the team's LES-001 are made at the same time: the project's
    // comes first. The team's LES-002 is the newest, so it comes first of all.
    project
        .run(
            "anna",
            &[
                "learn",
                "Steady runs overpredict the stall margin",
                "--valid-for",
                "Transonic front stages",
                "--evidence",
                "Report section 4.2",
                "--confidence",
                "measured",
            ],
        )
        .success();
    learn_in_team(
        &team,
        &project,
        NOW,
        "Stall margin of steady runs was measured on the rig",
    );
    learn_in_team(&team, &project, LATER, "Stall onset moves with the tip gap");

    let stdout = project.stdout("", &["search", "stall"]);
    assert_eq!(
        result_ids(&stdout),
        ["team:LES-002", "LES-001", "team:LES-001"],
        "{stdout}"
    );
    assert!(stdout.ends_with("3 matches\n"), "{stdout}");
}

#[test]
fn search_with_team_runs_in_the_team_and_marks_its_entries() {
    let project = Project::new();
    let team = linked_team(&project);
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the stall margin",
    );
    project
        .run("anna", &["found", "Stall seen at the lowest mass flow"])
        .success();

    // --team searches the team dossier alone, so the project's finding is not listed.
    let stdout = project.stdout("", &["--team", "search", "stall"]);
    assert_eq!(result_ids(&stdout), ["team:LES-001"], "{stdout}");
    assert!(stdout.ends_with("1 match\n"), "{stdout}");
}

#[test]
fn an_unreachable_team_leaves_the_project_search_working() {
    let project = Project::new();
    project
        .run("anna", &["found", "Stall seen at the lowest mass flow"])
        .success();
    // A team folder that does not exist: the project names it, but nothing is there.
    let missing = project.root().join("Nowhere").join("DOSSIER");
    dossier::config::set_value(
        &project.root().join("DOSSIER"),
        "team",
        &format!("\"{}\"", stored_path(&missing)),
    )
    .expect("the project names a team that is not there");

    let expected = "FND-001  Stall seen at the lowest mass flow  — \
        # FND-001 · Stall seen at the lowest mass flow\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "stall"]), expected);
}

/// A glossary file in the format of format.md section 20, with one invented term, its meaning
/// and its other spelling `also`, written into the dossier folder `dossier`.
fn write_glossary(dossier: &std::path::Path, code: &str, term: &str, also: &str) {
    let text = format!(
        "<!-- dossier-format: 1 -->\n# Glossary of {code}\n\n## {term}\n\n\
         - meaning: an invented test term\n- also: {also}\n"
    );
    fs::write(dossier.join("glossary.md"), text).expect("the glossary file is written");
}

#[test]
fn a_german_word_finds_the_english_entries_through_the_project_glossary() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["found", "The mass flow still oscillated by 0.4 %"],
        )
        .success();
    let expected = "FND-001  The mass flow still oscillated by 0.4 %  — \
        # FND-001 · The mass flow still oscillated by 0.4 %\n\
        1 match\n";

    project
        .run("", &["search", "Massenstrom"])
        .success()
        .stdout("0 matches\n");

    write_glossary(
        &project.root().join("DOSSIER"),
        "P-001",
        "mass flow",
        "Massenstrom",
    );
    assert_eq!(project.stdout("", &["search", "Massenstrom"]), expected);
    assert_eq!(project.stdout("", &["search", "MASSENSTROM"]), expected);
    assert_eq!(project.stdout("", &["search", "mass flow"]), expected);
}

#[test]
fn a_spelling_found_in_the_text_is_the_line_shown() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "assume",
                "Seal leakage is negligible",
                "--reason",
                "Measured mass flow through the seal stayed under 1 %.",
            ],
        )
        .success();
    write_glossary(
        &project.root().join("DOSSIER"),
        "P-001",
        "mass flow",
        "Massenstrom",
    );

    let stdout = project.stdout("", &["search", "Massenstrom"]);
    assert_eq!(result_ids(&stdout), ["ASM-001"], "{stdout}");
    assert!(
        stdout.contains("— Measured mass flow through the seal stayed under 1 %.\n"),
        "{stdout}"
    );
    assert!(stdout.ends_with("1 match\n"), "{stdout}");
}

#[test]
fn a_word_matches_a_word_of_the_same_stem_and_a_title_match_comes_first() {
    let project = Project::new();
    project
        .run("anna", &["found", "Reusing one valve connection"])
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &[
                "assume",
                "Seal leakage is negligible",
                "--reason",
                "The connection is reused on each run.",
            ],
        )
        .success();

    // `reuse` matches `Reusing` in the title, so the older finding is in the title group and comes
    // before the newer assumption, which has `reused` only in its text.
    let stdout = project.stdout("", &["search", "connection", "reuse"]);
    assert_eq!(result_ids(&stdout), ["FND-001", "ASM-001"], "{stdout}");
    assert!(stdout.ends_with("2 matches\n"), "{stdout}");
}

#[test]
fn a_word_found_by_its_stem_is_listed_with_the_line_that_holds_it() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "assume",
                "Seal wear is slow",
                "--reason",
                "The seal is replaced at each overhaul.",
            ],
        )
        .success();
    project
        .run_at(
            "anna",
            LATER,
            &[
                "assume",
                "Seal life is long",
                "--reason",
                "Seal replacement costs little.",
            ],
        )
        .success();

    let expected = "ASM-002  Seal life is long  — Seal replacement costs little.\n\
        ASM-001  Seal wear is slow  — The seal is replaced at each overhaul.\n\
        2 matches\n";
    assert_eq!(project.stdout("", &["search", "replaced"]), expected);
}

#[test]
fn the_team_glossary_is_used_by_the_project_search() {
    let project = Project::new();
    let team = linked_team(&project);
    write_glossary(
        &team.path().join("DOSSIER"),
        "TEAM",
        "pressure ratio",
        "Druckverhältnis",
    );
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the pressure ratio",
    );
    project
        .run("anna", &["found", "Stall margin of the first stage"])
        .success();

    let stdout = project.stdout("", &["search", "Druckverhältnis"]);
    assert_eq!(result_ids(&stdout), ["team:LES-001"], "{stdout}");
    assert!(stdout.ends_with("1 match\n"), "{stdout}");
}

#[test]
fn team_search_uses_only_the_team_glossary() {
    let project = Project::new();
    let team = linked_team(&project);
    write_glossary(
        &project.root().join("DOSSIER"),
        "P-001",
        "pressure ratio",
        "Druckverhältnis",
    );
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the pressure ratio",
    );

    // The project's term is not in the team's glossary, so `--team` finds nothing.
    project
        .run("", &["--team", "search", "Druckverhältnis"])
        .success()
        .stdout("0 matches\n");
    // The project's own search does use the project's term.
    let stdout = project.stdout("", &["search", "Druckverhältnis"]);
    assert_eq!(result_ids(&stdout), ["team:LES-001"], "{stdout}");
}

#[test]
fn an_unreadable_glossary_is_a_warning_and_the_search_goes_on() {
    let project = Project::new();
    project
        .run("anna", &["found", "Stall seen at the lowest mass flow"])
        .success();
    fs::write(
        project.root().join("DOSSIER").join("glossary.md"),
        [0xff, 0xfe, b'#'],
    )
    .expect("the glossary file is written");

    let assert = project
        .run("", &["search", "stall"])
        .success()
        .stderr(contains("warning: glossary.md: not UTF-8"));
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("UTF-8 output");
    let expected = "FND-001  Stall seen at the lowest mass flow  — \
        # FND-001 · Stall seen at the lowest mass flow\n\
        1 match\n";
    assert_eq!(stdout, expected);
}

#[test]
fn a_review_date_before_the_day_of_the_run_marks_the_result_as_due() {
    let project = Project::new();
    project
        .run("anna", &["found", "Stall seen at the lowest mass flow"])
        .success();
    project
        .run("anna", &["set", "FND-001", "--field", "review=2026-11-20"])
        .success();

    // On the day of the review date the entry is not yet due.
    let plain = "FND-001  Stall seen at the lowest mass flow  — \
        # FND-001 · Stall seen at the lowest mass flow\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "stall"]), plain);

    // The next day it is.
    project
        .run_at("", LATER, &["search", "stall"])
        .success()
        .stdout(
            "FND-001  [due for review] Stall seen at the lowest mass flow  — \
        # FND-001 · Stall seen at the lowest mass flow\n\
        1 match\n",
        );
}

#[test]
fn the_marks_of_a_result_come_in_the_order_show_gives_them() {
    let project = Project::new();
    project.write(
        "DOSSIER/findings/FND-001-stall-marks.md",
        "<!-- dossier-format: 1 -->\n# FND-001 · Stall seen in the rig\n\n- kind: finding\n- status: approved\n- created: 2026-11-10T09:00:00+01:00\n- author: anna\n- review: 2026-11-19\n- imported: yes\n- rests-on: ASM-001\n\n## Observation\n\nAn invented observation.\n",
    );
    project.write(
        "DOSSIER/findings/FND-002-stall-draft.md",
        "<!-- dossier-format: 1 -->\n# FND-002 · Stall draft\n\n- kind: finding\n- status: draft\n- created: 2026-11-10T09:00:00+01:00\n- author: anna\n- imported: yes\n\n## Observation\n\nAn invented observation.\n",
    );

    assert_eq!(
        project.stdout("", &["search", "stall", "--drafts"]),
        "FND-001  [due for review] [imported] [rests on ASM-001] Stall seen in the rig  — # FND-001 · Stall seen in the rig\nFND-002  [draft] [imported] Stall draft  — # FND-002 · Stall draft\n2 matches\n"
    );
}

#[test]
fn an_approved_how_to_changed_by_hand_is_marked_in_search() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "add",
                "howto",
                "--title",
                "Export the rig mesh",
                "--field",
                "software=MeshTool 2.4",
                "--section",
                "Purpose=Export the mesh of an invented test rig.",
                "--section",
                "Steps=Open the invented rig file, then export the mesh.",
            ],
        )
        .success();
    assert_eq!(
        project.stdout("", &["search", "mesh"]),
        "HOW-001  Export the rig mesh  — # HOW-001 · Export the rig mesh\n1 match\n"
    );

    let howto = project.entry_file("HOW-001");
    let text = fs::read_to_string(&howto).expect("the how-to file");
    let changed = text.replace("export the mesh.\n", "export the mesh twice.\n");
    assert_ne!(changed, text, "the steps are in the file");
    fs::write(&howto, changed).expect("the how-to is changed by hand");

    assert_eq!(
        project.stdout("", &["search", "mesh"]),
        "HOW-001  [changed after approval] Export the rig mesh  — # HOW-001 · Export the rig mesh\n1 match\n"
    );
}

#[test]
fn a_tool_whose_script_changed_since_approval_is_marked_in_search() {
    let project = Project::new();
    project.write("scripts/count.py", "print('invented count')\n");
    let checksum = dossier::filesum::checksum(&project.root().join("scripts/count.py"))
        .expect("the checksum of the script");
    let recorded = format!("checksum={checksum}");
    project
        .run(
            "anna",
            &[
                "add",
                "tool",
                "--title",
                "Count solver warnings",
                "--field",
                "type=script",
                "--field",
                "location=scripts/count.py",
                "--field",
                &recorded,
                "--section",
                "Purpose=Counts the warnings in an invented solver log.",
            ],
        )
        .success();
    assert_eq!(
        project.stdout("", &["search", "warnings"]),
        "TOOL-001  Count solver warnings  — # TOOL-001 · Count solver warnings\n1 match\n"
    );

    project.write(
        "scripts/count.py",
        "print('invented count, edited by hand')\n",
    );
    assert_eq!(
        project.stdout("", &["search", "warnings"]),
        "TOOL-001  [file changed since approval] Count solver warnings  — # TOOL-001 · Count solver warnings\n1 match\n"
    );
}

#[test]
fn a_team_entry_due_for_review_is_marked_after_its_team_prefix() {
    let project = Project::new();
    let team = linked_team(&project);
    learn_in_team(
        &team,
        &project,
        NOW,
        "Steady runs overpredict the stall margin",
    );
    project
        .command("anna", NOW)
        .current_dir(team.path())
        .args(["set", "LES-001", "--field", "review=2026-11-19"])
        .assert()
        .success();

    project
        .run_at("", NOW, &["search", "stall"])
        .success()
        .stdout(
            "team:LES-001  [due for review] Steady runs overpredict the stall margin  — \
        # LES-001 · Steady runs overpredict the stall margin\n\
        1 match\n",
        );
}

/// The problem of the investigations that the state-mark tests open with `problem`.
const PROBLEM: &str = "The density residual grows from iteration 40 on.";

#[test]
fn an_investigation_is_marked_open_and_then_solved() {
    let project = Project::new();
    project.run("anna", &["problem", PROBLEM]).success();

    let open = "INV-001  [open] The density residual grows from iteration 40 on.  — \
        # INV-001 · The density residual grows from iteration 40 on.\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "residual"]), open);

    project
        .run(
            "anna",
            &["solved", "INV-001", "The outlet flux had the wrong sign"],
        )
        .success();

    let solved = "INV-001  [solved] The density residual grows from iteration 40 on.  — \
        # INV-001 · The density residual grows from iteration 40 on.\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "residual"]), solved);
}

#[test]
fn an_investigation_given_up_is_marked_given_up() {
    let project = Project::new();
    project.run("anna", &["problem", PROBLEM]).success();
    project
        .run(
            "anna",
            &[
                "gave-up",
                "INV-001",
                "--reason",
                "The rig is booked for another test.",
            ],
        )
        .success();

    let given_up = "INV-001  [given up] The density residual grows from iteration 40 on.  — \
        # INV-001 · The density residual grows from iteration 40 on.\n\
        1 match\n";
    assert_eq!(project.stdout("", &["search", "residual"]), given_up);
}

#[test]
fn an_assumption_with_its_own_state_gets_no_investigation_mark() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "assume",
                "Seal leakage is negligible",
                "--reason",
                "The bearing housing is sealed",
            ],
        )
        .success();

    assert_eq!(
        project.stdout("", &["search", "leakage"]),
        "ASM-001  Seal leakage is negligible  — # ASM-001 · Seal leakage is negligible\n1 match\n"
    );
}

/// Runs `learn` as anna in the project, with an invented statement, the fixed evidence and the
/// `extra` options (tags). The title is the statement.
fn learn(project: &Project, statement: &str, extra: &[&str]) {
    let mut args = vec![
        "learn",
        statement,
        "--valid-for",
        "Invented test rig",
        "--evidence",
        "Invented log",
        "--confidence",
        "measured",
    ];
    args.extend_from_slice(extra);
    project.run("anna", &args).success();
}

/// Runs `learn` as anna in the team's folder, as `learn` in `learn_in_team` does, with the `extra`
/// options added.
fn learn_in_team_with(
    team: &tempfile::TempDir,
    project: &Project,
    statement: &str,
    extra: &[&str],
) {
    let mut args = vec![
        "learn",
        statement,
        "--valid-for",
        "Invented test rig",
        "--evidence",
        "Invented log",
        "--confidence",
        "measured",
    ];
    args.extend_from_slice(extra);
    project
        .command("anna", NOW)
        .current_dir(team.path())
        .args(&args)
        .assert()
        .success();
}

#[test]
fn similar_lists_the_entries_that_share_a_tag_or_a_title_word_and_not_the_entry_itself() {
    let project = Project::new();
    // LES-001 is the entry; LES-002 shares its machine and the title word `runs`; LES-003 shares
    // nothing with it.
    learn(
        &project,
        "Steady runs overpredict stall margin",
        &["--machine", "axial compressor", "--analysis", "steady RANS"],
    );
    learn(
        &project,
        "Transient runs at high speed",
        &["--machine", "axial compressor"],
    );
    learn(
        &project,
        "Fan blade flutter is damped at low speed",
        &[
            "--machine",
            "radial turbine",
            "--analysis",
            "modal analysis",
        ],
    );

    // LES-002 shares the machine, a tag, so the reason is `tag match` (not the title word).
    let stdout = project.stdout("", &["search", "--similar", "LES-001"]);
    assert_eq!(
        stdout,
        "LES-002  Transient runs at high speed  — tag match\n1 match\n"
    );

    let json = project.stdout("", &["--json", "search", "--similar", "LES-001"]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("one JSON line");
    assert_eq!(value["schema"], "dossier.search.v1");
    assert_eq!(value["ok"], true);
    let expected = serde_json::json!([
        {
            "id": "LES-002",
            "title": "Transient runs at high speed",
            "status": "approved",
            "line": "Transient runs at high speed",
            "place": "project",
            "reason": "tag match",
        },
    ]);
    assert_eq!(value["data"], expected);
}

#[test]
fn similar_says_why_each_entry_is_similar_and_a_word_search_gives_no_reason() {
    let project = Project::new();
    learn(
        &project,
        "Steady runs overpredict stall margin",
        &["--machine", "axial compressor"],
    );
    // A tag match: the machine. The title word `runs` also matches, but a tag match is the reason.
    learn(
        &project,
        "Transient runs at high speed",
        &["--machine", "axial compressor"],
    );
    // A word match only: it shares the title words `stall` and `margin` and no tag.
    project
        .run("anna", &["found", "Stall margin reading"])
        .success();

    let stdout = project.stdout("", &["search", "--similar", "LES-001"]);
    assert_eq!(
        stdout,
        "FND-001  Stall margin reading  — word match\n\
        LES-002  Transient runs at high speed  — tag match\n\
        2 matches\n"
    );

    let json = project.stdout("", &["--json", "search", "--similar", "LES-001"]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("one JSON line");
    assert_eq!(value["data"][0]["id"], "FND-001");
    assert_eq!(value["data"][0]["reason"], "word match");
    assert_eq!(value["data"][1]["id"], "LES-002");
    assert_eq!(value["data"][1]["reason"], "tag match");

    // A word search keeps its matched line and gives no reason, in text or in JSON.
    let plain = project.stdout("", &["search", "stall"]);
    assert!(!plain.contains("word match"), "{plain}");
    let json = project.stdout("", &["--json", "search", "stall"]);
    let value: serde_json::Value = serde_json::from_str(&json).expect("one JSON line");
    let hits = value["data"].as_array().expect("an array of results");
    assert!(!hits.is_empty(), "{json}");
    assert!(hits.iter().all(|hit| hit.get("reason").is_none()), "{json}");
}

#[test]
fn similar_orders_by_the_points_shared_and_not_by_age() {
    let project = Project::new();
    learn(
        &project,
        "Steady runs overpredict stall margin",
        &["--machine", "axial compressor", "--analysis", "steady RANS"],
    );
    // Oldest: a title word and a tag, 9 points.
    learn(
        &project,
        "Transient runs at high speed",
        &["--machine", "axial compressor"],
    );
    // Newer: a title word only, 6 points (the title and the observation both hold it).
    project
        .run_at("anna", LATER, &["found", "Stall seen in rig"])
        .success();
    // Newest: a tag only, 3 points.
    project
        .run_at(
            "anna",
            "2026-11-21T11:00:00+01:00",
            &["found", "Mixing plane check", "--analysis", "steady RANS"],
        )
        .success();

    let stdout = project.stdout("", &["search", "--similar", "LES-001"]);
    assert_eq!(
        result_ids(&stdout),
        ["LES-002", "FND-001", "FND-002"],
        "{stdout}"
    );
    assert!(stdout.ends_with("3 matches\n"), "{stdout}");
}

#[test]
fn similar_lists_at_most_ten_entries() {
    let project = Project::new();
    learn(&project, "Steady runs overpredict stall margin", &[]);
    for _ in 0..12 {
        project
            .run("anna", &["found", "Stall margin reading"])
            .success();
    }

    let stdout = project.stdout("", &["search", "--similar", "LES-001"]);
    assert_eq!(result_ids(&stdout).len(), 10, "{stdout}");
    assert!(stdout.ends_with("10 matches\n"), "{stdout}");
}

#[test]
fn similar_applies_the_filter_so_a_draft_is_listed_only_with_drafts() {
    let project = Project::new();
    learn(&project, "Steady runs overpredict stall margin", &[]);
    project
        .run("anna", &["found", "Stall margin read by hand", "--later"])
        .success();

    project
        .run("", &["search", "--similar", "LES-001"])
        .success()
        .stdout("0 matches\n");
    assert_eq!(
        project.stdout("", &["search", "--similar", "LES-001", "--drafts"]),
        "FND-001  [draft] Stall margin read by hand  — word match\n1 match\n"
    );
}

#[test]
fn similar_reads_the_team_dossier_and_leaves_out_only_the_entry_itself() {
    let project = Project::new();
    let team = linked_team(&project);
    learn(
        &project,
        "Steady runs overpredict stall margin",
        &["--machine", "axial compressor"],
    );
    // The team's LES-001 has the same title and tag. It is another entry, so it is listed.
    learn_in_team_with(
        &team,
        &project,
        "Steady runs overpredict stall margin",
        &["--machine", "axial compressor"],
    );

    let stdout = project.stdout("", &["search", "--similar", "LES-001"]);
    assert_eq!(result_ids(&stdout), ["team:LES-001"], "{stdout}");

    // With the place `team:`, the team's LES-001 is the entry; the project's is listed instead.
    let stdout = project.stdout("", &["search", "--similar", "team:LES-001"]);
    assert_eq!(result_ids(&stdout), ["LES-001"], "{stdout}");
}

#[test]
fn similar_with_words_is_a_usage_error_and_no_words_without_similar_is_too() {
    let project = Project::new();
    learn(&project, "Steady runs overpredict stall margin", &[]);

    project
        .run("", &["search", "stall", "--similar", "LES-001"])
        .code(2)
        .stderr(contains("not both"));
    project.run("", &["search"]).code(2);
}

#[test]
fn an_unknown_similar_id_is_not_found() {
    let project = Project::new();
    project
        .run("", &["search", "--similar", "LES-404"])
        .code(1)
        .stderr(contains("no entry LES-404"));
    // A team entry in a project with no team dossier: the team cannot be reached.
    project
        .run("", &["search", "--similar", "team:LES-001"])
        .code(4);
}

#[test]
fn similar_lists_an_entry_whose_title_holds_other_forms_of_the_words() {
    let project = Project::new();
    learn(&project, "Reusing one valve connection", &[]);
    learn(&project, "Connection reuse on the rig", &[]);

    assert_eq!(
        project.stdout("", &["search", "--similar", "LES-001"]),
        "LES-002  Connection reuse on the rig  — word match\n1 match\n"
    );
}

#[test]
fn similar_leaves_out_an_entry_that_shares_only_the_common_words() {
    let project = Project::new();
    learn(&project, "Reusing the valve and connection", &[]);
    // The two entries share only `the` and `and`, which relate nothing (R-FIND-1).
    learn(&project, "Pump and the drive shaft", &[]);

    project
        .run("", &["search", "--similar", "LES-001"])
        .success()
        .stdout("0 matches\n");
}
