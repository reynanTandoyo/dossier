//! Runs the built program for `decide`: a decision written at once with its reason, who decided
//! and through which LLM, and the refusals. Each test runs in its own temporary project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use predicates::str::contains;

/// The text of the decision file with `id`.
fn decision_text(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the decision file")
}

/// The history lines of an entry file, in file order.
fn history_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("- history: "))
        .collect()
}

/// Whether the decisions folder holds no file at all.
fn no_decision_written(project: &Project) -> bool {
    let folder = project.root().join("DOSSIER").join("decisions");
    std::fs::read_dir(folder)
        .expect("the decisions folder")
        .next()
        .is_none()
}

#[test]
fn anna_decides_and_the_file_has_the_sections_author_and_two_history_lines() {
    let project = Project::new();
    let output = project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
            "--rejected",
            "Steady runs with a finer mesh: same result, three times the cost.",
        ],
    );
    assert_eq!(output, "DEC-001: added (approved)\n");

    let text = decision_text(&project, "DEC-001");
    assert!(text.starts_with(
        "<!-- dossier-format: 1 -->\n# DEC-001 · Use the transient runs for the last three points.\n"
    ));
    assert!(text.contains("\n- kind: decision\n"));
    assert!(text.contains("\n- status: approved\n"));
    assert!(text.contains("\n- created: 2026-11-20T10:00:00+01:00\n"));
    assert!(text.contains("\n- author: anna\n"));
    assert!(!text.contains("- via:"), "a person's decision has no LLM");
    assert!(text.contains("\n- approved: 2026-11-20T10:00:00+01:00 by anna\n"));
    assert!(text.contains("\n- valid-from: 2026-11-20T10:00:00+01:00\n"));
    assert!(text.contains(
        "\n## Decision\n\nUse the transient runs for the last three points.\n\n## Reason\n\nSteady runs converged where the transient run already showed rotating stall.\n\n## Rejected\n\nSteady runs with a finer mesh: same result, three times the cost.\n"
    ));
    assert_eq!(
        history_lines(&text),
        [
            "- history: 2026-11-20T10:00:00+01:00 anna: created",
            "- history: 2026-11-20T10:00:00+01:00 anna: approved",
        ]
    );

    let journal = project.read("DOSSIER/.state/journal.jsonl");
    assert!(journal.contains("\"command\":\"decide\""));
    assert!(journal.contains("\"id\":\"DEC-001\""));
    assert!(journal.contains("\"time\":\"2026-11-20T10:00:00+01:00\""));
}

#[test]
fn an_llm_decision_without_said_by_is_refused_and_names_said_by() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &[
                "decide",
                "Use the transient runs.",
                "--reason",
                "Steady runs converged.",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("--said-by"));
    assert!(no_decision_written(&project));
}

#[test]
fn an_llm_decision_with_said_by_anna_is_approved_for_anna_via_the_llm() {
    let project = Project::new();
    let output = project.stdout(
        "anna-claude",
        &[
            "decide",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
            "--said-by",
            "anna",
        ],
    );
    assert_eq!(output, "DEC-001: added (approved)\n");

    let text = decision_text(&project, "DEC-001");
    assert!(text.contains("\n- status: approved\n"));
    assert!(text.contains("\n- author: anna\n"));
    assert!(text.contains("\n- via: anna-claude\n"));
    assert!(text.contains("\n- approved: 2026-11-20T10:00:00+01:00 by anna\n"));
    assert_eq!(
        history_lines(&text),
        [
            "- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: created",
            "- history: 2026-11-20T10:00:00+01:00 anna via anna-claude: approved",
        ]
    );
}

#[test]
fn said_by_a_name_that_is_not_a_person_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &[
                "decide",
                "Use the transient runs.",
                "--reason",
                "Steady runs converged.",
                "--said-by",
                "nobody",
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("no person nobody in dossier.toml"));
    assert!(no_decision_written(&project));
}

