//! Runs the built program for `decide` with related decisions: an LLM's decision that stands next
//! to a related one is refused until it says `--replaces <ID>` or `--besides`; `--replaces`
//! supersedes the old decision in the same step; a person is only warned. Each test runs in its
//! own temporary project folder (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

/// The first decision of every test (invented text), and a second one that shares words with it.
const FIRST: &str = "Use the transient runs for the speedline.";
const SECOND: &str = "Use steady runs for the speedline.";

/// The text of the decision file with `id`.
fn decision_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the decision file")
}

/// The number of files in the decisions folder.
fn decision_count(project: &Project) -> usize {
    std::fs::read_dir(project.root().join("DOSSIER").join("decisions"))
        .expect("the decisions folder")
        .count()
}

/// Records FIRST as anna, a person, so that it is approved and in force.
fn decide_first(project: &Project) {
    project.stdout(
        "anna",
        &[
            "decide",
            FIRST,
            "--reason",
            "Steady runs converged too late.",
        ],
    );
}

#[test]
fn an_llm_decision_next_to_a_related_one_is_refused_and_names_it() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna-claude",
            &[
                "decide",
                SECOND,
                "--reason",
                "Steady runs converged.",
                "--said-by",
                "anna",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("related decisions in force:"))
        .stderr(contains(
            "DEC-001 Use the transient runs for the speedline.",
        ))
        .stderr(contains("state --replaces <ID> or --besides"));
    assert_eq!(decision_count(&project), 1, "nothing was written");

    // The refusal gave no number out: the next decision is DEC-002.
    let output = project.stdout(
        "anna",
        &[
            "decide",
            "Keep cooling holes open.",
            "--reason",
            "Film cooling needs them.",
        ],
    );
    assert_eq!(output, "DEC-002: added (approved)\n");
}

#[test]
fn besides_writes_the_decision_and_warns_about_the_related_one() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna-claude",
            &[
                "decide",
                SECOND,
                "--reason",
                "Steady runs converged.",
                "--said-by",
                "anna",
                "--besides",
            ],
        )
        .success()
        .stdout(contains("DEC-002: added (approved)"))
        .stderr(contains(
            "warning: related: DEC-001 Use the transient runs for the speedline.",
        ));
    assert_eq!(decision_count(&project), 2);
    let text = decision_text(&project, "DEC-002");
    assert!(text.contains("\n- status: approved\n"));
    assert!(!text.contains("- supersedes:"), "nothing was replaced");
}

#[test]
fn replaces_supersedes_the_old_decision_in_the_same_step() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna-claude",
            &[
                "decide",
                SECOND,
                "--reason",
                "Steady runs converged.",
                "--said-by",
                "anna",
                "--replaces",
                "DEC-001",
            ],
        )
        .success()
        .stdout("DEC-002: added (approved)\nDEC-001: superseded by DEC-002\n")
        .stderr(contains("warning:").not());

    let old = decision_text(&project, "DEC-001");
    assert!(old.contains("\n- status: superseded\n"));
    assert!(old.contains("\n- valid-until: 2026-11-20T10:00:00+01:00\n"));
    assert!(old.contains("\n- superseded-by: DEC-002\n"));
    assert!(old.contains(
        "- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: superseded — by DEC-002"
    ));

    let new = decision_text(&project, "DEC-002");
    assert!(new.contains("\n- status: approved\n"));
    assert!(new.contains("\n- author: anna\n"));
    assert!(new.contains("\n- via: anna-claude\n"));
    assert!(new.contains("\n- supersedes: DEC-001\n"));
    assert!(
        project
            .read("DOSSIER/.state/journal.jsonl")
            .contains("\"event\":\"superseded\""),
        "the change to the old decision is in the journal"
    );
}

#[test]
fn a_person_without_flags_is_warned_and_the_decision_is_written() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna",
            &["decide", SECOND, "--reason", "Steady runs converged."],
        )
        .success()
        .stdout(contains("DEC-002: added (approved)"))
        .stderr(contains(
            "warning: related: DEC-001 Use the transient runs for the speedline.",
        ));
    assert_eq!(decision_count(&project), 2);
}

#[test]
fn replaces_and_besides_together_is_a_usage_error() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna-claude",
            &[
                "decide",
                SECOND,
                "--reason",
                "Steady runs converged.",
                "--said-by",
                "anna",
                "--replaces",
                "DEC-001",
                "--besides",
            ],
        )
        .failure()
        .code(2)
        .stderr(contains(
            "--replaces and --besides cannot be given together",
        ));
    assert_eq!(decision_count(&project), 1);
}

#[test]
fn replacing_a_decision_that_is_no_longer_approved_is_refused() {
    let project = Project::new();
    decide_first(&project);
    project.stdout(
        "anna-claude",
        &[
            "decide",
            SECOND,
            "--reason",
            "Steady runs converged.",
            "--said-by",
            "anna",
            "--replaces",
            "DEC-001",
        ],
    );

    project
        .run(
            "anna-claude",
            &[
                "decide",
                "Use the transient runs with a finer tail.",
                "--reason",
                "A finer tail was checked.",
                "--said-by",
                "anna",
                "--replaces",
                "DEC-001",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains(
            "DEC-001 is not an approved decision (status: superseded)",
        ));
    assert_eq!(decision_count(&project), 2);
}

#[test]
fn replacing_a_missing_decision_is_not_found() {
    let project = Project::new();
    decide_first(&project);

    project
        .run(
            "anna",
            &[
                "decide",
                SECOND,
                "--reason",
                "Steady runs converged.",
                "--replaces",
                "DEC-009",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));
    assert_eq!(decision_count(&project), 1);
}
