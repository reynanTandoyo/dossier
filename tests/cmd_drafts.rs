//! Runs the built program for `drafts`: the drafts oldest first with who proposed each, the task
//! and the age; `--mine` for the calling person; the `(old)` mark; the order by instant rather
//! than by the text of the stamp; the JSON data; `--imported` grouped by source document with the
//! `[imported]` mark elsewhere; and a broken entry file that gives a warning while the list still
//! works. Each test runs in its own temporary project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00 unless a test gives
//! another time.

mod common;

use common::Project;
use predicates::str::contains;
use serde_json::{Value, json};

/// Invented text for the approved finding and the two draft lessons.
const OBSERVATION: &str = "The invented test rig stalls above a tip Mach number of 1.1.";
const FINDING_TITLE: &str = "Stall above Mach 1.1";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";
const BEN_STATEMENT: &str = "Invented: a coarse mesh overpredicts the stall margin.";
const BEN_TITLE: &str = "Coarse mesh margin";
const ANNA_STATEMENT: &str = "Invented: a wall function shifts the speedline.";
const ANNA_TITLE: &str = "Wall function shift";
const EARLY_STATEMENT: &str = "Invented: the rig needs a warm-up before the first speedline.";
const EARLY_TITLE: &str = "Rig warm-up";
const OWN_STATEMENT: &str = "Invented: a lesson drafted from the project's own runs.";
const OWN_TITLE: &str = "Own lesson";
const REPORT_ONE: &str = "old/report one.docx";
const REPORT_TWO: &str = "old/report two.docx";

/// `learn` arguments for a lesson with the invented texts above, followed by `extra`.
fn learn<'a>(statement: &'a str, title: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "learn",
        statement,
        "--valid-for",
        VALID_FOR,
        "--evidence",
        EVIDENCE,
        "--title",
        title,
        "--confidence",
        "simulated",
    ];
    args.extend_from_slice(extra);
    args
}

/// A project with two drafts and one approved finding. LES-001 is a lesson that ben-codex
/// proposed on 2026-11-19 for task T-042. LES-002 is a lesson that anna-claude proposed on
/// 2026-11-18 with no task, so it is the older one although its ID is the later. FND-001 is an
/// approved finding by anna: it is not a draft and is not listed.
fn project_with_two_drafts() -> Project {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                OBSERVATION,
                "--title",
                FINDING_TITLE,
                "--evidence",
                EVIDENCE,
            ],
        )
        .success();
    project
        .run_at(
            "ben-codex",
            "2026-11-19T09:00:00+01:00",
            &learn(BEN_STATEMENT, BEN_TITLE, &["--task", "T-042"]),
        )
        .success();
    project
        .run_at(
            "anna-claude",
            "2026-11-18T09:00:00+01:00",
            &learn(ANNA_STATEMENT, ANNA_TITLE, &[]),
        )
        .success();
    project
}

#[test]
fn the_drafts_are_listed_oldest_first_with_proposer_task_and_age() {
    let project = project_with_two_drafts();

    assert_eq!(
        project.stdout("", &["drafts"]),
        "LES-002  lesson  Wall function shift  by anna via anna-claude  2 days\nLES-001  lesson  Coarse mesh margin  by ben via ben-codex  task T-042  1 day\n2 drafts\n"
    );
}

#[test]
fn mine_shows_only_the_drafts_proposed_for_the_calling_person() {
    let project = project_with_two_drafts();

    assert_eq!(
        project.stdout("anna", &["drafts", "--mine"]),
        "LES-002  lesson  Wall function shift  by anna via anna-claude  2 days\n1 draft\n"
    );
    assert_eq!(
        project.stdout("ben-codex", &["drafts", "--mine"]),
        "LES-001  lesson  Coarse mesh margin  by ben via ben-codex  task T-042  1 day\n1 draft\n"
    );
}

#[test]
fn mine_without_a_name_is_refused() {
    let project = project_with_two_drafts();

    project
        .run("", &["drafts", "--mine"])
        .failure()
        .code(2)
        .stderr(contains("this command needs a name"));
}

