//! Runs the built program for the similar-entry hint of `learn` and for `learn --update`: a new
//! lesson names the lessons most like it, `--update` changes one in place, and an LLM's change to
//! an approved lesson takes it back to draft. Each test runs in its own project folder, with the
//! clock fixed by `common::Project`.

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Status};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

/// The fixed clock of `common::Project`.
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// Invented text for the parts of a lesson.
const STATEMENT: &str = "Steady runs with a mixing-plane interface overpredict the stall margin.";
const VALID_FOR: &str = "An invented test rig with a tip Mach number above 1.1.";
const EVIDENCE: &str = "An invented comparison of two speedlines, section 1.";

/// Invented titles. The second shares several words with the first; the third shares two.
const FIRST_TITLE: &str = "Steady runs overpredict the stall margin";
const SIMILAR_TITLE: &str = "Stall margin overpredicted by steady runs";
const THIRD_TITLE: &str = "Steady runs and mesh coarsening";

/// The file of the first lesson, which is named after its title.
const FIRST_FILE: &str = "LES-001-steady-runs-overpredict-the-stall-margin.md";

/// `learn` with a complete lesson: its statement, the other parts, then `extra`.
fn learn<'a>(statement: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "learn",
        statement,
        "--valid-for",
        VALID_FOR,
        "--evidence",
        EVIDENCE,
    ];
    args.extend_from_slice(extra);
    args
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    Entry::parse(&text_of(project, id)).expect("the entry parses")
}

/// The text of the entry file with `id`.
fn text_of(project: &Project, id: &str) -> String {
    std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read")
}

/// The names of the files in the lessons folder, in name order.
fn lesson_files(project: &Project) -> Vec<String> {
    let folder = project.root().join("DOSSIER").join("lessons");
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .expect("the lessons folder is read")
        .map(|entry| {
            entry
                .expect("a folder entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn a_similar_lesson_is_named_as_a_warning_with_its_id_and_title() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success()
        .stdout("LES-001: added (approved)\n");
    project
        .run(
            "anna",
            &learn(
                "Stall margin is overpredicted by steady runs at the mixing plane.",
                &["--title", SIMILAR_TITLE, "--confidence", "measured"],
            ),
        )
        .success()
        .stdout("LES-002: added (approved)\n")
        .stderr(contains(format!(
            "warning: similar: LES-001 {FIRST_TITLE}\n"
        )));
}

#[test]
fn json_output_carries_the_similar_lessons_as_warnings() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();
    let out = project.stdout(
        "anna",
        &learn(
            "Stall margin is overpredicted by steady runs.",
            &[
                "--title",
                SIMILAR_TITLE,
                "--confidence",
                "measured",
                "--json",
            ],
        ),
    );
    let value: serde_json::Value = serde_json::from_str(out.trim()).expect("one JSON line");
    assert_eq!(
        value["data"],
        serde_json::json!({"id": "LES-002", "status": "approved"})
    );
    assert_eq!(
        value["warnings"],
        serde_json::json!([format!("similar: LES-001 {FIRST_TITLE}")])
    );
}

#[test]
fn a_similar_draft_and_a_rejected_lesson_are_marked_with_their_status() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "guess"],
            ),
        )
        .success()
        .stdout("LES-001: added (draft, waiting for approval)\n");
    project
        .run(
            "anna",
            &learn(
                "Stall margin is overpredicted by steady runs.",
                &["--title", SIMILAR_TITLE, "--confidence", "measured"],
            ),
        )
        .success()
        .stderr(contains(format!(
            "warning: similar: LES-001 {FIRST_TITLE} [draft]\n"
        )));

    project
        .run("anna", &["reject", "LES-001"])
        .success()
        .stdout("LES-001: rejected\n");
    // LES-002 is approved, so it is named without a mark; LES-001 is rejected.
    project
        .run(
            "anna-claude",
            &learn(
                "Mixing-plane runs stall early in steady mode.",
                &["--title", THIRD_TITLE, "--confidence", "guess"],
            ),
        )
        .success()
        .stderr(contains(format!(
            "warning: similar: LES-001 {FIRST_TITLE} [rejected]\n"
        )))
        .stderr(contains(format!(
            "warning: similar: LES-002 {SIMILAR_TITLE}\n"
        )));
}

