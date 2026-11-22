//! Runs the built program for `tool add` (docs/design.md section 12, D31, D37, format.md §22): a
//! tool with only a name and its Purpose is recorded, approved when a person writes it and a draft
//! when an LLM does; a script whose location is a file inside the project, or an absolute path to a
//! file, gets the checksum of that file, stored relative to the project folder when it is inside;
//! a location that is a command name or a URL is kept as typed with no checksum; a bad type, a
//! blank name, a bad review date and a person's tool without a Purpose are refused and write
//! nothing; sections come from a file, from standard input and from `--section`; a team tool is
//! written with `--team` after the confidential check; and `--json` gives the data. `tool update`
//! records the checksum of a script again after a change to its file, takes an approved script
//! back to draft when an LLM makes that change, and refuses a tool that is not a script with a file.
//! Each test runs in its own temporary project folder (common::Project), with the clock fixed at
//! 2026-11-20T10:00:00+01:00.

mod common;

use common::Project;
use dossier::approval::{self, Mark};
use dossier::entry::{Entry, Kind, Status};
use dossier::filesum;
use predicates::str::contains;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The fixed clock of every run (the same as common::Project's).
const NOW: &str = "2026-11-20T10:00:00+01:00";

/// The name most tests give the tool, and the file its entry gets.
const NAME: &str = "Mesh check script";
const FILE: &str = "DOSSIER/tools/TOOL-001-mesh-check-script.md";

/// The Purpose of the tool, as the section text and as the `--section` option.
const PURPOSE: &str = "Checks an invented mesh for negative cell volumes.";
const PURPOSE_ARG: &str = "Purpose=Checks an invented mesh for negative cell volumes.";

/// `tool add` with the name `Mesh check script`, the type `tool_type`, the Purpose section, and
/// `extra` after them.
fn tool_add<'a>(tool_type: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "tool",
        "add",
        "--title",
        NAME,
        "--type",
        tool_type,
        "--section",
        PURPOSE_ARG,
    ];
    args.extend_from_slice(extra);
    args
}

/// The entry file with `id`, read and parsed.
fn entry_of(project: &Project, id: &str) -> Entry {
    let text = fs::read_to_string(project.entry_file(id)).expect("the entry file is read");
    Entry::parse(&text).expect("the entry parses")
}

/// Whether nothing is written to the project's tools folder: the folder is missing or empty.
fn no_tool_written(project: &Project) -> bool {
    let tools = project.root().join("DOSSIER").join("tools");
    !tools.exists()
        || fs::read_dir(&tools)
            .expect("the folder is read")
            .next()
            .is_none()
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

#[test]
fn a_tool_with_only_a_name_and_its_purpose_is_approved_when_a_person_writes_it() {
    let project = Project::new();
    project
        .run("anna", &tool_add("command", &[]))
        .success()
        .stdout("TOOL-001: added (approved)\n");

    assert!(project.root().join(FILE).is_file());
    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.kind(), Some(Kind::Tool));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(entry.title(), NAME);
    assert_eq!(entry.field("type").as_deref(), Some("command"));
    assert_eq!(entry.field("location"), None);
    assert_eq!(entry.field("checksum"), None);
    assert_eq!(entry.section("Purpose").as_deref(), Some(PURPOSE));
    assert_eq!(
        entry.history(),
        [
            format!("{NOW} anna: created"),
            format!("{NOW} anna: approved"),
        ]
    );
    // An approved tool has its checksum recorded (R-LIFE-6).
    assert!(
        project
            .read("DOSSIER/.state/approved.json")
            .contains("TOOL-001")
    );
}

