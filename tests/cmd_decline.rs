//! Runs the built program for `decline`: a decider at a terminal turns a proposal down. The
//! declined proposal is kept with status `declined` and its reason, and it is hidden from `list`
//! except with `--status declined`. Only a person named in `deciders` may decline, only at a
//! terminal, and only a proposal can be declined. Each test runs in its own project folder
//! (common::Project), with the clock fixed at 2026-11-20T10:00:00+01:00 unless a test says
//! otherwise.

mod common;

use common::Project;
use dossier::entry::{Entry, Status};
use predicates::str::contains;

/// The fixed clock of `common::Project`: when the proposals are made.
const NOW: &str = "2026-11-20T10:00:00+01:00";
/// The time of the declining in the tests that decline, a day later.
const LATER: &str = "2026-11-21T09:30:00+01:00";
/// Invented text: why the proposal is turned down.
const REASON: &str = "Invented reason: the steady runs were not checked against the inlet profile.";

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

/// Ben proposes a decision with `propose`: a proposal, DEC-001. Returns its ID.
fn ben_proposes(project: &Project) -> &'static str {
    project.stdout(
        "ben",
        &[
            "propose",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged where the transient run already showed rotating stall.",
        ],
    );
    "DEC-001"
}

/// Runs `decline` as `agent` at `now` with `reason`, at a terminal (DOSSIER_ANSWERS is set; the
/// command asks nothing, the variable only makes the run a terminal).
fn decline_with(
    project: &Project,
    agent: &str,
    now: &str,
    id: &str,
    reason: &str,
) -> assert_cmd::assert::Assert {
    project
        .command(agent, now)
        .env("DOSSIER_ANSWERS", "")
        .args(["decline", id, "--reason", reason])
        .assert()
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = std::fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// The bytes of the entry file with `id`.
fn bytes_of(project: &Project, id: &str) -> Vec<u8> {
    std::fs::read(project.entry_file(id)).expect("the entry file is read")
}

#[test]
fn a_decider_declines_a_proposal_with_its_reason_and_it_is_kept() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));

    decline_with(&project, "anna", LATER, id, REASON)
        .success()
        .stdout("DEC-001: declined\n");

    let entry = entry_of(&project, id);
    assert_eq!(entry.status(), Some(Status::Declined));
    assert_eq!(entry.field("declined-reason").as_deref(), Some(REASON));
    assert_eq!(
        entry.field("approved"),
        None,
        "a declined proposal is not approved"
    );
    assert_eq!(
        entry.field("valid-from"),
        None,
        "a declined proposal is not in force"
    );
    // The proposal keeps its author and its creation time; the declining is a history line.
    assert_eq!(entry.field("author").as_deref(), Some("ben"));
    assert_eq!(entry.field("created").as_deref(), Some(NOW));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} ben: created"),
            format!("{NOW} ben: proposed"),
            format!("{LATER} anna: declined — {REASON}"),
        ]
    );

    // The journal has the declining, by the person, with the command that did it.
    let journal = project.entry_journal();
    let last = journal.last().expect("a journal line");
    assert_eq!(last.command, "decline");
    assert_eq!(last.id, "DEC-001");
    assert_eq!(last.event, "declined");
    assert_eq!(last.detail, REASON);
    assert_eq!(last.person, "anna");
    assert!(!last.llm);
}

#[test]
fn a_declined_proposal_is_hidden_from_list_and_shown_by_status_declined() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    decline_with(&project, "anna", LATER, id, REASON)
        .success()
        .stdout("DEC-001: declined\n");

    let list = project.stdout("anna", &["list"]);
    assert!(!list.contains("DEC-001"), "list shows: {list}");

    let declined = project.stdout("anna", &["list", "--status", "declined"]);
    assert!(declined.contains("DEC-001"), "list shows: {declined}");
    assert!(declined.contains("[declined]"), "list shows: {declined}");
}