#[test]
fn a_superseded_lesson_is_not_named_as_similar() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();
    project
        .run(
            "anna",
            &learn(
                "Mesh coarsening changes the stall margin little.",
                &[
                    "--title",
                    "Mesh coarsening and the stall margin",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success()
        .stdout("LES-002: added (approved)\n");
    project
        .run("anna", &["supersede", "LES-001", "--by", "LES-002"])
        .success()
        .stdout("LES-001: superseded by LES-002\n");

    // LES-001 matches this title best of all, so it would be named if it were still in use.
    project
        .run(
            "anna",
            &learn(
                "Steady runs need a finer mesh near the stall margin.",
                &["--title", SIMILAR_TITLE, "--confidence", "measured"],
            ),
        )
        .success()
        .stderr(contains("similar: LES-001").not());
}

#[test]
fn a_lesson_with_the_same_tag_is_named_even_when_its_words_differ() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                "Tip clearance changes the surge line.",
                &[
                    "--title",
                    "Tip clearance changes the surge line",
                    "--confidence",
                    "measured",
                    "--machine",
                    "axial compressor",
                ],
            ),
        )
        .success();
    project
        .run(
            "anna",
            &learn(
                "Hub seal leaks slowly.",
                &[
                    "--title",
                    "Hub seal leakage",
                    "--confidence",
                    "measured",
                    "--machine",
                    "Axial Compressor, fan",
                ],
            ),
        )
        .success()
        .stderr(contains(
            "warning: similar: LES-001 Tip clearance changes the surge line\n",
        ));
}

#[test]
fn common_english_words_alone_do_not_make_lessons_similar() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                "Warm the cache first.",
                &[
                    "--title",
                    "The cache must be warm before the run",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success();
    // Its title shares only "the" with the first lesson's title; a common word alone is no match.
    project
        .run(
            "anna",
            &learn(
                "A coarse mesh overpredicts.",
                &["--title", "The mesh is coarse", "--confidence", "measured"],
            ),
        )
        .success()
        .stderr(contains("similar").not());
}