#[test]
fn a_script_inside_the_project_gets_the_checksum_of_its_file() {
    let project = Project::new();
    project.write("scripts/check mesh ü.py", "print('invented')\n");
    project
        .run(
            "anna",
            &tool_add(
                "script",
                &[
                    "--location",
                    "scripts/check mesh ü.py",
                    "--language",
                    "Python",
                    "--software",
                    "Invented Solver 2406",
                ],
            ),
        )
        .success()
        .stdout("TOOL-001: added (approved)\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.field("type").as_deref(), Some("script"));
    assert_eq!(
        entry.field("location").as_deref(),
        Some("scripts/check mesh ü.py")
    );
    assert_eq!(entry.field("language").as_deref(), Some("Python"));
    assert_eq!(
        entry.field("software").as_deref(),
        Some("Invented Solver 2406")
    );
    let file = project.root().join("scripts").join("check mesh ü.py");
    let expected = filesum::checksum(&file).expect("the checksum of the script");
    assert_eq!(entry.field("checksum"), Some(expected));
}

#[test]
fn a_location_typed_from_a_subfolder_is_stored_from_the_project_folder() {
    let project = Project::new();
    project.write("scripts/check.py", "print('invented')\n");
    let subfolder = project.root().join("notes");
    fs::create_dir_all(&subfolder).expect("the subfolder is made");

    project
        .command("anna", NOW)
        .current_dir(&subfolder)
        .args(tool_add("script", &["--location", "../scripts/check.py"]))
        .assert()
        .success()
        .stdout("TOOL-001: added (approved)\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.field("location").as_deref(), Some("scripts/check.py"));
    let file = project.root().join("scripts").join("check.py");
    let expected = filesum::checksum(&file).expect("the checksum of the script");
    assert_eq!(entry.field("checksum"), Some(expected));
}

#[test]
fn an_absolute_path_to_a_script_file_outside_the_project_keeps_its_path_and_gets_its_checksum() {
    let project = Project::new();
    let outside = tempfile::TempDir::new().expect("a temporary folder");
    let file = outside.path().join("outside script.ps1");
    fs::write(&file, "Write-Output 'invented'\n").expect("the script is written");
    let location = file.display().to_string();

    project
        .run(
            "anna",
            &tool_add("script", &["--location", location.as_str()]),
        )
        .success()
        .stdout("TOOL-001: added (approved)\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.field("location").as_deref(), Some(location.as_str()));
    let expected = filesum::checksum(&file).expect("the checksum of the script");
    assert_eq!(entry.field("checksum"), Some(expected));
}

#[test]
fn a_script_location_that_is_not_a_file_is_kept_with_no_checksum_and_a_warning() {
    let project = Project::new();
    project
        .run(
            "anna",
            &tool_add("script", &["--location", "python scripts/missing.py"]),
        )
        .success()
        .stdout("TOOL-001: added (approved)\n")
        .stderr(contains(
            "warning: no checksum: python scripts/missing.py is not a file",
        ));

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(
        entry.field("location").as_deref(),
        Some("python scripts/missing.py")
    );
    assert_eq!(entry.field("checksum"), None);
}

#[test]
fn a_command_name_or_a_url_is_kept_as_typed_with_no_checksum_and_no_warning() {
    let project = Project::new();
    project
        .run("anna", &tool_add("command", &["--location", "ansys-cfx"]))
        .success()
        .stdout("TOOL-001: added (approved)\n")
        .stderr("");
    project
        .run(
            "anna",
            &[
                "tool",
                "add",
                "--title",
                "Team mesh tools",
                "--type",
                "repo",
                "--location",
                "https://example.invalid/team/mesh-tools.git",
                "--section",
                PURPOSE_ARG,
            ],
        )
        .success()
        .stdout("TOOL-002: added (approved)\n")
        .stderr("");

    let command = entry_of(&project, "TOOL-001");
    assert_eq!(command.field("location").as_deref(), Some("ansys-cfx"));
    assert_eq!(command.field("checksum"), None);
    let repo = entry_of(&project, "TOOL-002");
    assert_eq!(
        repo.field("location").as_deref(),
        Some("https://example.invalid/team/mesh-tools.git")
    );
    assert_eq!(repo.field("checksum"), None);
}

#[test]
fn a_tool_an_llm_writes_is_a_draft_with_its_person_as_author_and_the_llm_as_via() {
    let project = Project::new();
    project
        .run(
            "anna-claude",
            &tool_add("software", &["--software", "Invented Solver 2406"]),
        )
        .success()
        .stdout("TOOL-001: added (draft, waiting for approval)\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
    assert_eq!(
        entry.field("software").as_deref(),
        Some("Invented Solver 2406")
    );
    // A draft has no approval yet, so nothing is recorded for its checksum.
    assert_eq!(entry.field("approved"), None);
}

#[test]
fn a_persons_tool_without_its_purpose_is_refused_and_takes_no_number() {
    let project = Project::new();
    project
        .run(
            "anna",
            &["tool", "add", "--title", NAME, "--type", "command"],
        )
        .failure()
        .code(1)
        .stderr(contains(
            "error: cannot approve tool: missing section Purpose",
        ));
    assert!(no_tool_written(&project));

    // The refused tool took no number: the next one is TOOL-001.
    project
        .run("anna", &tool_add("command", &[]))
        .success()
        .stdout("TOOL-001: added (approved)\n");
}

#[test]
fn a_bad_type_or_a_missing_one_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &tool_add("binary", &[]))
        .failure()
        .code(1)
        .stderr(contains(
            "error: --type must be script, command, software or repo, not \"binary\"",
        ));
    project
        .command("anna", NOW)
        .args(["tool", "add", "--title", NAME, "--section", PURPOSE_ARG])
        .assert()
        .failure()
        .code(2);
    assert!(no_tool_written(&project));
}

#[test]
fn a_blank_name_and_a_bad_review_date_are_refused_and_a_good_review_date_is_kept() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "tool",
                "add",
                "--title",
                "   ",
                "--type",
                "command",
                "--section",
                PURPOSE_ARG,
            ],
        )
        .failure()
        .code(1)
        .stderr(contains("error: the title is empty"));
    project
        .run("anna", &tool_add("command", &["--review", "2027-13-01"]))
        .failure()
        .code(1)
        .stderr(contains("error: --review needs a date such as 2027-06-01"));
    assert!(no_tool_written(&project));

    project
        .run("anna", &tool_add("command", &["--review", "2027-06-01"]))
        .success()
        .stdout("TOOL-001: added (approved)\n");
    assert_eq!(
        entry_of(&project, "TOOL-001").field("review").as_deref(),
        Some("2027-06-01")
    );
}