#[test]
fn a_declined_proposal_cannot_be_accepted_or_declined_again() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    decline_with(&project, "anna", LATER, id, REASON)
        .success()
        .stdout("DEC-001: declined\n");
    let before = bytes_of(&project, id);

    project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "yes")
        .args(["accept", id])
        .assert()
        .failure()
        .code(1)
        .stderr(contains(
            "only a proposal can be accepted; DEC-001 is declined",
        ));

    decline_with(&project, "anna", LATER, id, "Another reason.")
        .failure()
        .code(1)
        .stderr(contains(
            "only a proposal can be declined; DEC-001 is declined",
        ));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
}

#[test]
fn ben_is_not_a_decider_so_he_is_refused_and_the_proposal_stays() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    let before = bytes_of(&project, id);

    decline_with(&project, "ben", LATER, id, REASON)
        .failure()
        .code(1)
        .stderr(contains("ben is not in deciders"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn anna_claude_is_an_llm_so_it_is_refused_even_though_its_person_decides() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    let before = bytes_of(&project, id);

    decline_with(&project, "anna-claude", LATER, id, REASON)
        .failure()
        .code(1)
        .stderr(contains("decline is for a person; anna-claude is an LLM"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn without_a_terminal_nothing_is_written() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    let before = bytes_of(&project, id);

    project
        .command("anna", LATER)
        .env_remove("DOSSIER_ANSWERS")
        .args(["decline", id, "--reason", REASON])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("must be run by a person at a terminal"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
}

#[test]
fn an_approved_decision_is_refused_because_only_a_proposal_can_be_declined() {
    let project = Project::new();
    only_anna_decides(&project);
    // Anna is a decider, so her decision is approved at once and is not a proposal.
    project.stdout(
        "anna",
        &[
            "decide",
            "Use the transient runs for the last three points.",
            "--reason",
            "Steady runs converged.",
        ],
    );
    let before = bytes_of(&project, "DEC-001");

    decline_with(&project, "anna", LATER, "DEC-001", REASON)
        .failure()
        .code(1)
        .stderr(contains(
            "only a proposal can be declined; DEC-001 is approved",
        ));

    assert_eq!(
        bytes_of(&project, "DEC-001"),
        before,
        "the file is unchanged"
    );
}

#[test]
fn a_reason_that_is_blank_is_refused_and_nothing_is_written() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    let before = bytes_of(&project, id);

    decline_with(&project, "anna", LATER, id, "   ")
        .failure()
        .code(1)
        .stderr(contains("the reason must not be empty"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
    assert_eq!(entry_of(&project, id).status(), Some(Status::Proposed));
}

#[test]
fn the_reason_is_required() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);
    let before = bytes_of(&project, id);

    project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "")
        .args(["decline", id])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("--reason"));

    assert_eq!(bytes_of(&project, id), before, "the file is unchanged");
}

#[test]
fn an_unknown_id_is_not_found() {
    let project = Project::new();
    only_anna_decides(&project);
    decline_with(&project, "anna", LATER, "DEC-009", REASON)
        .failure()
        .code(1)
        .stderr(contains("no entry DEC-009"));
}

#[test]
fn with_no_deciders_named_anyone_may_decline_a_proposal() {
    let project = Project::new();
    let id = ben_proposes(&project);

    decline_with(&project, "ben", LATER, id, REASON)
        .success()
        .stdout("DEC-001: declined\n");

    let entry = entry_of(&project, id);
    assert_eq!(entry.status(), Some(Status::Declined));
    assert_eq!(entry.field("declined-reason").as_deref(), Some(REASON));
}

#[test]
fn json_gives_the_id_and_the_status_and_a_refusal_names_its_code() {
    let project = Project::new();
    only_anna_decides(&project);
    let id = ben_proposes(&project);

    let declined = project
        .command("anna", LATER)
        .env("DOSSIER_ANSWERS", "")
        .args(["decline", id, "--reason", REASON, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&declined).expect("one line of JSON");
    assert_eq!(value["schema"], "dossier.decline.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "DEC-001");
    assert_eq!(value["data"]["status"], "declined");

    project
        .command("ben", LATER)
        .env("DOSSIER_ANSWERS", "")
        .args(["decline", "DEC-001", "--reason", REASON, "--json"])
        .assert()
        .failure()
        .code(1)
        .stdout(contains("\"code\":\"REFUSED\""));
}