#[test]
fn a_lesson_with_other_forms_of_the_same_words_is_named_as_similar() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &[
                    "--title",
                    "Mixing planes overpredict the stall margin",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success()
        .stdout("LES-001: added (approved)\n");
    // Its words are other forms of the first title's words: margins, overpredicted, mixing, plane.
    project
        .run(
            "anna",
            &learn(
                "Stall margins are overpredicted by a mixing plane.",
                &[
                    "--title",
                    "Stall margins are overpredicted by a mixing plane",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success()
        .stdout("LES-002: added (approved)\n")
        .stderr(contains(
            "warning: similar: LES-001 Mixing planes overpredict the stall margin\n",
        ));
}

#[test]
fn a_lesson_sharing_only_common_words_is_not_named_as_similar() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                "Nozzle and duct sizing depends on the flow.",
                &[
                    "--title",
                    "Nozzles and ducts are sized",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success()
        .stdout("LES-001: added (approved)\n");
    // The two lessons share only "the", "and" and "are".
    project
        .run(
            "anna",
            &learn(
                "Balance the rotor and the fan after trimming.",
                &[
                    "--title",
                    "Fans and rotors are balanced",
                    "--confidence",
                    "measured",
                ],
            ),
        )
        .success()
        .stdout("LES-002: added (approved)\n")
        .stderr(contains("similar").not());
}

#[test]
fn update_changes_the_lesson_in_place_and_creates_no_file() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();

    let new_statement = "Steady runs overpredict the stall margin on a finer mesh.";
    project
        .run(
            "anna",
            &[
                "learn",
                new_statement,
                "--valid-for",
                "A changed test rig with a tip Mach number above 1.2.",
                "--evidence",
                "A changed comparison of two speedlines.",
                "--confidence",
                "experience",
                "--update",
                "LES-001",
                "--machine",
                "axial compressor",
            ],
        )
        .success()
        .stdout("LES-001: updated\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.section("Statement").as_deref(), Some(new_statement));
    assert_eq!(
        entry.section("Valid for").as_deref(),
        Some("A changed test rig with a tip Mach number above 1.2.")
    );
    assert_eq!(
        entry.section("Evidence").as_deref(),
        Some("A changed comparison of two speedlines.")
    );
    assert_eq!(entry.field("confidence").as_deref(), Some("experience"));
    assert_eq!(entry.field("machine").as_deref(), Some("axial compressor"));
    assert_eq!(entry.title(), FIRST_TITLE);
    // A person's change keeps the lesson approved, with its approval.
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.field("approved").as_deref(),
        Some(format!("{NOW} by anna").as_str())
    );
    let history = entry.history();
    assert_eq!(history.len(), 3, "{history:?}");
    assert_eq!(history[2], format!("{NOW} anna: changed — updated"));

    // No new file: the one lesson is still the only file in the lessons folder.
    assert_eq!(lesson_files(&project), vec![FIRST_FILE.to_string()]);
    // The approval is recorded again for the new text.
    let state = project.root().join("DOSSIER").join(".state");
    assert!(matches!(
        approval::mark(&state, "LES-001", text_of(&project, "LES-001").as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn an_llms_update_of_an_approved_lesson_makes_it_a_draft_again() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();
    project
        .run(
            "anna-claude",
            &learn(
                "A statement the LLM corrected.",
                &["--update", "LES-001", "--confidence", "simulated"],
            ),
        )
        .success()
        .stdout("LES-001: updated (back to draft, waiting for approval)\n");

    let entry = entry_of(&project, "LES-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("confidence").as_deref(), Some("simulated"));
    assert_eq!(
        entry.section("Statement").as_deref(),
        Some("A statement the LLM corrected.")
    );
    let history = entry.history();
    assert_eq!(
        history[2],
        format!("{NOW} anna via anna-claude: changed — updated, back to draft")
    );
    // The draft has no approved checksum any more.
    assert!(
        !project
            .read("DOSSIER/.state/approved.json")
            .contains("LES-001")
    );
}

#[test]
fn a_persons_update_that_leaves_an_approved_lesson_incomplete_is_refused() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();
    let before = text_of(&project, "LES-001");

    project
        .run(
            "anna",
            &[
                "learn",
                STATEMENT,
                "--valid-for",
                "",
                "--evidence",
                EVIDENCE,
                "--confidence",
                "measured",
                "--update",
                "LES-001",
            ],
        )
        .code(1)
        .stderr(contains(
            "error: cannot approve lesson: missing section Valid for",
        ));
    assert_eq!(text_of(&project, "LES-001"), before);
}

#[test]
fn a_dry_run_of_an_update_says_what_it_would_write_and_changes_nothing() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--title", FIRST_TITLE, "--confidence", "measured"],
            ),
        )
        .success();
    let before = text_of(&project, "LES-001");

    project
        .run(
            "anna",
            &learn(
                "A changed statement for the dry run.",
                &[
                    "--update",
                    "LES-001",
                    "--confidence",
                    "measured",
                    "--dry-run",
                ],
            ),
        )
        .success()
        .stdout("LES-001: updated\n")
        .stderr(contains("warning: dry run: would write"));
    assert_eq!(text_of(&project, "LES-001"), before);
}

#[test]
fn update_names_only_a_lesson() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "decide",
                "Use the transient runs for the stall margin.",
                "--reason",
                "An invented reason for a test.",
            ],
        )
        .success()
        .stdout("DEC-001: added (approved)\n");
    let before = text_of(&project, "DEC-001");

    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--update", "DEC-001", "--confidence", "measured"],
            ),
        )
        .code(1)
        .stderr(contains(
            "error: DEC-001 is not a lesson; --update changes a lesson only",
        ));
    assert_eq!(text_of(&project, "DEC-001"), before);
    assert!(lesson_files(&project).is_empty());
}

#[test]
fn update_of_an_unknown_id_is_not_found() {
    let project = Project::new();
    project
        .run(
            "anna",
            &learn(
                STATEMENT,
                &["--update", "LES-404", "--confidence", "measured"],
            ),
        )
        .code(1)
        .stderr(contains("no entry"));
    assert!(lesson_files(&project).is_empty());
}