#[test]
fn sections_come_from_a_file_from_standard_input_and_from_section_options_in_the_order_of_the_kind()
{
    let project = Project::new();
    project.write(
        "body.md",
        "Text before the first heading is ignored.\n\n## Inputs\n\nAn invented mesh file.\n\n## Purpose\n\nChecks an invented mesh for negative cell volumes.\n",
    );
    project
        .run(
            "anna",
            &[
                "tool",
                "add",
                "--title",
                NAME,
                "--type",
                "command",
                "--from",
                "body.md",
                "--section",
                "Traps=Slow on big meshes.",
            ],
        )
        .success()
        .stdout("TOOL-001: added (approved)\n");

    let entry = entry_of(&project, "TOOL-001");
    let names: Vec<String> = entry.sections().into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, ["Purpose", "Inputs", "Traps"]);
    assert_eq!(entry.section("Purpose").as_deref(), Some(PURPOSE));
    assert_eq!(
        entry.section("Inputs").as_deref(),
        Some("An invented mesh file.")
    );
    assert_eq!(
        entry.section("Traps").as_deref(),
        Some("Slow on big meshes.")
    );

    project
        .command("anna", NOW)
        .args([
            "tool",
            "add",
            "--title",
            "Stdin tool",
            "--type",
            "command",
            "--stdin",
        ])
        .write_stdin(format!("## Purpose\n\n{PURPOSE}\n"))
        .assert()
        .success()
        .stdout("TOOL-002: added (approved)\n");
    assert_eq!(
        entry_of(&project, "TOOL-002").section("Purpose").as_deref(),
        Some(PURPOSE)
    );
}

#[test]
fn a_team_tool_is_written_to_the_team_dossier_with_team_after_the_confidential_check() {
    let project = Project::new();
    let (_folder, team) = team_dossier(&project);
    link(&project, &team);
    state_none(&project);

    project
        .run(
            "anna",
            &[
                "--team",
                "tool",
                "add",
                "--title",
                NAME,
                "--type",
                "repo",
                "--location",
                "https://example.invalid/team/mesh-tools.git",
                "--section",
                PURPOSE_ARG,
            ],
        )
        .success()
        .stdout("TOOL-001: added (approved)\n");

    assert!(
        team.join("tools")
            .join("TOOL-001-mesh-check-script.md")
            .is_file()
    );
    assert!(no_tool_written(&project));
}