#[test]
fn a_decision_with_an_empty_reason_is_refused_before_an_id_is_given() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["decide", "Use the transient runs.", "--reason", "  "],
        )
        .failure()
        .code(1)
        .stderr(contains("cannot approve decision: missing section Reason"));
    assert!(no_decision_written(&project));
    assert!(!project.root().join("DOSSIER/.state/ids.json").exists());
}

#[test]
fn the_title_is_the_first_80_characters_of_the_decision_cut_at_a_character() {
    let project = Project::new();
    // 79 letters A-umlaut, then sharp s, then more text: the 80th character is the sharp s.
    let decision = format!("{}ß and more text after the cut", "Ä".repeat(79));
    project.stdout(
        "anna",
        &["decide", &decision, "--reason", "Because the test says so."],
    );

    let text = decision_text(&project, "DEC-001");
    let title = format!("{}ß", "Ä".repeat(79));
    assert!(text.starts_with(&format!(
        "<!-- dossier-format: 1 -->\n# DEC-001 · {title}\n"
    )));
    assert!(text.contains(&format!("\n## Decision\n\n{decision}\n")));
}

#[test]
fn an_explicit_title_and_the_tags_and_task_are_written_as_given() {
    let project = Project::new();
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs.",
            "--reason",
            "Steady runs converged.",
            "--title",
            "Transient tail",
            "--task",
            " t-042 ",
            "--machine",
            " axial compressor ",
            "--analysis",
            "steady RANS, speedline",
            "--software",
            "   ",
            "--tags",
            "speedline,tail",
        ],
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.starts_with("<!-- dossier-format: 1 -->\n# DEC-001 · Transient tail\n"));
    assert!(
        project
            .entry_file("DEC-001")
            .ends_with("DEC-001-transient-tail.md")
    );
    assert!(text.contains("\n- task: T-042\n"));
    assert!(text.contains("\n- machine: axial compressor\n"));
    assert!(text.contains("\n- analysis: steady RANS, speedline\n"));
    assert!(text.contains("\n- tags: speedline,tail\n"));
    assert!(!text.contains("- software:"), "a blank value is left out");
}

/// Writes `deciders = ["anna"]` into dossier.toml, after the `code` line, so that it is a
/// top-level key before the [tags] table (format.md §19).
fn only_anna_decides(project: &Project) {
    let text = project.read("DOSSIER/dossier.toml");
    let mut edited = String::new();
    for line in text.split_inclusive('\n') {
        edited.push_str(line);
        if line.starts_with("code = ") {
            edited.push_str("deciders = [\"anna\"]\n");
        }
    }
    assert!(edited.contains("deciders = [\"anna\"]\n"), "{edited}");
    project.write("DOSSIER/dossier.toml", &edited);
}

#[test]
fn a_decision_by_someone_not_in_deciders_is_a_proposal_and_the_output_says_so() {
    let project = Project::new();
    only_anna_decides(&project);
    let output = project.stdout(
        "ben",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );
    assert_eq!(
        output,
        "DEC-001: proposed (ben is not in deciders; a decider accepts it with: dossier accept DEC-001)\n"
    );

    let text = decision_text(&project, "DEC-001");
    assert!(text.contains("\n- status: proposed\n"));
    assert!(text.contains("\n- author: ben\n"));
    assert!(
        !text.contains("\n- approved:"),
        "a proposal is not approved"
    );
}

#[test]
fn a_proposal_from_decide_gives_the_status_proposed_in_json() {
    let project = Project::new();
    only_anna_decides(&project);
    let stdout = project.stdout(
        "ben",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged.",
            "--json",
        ],
    );
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("one line of JSON");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "DEC-001");
    assert_eq!(value["data"]["status"], "proposed");
}

#[test]
fn a_decider_decides_at_once_and_the_output_says_added_approved() {
    let project = Project::new();
    only_anna_decides(&project);
    let output = project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged.",
        ],
    );
    assert_eq!(output, "DEC-001: added (approved)\n");
}