#[test]
fn a_draft_older_than_draft_days_is_marked_old_and_one_at_the_limit_is_not() {
    let project = Project::new();
    project
        .run_at(
            "anna-claude",
            "2026-10-31T10:00:00+01:00",
            &learn(ANNA_STATEMENT, ANNA_TITLE, &[]),
        )
        .success();
    // Exactly 14 days (draft_days) old: not older than the setting, so no mark.
    project
        .run_at(
            "ben-codex",
            "2026-11-06T10:00:00+01:00",
            &learn(BEN_STATEMENT, BEN_TITLE, &[]),
        )
        .success();

    assert_eq!(
        project.stdout("", &["drafts"]),
        "LES-001  lesson  Wall function shift  by anna via anna-claude  20 days  (old)\nLES-002  lesson  Coarse mesh margin  by ben via ben-codex  14 days\n2 drafts\n"
    );
}

#[test]
fn drafts_are_put_in_order_by_the_instant_not_by_the_text_of_the_stamp() {
    let project = Project::new();
    // 2026-11-19T10:00-05:00 is 15:00 UTC, which is later than 11:00 UTC, the instant of
    // 2026-11-19T12:00+01:00. The text of the first stamp sorts before the second.
    project
        .run_at(
            "anna-claude",
            "2026-11-19T10:00:00-05:00",
            &learn(ANNA_STATEMENT, ANNA_TITLE, &[]),
        )
        .success();
    project
        .run_at(
            "ben-codex",
            "2026-11-19T12:00:00+01:00",
            &learn(BEN_STATEMENT, BEN_TITLE, &[]),
        )
        .success();

    project
        .run_at(
            "",
            "2026-11-21T10:00:00+01:00",
            &["drafts"],
        )
        .success()
        .stdout("LES-002  lesson  Coarse mesh margin  by ben via ben-codex  1 day\nLES-001  lesson  Wall function shift  by anna via anna-claude  1 day\n2 drafts\n");
}

#[test]
fn with_no_drafts_the_count_is_zero() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "found",
                OBSERVATION,
                "--title",
                FINDING_TITLE,
                "--evidence",
                EVIDENCE,
            ],
        )
        .success();

    assert_eq!(project.stdout("", &["drafts"]), "0 drafts\n");
}