#[test]
fn json_gives_the_id_status_type_and_location_of_the_tool() {
    let project = Project::new();
    let output = project
        .command("anna", NOW)
        .args(["--json"])
        .args(tool_add("command", &["--location", "ansys-cfx"]))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("the JSON result parses");
    assert_eq!(value["schema"], "dossier.tool.v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["id"], "TOOL-001");
    assert_eq!(value["data"]["status"], "approved");
    assert_eq!(value["data"]["type"], "command");
    assert_eq!(value["data"]["location"], "ansys-cfx");
    assert_eq!(value["data"]["checksum"], Value::Null);
    assert_eq!(value["warnings"], serde_json::json!([]));
}

/// The script the update tests use: a file inside the project folder, with a space and an umlaut in
/// its name, as the `add` tests use.
const SCRIPT: &str = "scripts/check mesh ü.py";

/// `tool add` of `SCRIPT` by anna, after the file is written with `text`: the tool TOOL-001,
/// approved.
fn anna_adds_script(project: &Project, text: &str) {
    project.write(SCRIPT, text);
    project
        .run(
            "anna",
            &tool_add("script", &["--location", SCRIPT, "--language", "Python"]),
        )
        .success()
        .stdout("TOOL-001: added (approved)\n");
}

/// The checksum of `SCRIPT` as its file is now.
fn script_checksum(project: &Project) -> String {
    filesum::checksum(&project.root().join(SCRIPT)).expect("the checksum of the script")
}

/// The state folder of the project, where the approval checksums are.
fn state_of(project: &Project) -> PathBuf {
    project.root().join("DOSSIER").join(".state")
}

#[test]
fn updating_a_script_after_its_change_records_the_new_checksum_on_purpose() {
    let project = Project::new();
    anna_adds_script(&project, "print('invented')\n");
    let recorded = entry_of(&project, "TOOL-001")
        .field("checksum")
        .expect("add records the checksum");

    project.write(SCRIPT, "print('changed')\n");
    let new = script_checksum(&project);
    assert_ne!(new, recorded);
    project
        .run_at(
            "anna",
            "2026-11-21T09:00:00+01:00",
            &["tool", "update", "TOOL-001"],
        )
        .success()
        .stdout("TOOL-001: changed\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.field("checksum"), Some(new));
    assert_eq!(entry.field("location").as_deref(), Some(SCRIPT));
    assert_eq!(entry.status(), Some(Status::Approved));
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-21T09:00:00+01:00 anna: changed — checksum")
    );
    let last = project
        .entry_journal()
        .pop()
        .expect("the update is in the journal");
    assert_eq!(
        (last.id.as_str(), last.event.as_str(), last.detail.as_str()),
        ("TOOL-001", "changed", "checksum")
    );
}

#[test]
fn an_llm_updating_an_approved_script_takes_it_back_to_draft() {
    let project = Project::new();
    anna_adds_script(&project, "print('invented')\n");
    project.write(SCRIPT, "print('changed')\n");
    let new = script_checksum(&project);

    project
        .run("anna-claude", &["tool", "update", "TOOL-001"])
        .success()
        .stdout("TOOL-001: changed (back to draft, waiting for approval)\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.status(), Some(Status::Draft));
    assert_eq!(entry.field("approved"), None);
    assert_eq!(entry.field("valid-from"), None);
    assert_eq!(entry.field("checksum"), Some(new));
    // The tool was written by anna, so there is no via field; the history line names the LLM.
    assert_eq!(entry.field("author").as_deref(), Some("anna"));
    assert_eq!(entry.field("via"), None);
    assert_eq!(
        entry.history().last().map(String::as_str),
        Some("2026-11-20T10:00:00+01:00 anna via anna-claude: changed — checksum, back to draft")
    );
    // The approval of the old text is forgotten, so the draft has no recorded checksum.
    let text = project.read(FILE);
    assert!(matches!(
        approval::mark(&state_of(&project), "TOOL-001", text.as_bytes()),
        Mark::NotRecorded
    ));
}

#[test]
fn a_persons_update_of_an_approved_script_keeps_it_approved_and_records_its_approval_again() {
    let project = Project::new();
    anna_adds_script(&project, "print('invented')\n");
    project.write(SCRIPT, "print('changed')\n");

    project
        .run("anna", &["tool", "update", "TOOL-001"])
        .success()
        .stdout("TOOL-001: changed\n");

    assert_eq!(
        entry_of(&project, "TOOL-001").status(),
        Some(Status::Approved)
    );
    let text = project.read(FILE);
    assert!(matches!(
        approval::mark(&state_of(&project), "TOOL-001", text.as_bytes()),
        Mark::Unchanged
    ));
}

#[test]
fn updating_a_script_whose_checksum_is_recorded_already_changes_nothing() {
    let project = Project::new();
    anna_adds_script(&project, "print('invented')\n");
    let before = project.read(FILE);

    project
        .run("anna-claude", &["tool", "update", "TOOL-001"])
        .success()
        .stdout("TOOL-001: no change\n");

    assert_eq!(project.read(FILE), before);
    assert_eq!(
        entry_of(&project, "TOOL-001").status(),
        Some(Status::Approved)
    );
}

#[test]
fn an_absolute_script_location_is_read_where_it_is() {
    let project = Project::new();
    let outside = tempfile::TempDir::new().expect("a temporary folder");
    let file = outside.path().join("outside script.ps1");
    fs::write(&file, "Write-Output 'invented'\n").expect("the script is written");
    let location = file.display().to_string();
    project
        .run(
            "anna",
            &tool_add("script", &["--location", location.as_str()]),
        )
        .success();

    fs::write(&file, "Write-Output 'changed'\n").expect("the script is changed");
    project
        .run("anna", &["tool", "update", "TOOL-001"])
        .success()
        .stdout("TOOL-001: changed\n");

    let entry = entry_of(&project, "TOOL-001");
    assert_eq!(entry.field("location").as_deref(), Some(location.as_str()));
    let expected = filesum::checksum(&file).expect("the checksum of the script");
    assert_eq!(entry.field("checksum"), Some(expected));
}

#[test]
fn a_tool_that_is_not_a_script_is_refused_and_nothing_is_written() {
    let project = Project::new();
    project
        .run("anna", &tool_add("command", &["--location", "ansys-cfx"]))
        .success();
    let before = project.read(FILE);

    project
        .run("anna", &["tool", "update", "TOOL-001"])
        .failure()
        .code(1)
        .stderr(contains(
            "error: TOOL-001 is not a script; only a script has the checksum of its file",
        ));
    assert_eq!(project.read(FILE), before);
}

#[test]
fn a_script_whose_file_is_gone_is_refused_and_nothing_is_written() {
    let project = Project::new();
    anna_adds_script(&project, "print('invented')\n");
    let before = project.read(FILE);
    fs::remove_file(project.root().join(SCRIPT)).expect("the script is removed");

    project
        .run("anna", &["tool", "update", "TOOL-001"])
        .failure()
        .code(1)
        .stderr(contains(
            "error: the script file is not there: scripts/check mesh ü.py",
        ));
    assert_eq!(project.read(FILE), before);
}

#[test]
fn an_entry_that_is_not_a_tool_and_an_unknown_id_are_refused() {
    let project = Project::new();
    project
        .run(
            "anna",
            &[
                "learn",
                "Invented statement for the update test.",
                "--valid-for",
                "invented case",
                "--evidence",
                "invented evidence",
                "--confidence",
                "simulated",
            ],
        )
        .success();

    project
        .run("anna", &["tool", "update", "LES-001"])
        .failure()
        .code(1)
        .stderr(contains(
            "error: LES-001 is not a tool; only a tool has a checksum to record again",
        ));
    project
        .run("anna", &["tool", "update", "TOOL-009"])
        .failure()
        .code(1)
        .stderr(contains("error: no entry TOOL-009"));
}