#[test]
fn json_gives_one_object_per_draft_with_null_where_a_value_is_missing() {
    let project = project_with_two_drafts();

    let stdout = project.stdout("", &["drafts", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("the JSON parses");

    assert_eq!(value["schema"], "dossier.drafts.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["warnings"], json!([]));
    assert_eq!(
        value["data"],
        json!([
            {
                "id": "LES-002",
                "kind": "lesson",
                "title": ANNA_TITLE,
                "author": "anna",
                "via": "anna-claude",
                "task": null,
                "created": "2026-11-18T09:00:00+01:00",
                "age_days": 2
            },
            {
                "id": "LES-001",
                "kind": "lesson",
                "title": BEN_TITLE,
                "author": "ben",
                "via": "ben-codex",
                "task": "T-042",
                "created": "2026-11-19T09:00:00+01:00",
                "age_days": 1
            }
        ])
    );
}

#[test]
fn a_broken_entry_file_gives_a_warning_and_the_list_still_works() {
    let project = project_with_two_drafts();
    project.write(
        "DOSSIER/lessons/LES-009-broken.md",
        "<!-- dossier-format: 1 -->\nthis file has no heading\n",
    );

    assert_eq!(
        project.stdout("", &["drafts"]),
        "LES-002  lesson  Wall function shift  by anna via anna-claude  2 days\nLES-001  lesson  Coarse mesh margin  by ben via ben-codex  task T-042  1 day\n2 drafts\n"
    );
    project.run("", &["drafts"]).success().stderr(contains(
        "warning: DOSSIER/lessons/LES-009-broken.md: line 2: no heading",
    ));
}

/// A project with four drafts, three of them imported (`--from-document`). LES-001 comes from
/// "old/report two.docx" (ben-codex, 2026-11-19). LES-002 comes from "old/report one.docx"
/// (anna-claude, 2026-11-18). LES-003 comes from the same document as LES-002, by anna-claude on
/// 2026-11-17: it has the later ID but is the older draft. LES-004 is an own draft by ben-codex on
/// 2026-11-18 at noon, with no document, so it is not imported.
fn project_with_imported_drafts() -> Project {
    let project = Project::new();
    project
        .run_at(
            "ben-codex",
            "2026-11-19T09:00:00+01:00",
            &learn(BEN_STATEMENT, BEN_TITLE, &["--from-document", REPORT_TWO]),
        )
        .success();
    project
        .run_at(
            "anna-claude",
            "2026-11-18T09:00:00+01:00",
            &learn(ANNA_STATEMENT, ANNA_TITLE, &["--from-document", REPORT_ONE]),
        )
        .success();
    project
        .run_at(
            "anna-claude",
            "2026-11-17T09:00:00+01:00",
            &learn(
                EARLY_STATEMENT,
                EARLY_TITLE,
                &["--from-document", REPORT_ONE],
            ),
        )
        .success();
    project
        .run_at(
            "ben-codex",
            "2026-11-18T12:00:00+01:00",
            &learn(OWN_STATEMENT, OWN_TITLE, &[]),
        )
        .success();
    project
}

#[test]
fn imported_drafts_are_grouped_by_their_document_with_the_oldest_first_in_each() {
    let project = project_with_imported_drafts();

    assert_eq!(
        project.stdout("", &["drafts", "--imported"]),
        "## old/report one.docx\nLES-003  lesson  Rig warm-up  by anna via anna-claude  3 days\nLES-002  lesson  Wall function shift  by anna via anna-claude  2 days\n\n## old/report two.docx\nLES-001  lesson  Coarse mesh margin  by ben via ben-codex  1 day\n3 drafts\n"
    );
}

#[test]
fn without_imported_an_imported_draft_is_marked_and_an_own_draft_is_not() {
    let project = project_with_imported_drafts();

    assert_eq!(
        project.stdout("", &["drafts"]),
        "LES-003  lesson  Rig warm-up  by anna via anna-claude  3 days  [imported]\nLES-002  lesson  Wall function shift  by anna via anna-claude  2 days  [imported]\nLES-004  lesson  Own lesson  by ben via ben-codex  1 day\nLES-001  lesson  Coarse mesh margin  by ben via ben-codex  1 day  [imported]\n4 drafts\n"
    );
}

#[test]
fn imported_with_mine_shows_only_the_imported_drafts_of_the_calling_person() {
    let project = project_with_imported_drafts();

    assert_eq!(
        project.stdout("anna", &["drafts", "--imported", "--mine"]),
        "## old/report one.docx\nLES-003  lesson  Rig warm-up  by anna via anna-claude  3 days\nLES-002  lesson  Wall function shift  by anna via anna-claude  2 days\n2 drafts\n"
    );
}

#[test]
fn imported_json_lists_the_imported_drafts_in_the_grouped_order() {
    let project = project_with_imported_drafts();

    let stdout = project.stdout("", &["drafts", "--imported", "--json"]);
    let value: Value = serde_json::from_str(&stdout).expect("the JSON parses");
    let ids: Vec<&str> = value["data"]
        .as_array()
        .expect("the data is a list")
        .iter()
        .map(|row| row["id"].as_str().expect("each row has an ID"))
        .collect();

    assert_eq!(ids, ["LES-003", "LES-002", "LES-001"]);
}

#[test]
fn imported_with_no_imported_drafts_says_zero_drafts() {
    let project = Project::new();
    project
        .run_at(
            "ben-codex",
            "2026-11-18T12:00:00+01:00",
            &learn(OWN_STATEMENT, OWN_TITLE, &[]),
        )
        .success();

    assert_eq!(project.stdout("", &["drafts", "--imported"]), "0 drafts\n");
}
