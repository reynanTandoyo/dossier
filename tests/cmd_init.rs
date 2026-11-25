//! Runs the built program for `init`: the folders and the settings of a new project dossier, the
//! journal line that records who created it, the refusals, `--with-team`, which links the project
//! to its team dossier and registers its code there, `--past` with `--in`, which make the dossier of
//! a finished project, and the first member of a new team dossier. Each test runs in its own
//! temporary folder.

mod common;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::path::{Path, PathBuf};

/// The fixed clock of every test in this file.
const NOW: &str = "2026-11-21T09:00:00+01:00";

/// The fixed clock of the `--with-team` tests: the clock of `common::Project`.
const TEAM_NOW: &str = "2026-11-20T10:00:00+01:00";

/// `init` with `args` run in `folder` with `vars` set, not yet checked. The agent names are
/// removed from the environment, and the clock is fixed.
fn init_with(folder: &Path, vars: &[(&str, &str)], args: &[&str]) -> assert_cmd::assert::Assert {
    let mut command = Command::cargo_bin("dossier").unwrap();
    command
        .current_dir(folder)
        .env("DOSSIER_NOW", NOW)
        .env_remove("DOSSIER_AGENT")
        .env_remove("TODO_AGENT");
    for (key, value) in vars {
        command.env(key, value);
    }
    command.arg("init").args(args).assert()
}

/// `init` with `args` run in `folder`, not yet checked. See `init_with`.
fn init(folder: &Path, args: &[&str]) -> assert_cmd::assert::Assert {
    init_with(folder, &[], args)
}

/// The text of dossier.toml in the dossier of `folder`.
fn settings(folder: &Path) -> String {
    std::fs::read_to_string(folder.join("DOSSIER").join("dossier.toml")).unwrap()
}

/// The journal records of the dossier of `folder`, oldest first.
fn journal_of(folder: &Path) -> Vec<dossier::journal::Record> {
    dossier::journal::read(&folder.join("DOSSIER").join(".state")).unwrap()
}

#[test]
fn init_creates_the_folders_and_a_settings_file_with_the_code() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("created DOSSIER for P-001"))
        .stdout(contains("next: dossier people add <person> --llm <name>"));
    let dossier = folder.path().join("DOSSIER");
    for name in [
        "decisions",
        "assumptions",
        "findings",
        "lessons",
        "usage",
        ".state",
    ] {
        assert!(dossier.join(name).is_dir(), "{name} is missing");
    }
    assert!(settings(folder.path()).contains("code = \"P-001\""));
    let cfg = dossier::config::load(&dossier).unwrap();
    assert_eq!(cfg.code, "P-001");
    assert_eq!(cfg.format, 1);
}

#[test]
fn a_second_init_is_refused_and_changes_nothing() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"]).success();
    let before = settings(folder.path());
    init(folder.path(), &["--code", "P-002"])
        .code(1)
        .stderr(contains("error: DOSSIER already exists here"));
    assert_eq!(settings(folder.path()), before);
    assert_eq!(journal_of(folder.path()).len(), 1);
}

#[test]
fn a_file_named_dossier_is_refused_too() {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("DOSSIER"), "not a folder").unwrap();
    init(folder.path(), &["--code", "P-001"])
        .code(1)
        .stderr(contains("error: DOSSIER already exists here"));
    assert!(folder.path().join("DOSSIER").is_file());
}

#[test]
fn a_code_with_a_space_is_refused_and_nothing_is_created() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P 001"])
        .code(1)
        .stderr(contains(
            "error: a project code has only letters, digits and -, such as P-017",
        ));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn an_empty_code_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", ""])
        .code(1)
        .stderr(contains("a project code has only letters, digits and -"));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn a_code_with_a_non_ascii_letter_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-Ü01"]).code(1);
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn letters_and_digits_in_either_case_with_hyphens_are_accepted() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "abc-17-X"]).success();
    assert!(settings(folder.path()).contains("code = \"abc-17-X\""));
}

#[test]
fn init_without_a_code_is_a_usage_error() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &[]).code(2);
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn init_works_in_a_folder_with_spaces_and_non_ascii_characters() {
    let folder = tempfile::tempdir().unwrap();
    let project = folder.path().join("Projekt Übersicht");
    std::fs::create_dir(&project).unwrap();
    init(&project, &["--code", "P-001"]).success();
    assert!(project.join("DOSSIER").join("dossier.toml").is_file());
}

#[test]
fn init_as_json_names_the_code_and_the_dossier() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--json"])
        .success()
        .stdout(contains("\"schema\":\"dossier.init.v1\""))
        .stdout(contains("\"ok\":true"))
        .stdout(contains("\"code\":\"P-001\""));
}

#[test]
fn a_refused_init_as_json_gives_the_error_name() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P 001", "--json"])
        .code(1)
        .stdout(contains("\"ok\":false"))
        .stdout(contains("\"code\":\"BAD_VALUE\""));
}

#[test]
fn init_appends_one_journal_line_with_the_agent_and_the_time() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("DOSSIER_AGENT", "anna")],
        &["--code", "P-001"],
    )
    .success();
    let records = journal_of(folder.path());
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.time, NOW);
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert!(!record.llm);
    assert_eq!(record.command, "init");
    assert_eq!(record.id, "DOSSIER");
    assert_eq!(record.event, "created");
    assert_eq!(record.detail, "code P-001");
}

#[test]
fn the_agent_option_gives_the_name_in_the_journal_before_the_environment() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("DOSSIER_AGENT", "anna")],
        &["--code", "P-001", "--agent", "ben"],
    )
    .success();
    let records = journal_of(folder.path());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "ben");
    assert_eq!(records[0].person, "ben");
}

#[test]
fn without_a_name_the_journal_names_the_operating_system_user() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("USERNAME", "osuser"), ("USER", "osuser")],
        &["--code", "P-001"],
    )
    .success();
    let records = journal_of(folder.path());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "osuser");
    assert_eq!(records[0].person, "osuser");
    assert_eq!(records[0].event, "created");
}

#[test]
fn init_creates_the_usage_folder() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"]).success();
    assert!(folder.path().join("DOSSIER").join("usage").is_dir());
}

#[test]
fn code_project_marks_the_settings_and_says_so() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--code-project"])
        .success()
        .stdout(contains("code project: map and code pointers are on"));
    assert!(settings(folder.path()).contains("code = \"P-001\"\ncode_project = true\n"));
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(cfg.code_project);
}

#[test]
fn without_the_flag_the_project_is_not_a_code_project() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("code project:").not());
    assert!(!settings(folder.path()).contains("code_project"));
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(!cfg.code_project);
}

#[cfg(unix)]
#[test]
fn on_linux_the_folders_and_the_settings_are_group_writable() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains(
            "group: files are writable by the group of this folder",
        ));
    let dossier = folder.path().join("DOSSIER");
    let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode(&dossier), 0o2775);
    for name in [
        "decisions",
        "assumptions",
        "findings",
        "lessons",
        "usage",
        ".state",
    ] {
        assert_eq!(mode(&dossier.join(name)), 0o2775, "{name}");
    }
    assert_eq!(mode(&dossier.join("dossier.toml")), 0o664);
    assert_eq!(mode(&dossier.join(".state").join("journal.jsonl")), 0o664);
}

#[cfg(not(unix))]
#[test]
fn on_windows_no_group_line_is_written() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("group:").not());
}

#[test]
fn team_dossier_creates_the_team_folders_and_the_state_folder_only() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--team-dossier"]).success();
    let dossier = folder.path().join("DOSSIER");
    for name in ["lessons", "howtos", "tools", ".state"] {
        assert!(dossier.join(name).is_dir(), "{name} is missing");
    }
    for name in ["decisions", "assumptions", "findings", "usage"] {
        assert!(!dossier.join(name).exists(), "{name} must not be there");
    }
}

#[test]
fn team_dossier_settings_say_kind_team_right_after_the_code() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--team-dossier"]).success();
    assert!(
        settings(folder.path()).starts_with("format = 1\ncode = \"P-001\"\nkind = \"team\"\n"),
        "{}",
        settings(folder.path())
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.kind, "team");
    assert_eq!(cfg.code, "P-001");
    assert!(!cfg.code_project);
}

#[test]
fn team_dossier_state_folder_has_a_gitignore_for_the_lock_the_holder_and_shown() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--team-dossier"]).success();
    let ignore = folder
        .path()
        .join("DOSSIER")
        .join(".state")
        .join(".gitignore");
    assert_eq!(
        std::fs::read_to_string(ignore).unwrap(),
        "write.lock\nholder.json\nshown/\n"
    );
}

#[test]
fn team_dossier_says_so_on_its_first_line() {
    let folder = tempfile::tempdir().unwrap();
    let out = init(folder.path(), &["--code", "P-001", "--team-dossier"])
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert_eq!(text.lines().next(), Some("created the team dossier P-001"));
}

#[test]
fn team_dossier_journal_line_names_the_team_dossier_and_the_agent() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("DOSSIER_AGENT", "anna")],
        &["--code", "P-001", "--team-dossier"],
    )
    .success();
    let records = journal_of(folder.path());
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.time, NOW);
    assert_eq!(record.name, "anna");
    assert_eq!(record.person, "anna");
    assert_eq!(record.command, "init");
    assert_eq!(record.id, "DOSSIER");
    assert_eq!(record.event, "created");
    assert_eq!(record.detail, "team dossier, code P-001");
}

#[test]
fn team_dossier_with_code_project_is_refused_and_nothing_is_created() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &["--code", "P-001", "--team-dossier", "--code-project"],
    )
    .code(1)
    .stderr(contains(
        "error: a team dossier is not a code project: drop --code-project or --team-dossier",
    ));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn a_refused_team_dossier_as_json_gives_the_bad_value_name() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &[
            "--code",
            "P-001",
            "--team-dossier",
            "--code-project",
            "--json",
        ],
    )
    .code(1)
    .stdout(contains("\"ok\":false"))
    .stdout(contains("\"code\":\"BAD_VALUE\""));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn a_project_dossier_has_no_kind_line_and_a_state_gitignore_for_the_session_files() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"]).success();
    let dossier = folder.path().join("DOSSIER");
    assert!(!settings(folder.path()).contains("kind ="));
    assert_eq!(
        std::fs::read_to_string(dossier.join(".state").join(".gitignore")).unwrap(),
        "write.lock\nholder.json\nshown/\n"
    );
    let cfg = dossier::config::load(&dossier).unwrap();
    assert_eq!(cfg.kind, "project");
}

#[cfg(unix)]
#[test]
fn on_linux_the_team_dossier_ignore_file_is_group_writable() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001", "--team-dossier"]).success();
    let ignore = folder
        .path()
        .join("DOSSIER")
        .join(".state")
        .join(".gitignore");
    let mode = std::fs::metadata(&ignore).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode, 0o664);
}

/// `init --team-dossier --code TEAM` run as `agent` ("" = no name) in `folder`, not yet checked.
fn init_team_as(folder: &Path, agent: &str) -> assert_cmd::assert::Assert {
    empty_project()
        .command(agent, TEAM_NOW)
        .current_dir(folder)
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
}

#[test]
fn a_new_team_dossier_lists_its_creator_as_the_first_member() {
    let folder = tempfile::tempdir().unwrap();
    init_team_as(folder.path(), "anna").success();
    assert!(
        settings(folder.path()).contains("kind = \"team\"\nmembers = [\"anna\"]\n"),
        "{}",
        settings(folder.path())
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.members, ["anna"]);
}

#[test]
fn the_person_of_an_llm_name_is_the_first_member_of_a_new_team_dossier() {
    let folder = tempfile::tempdir().unwrap();
    init_team_as(folder.path(), "anna - claude").success();
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.members, ["anna"]);
}

#[test]
fn the_agent_option_names_the_first_member_before_the_environment() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("DOSSIER_AGENT", "anna")],
        &["--team-dossier", "--code", "TEAM", "--agent", "ben"],
    )
    .success();
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.members, ["ben"]);
}

#[test]
fn anna_can_add_ben_as_a_member_of_the_team_dossier_she_made() {
    let folder = tempfile::tempdir().unwrap();
    init_team_as(folder.path(), "anna").success();
    empty_project()
        .command("anna", TEAM_NOW)
        .current_dir(folder.path())
        .args(["team", "member", "add", "ben"])
        .assert()
        .success()
        .stdout(contains("added member ben"));
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert_eq!(cfg.members, ["anna", "ben"]);
}

#[test]
fn a_new_team_dossier_without_a_name_has_no_member_and_says_how_to_add_one() {
    let folder = tempfile::tempdir().unwrap();
    init_team_as(folder.path(), "")
        .success()
        .stderr(contains(
            "warning: no name given, so the team dossier has no member",
        ))
        .stderr(contains("team member add"));
    assert!(
        !settings(folder.path()).contains("members"),
        "{}",
        settings(folder.path())
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(cfg.members.is_empty());
}

#[test]
fn a_project_dossier_gets_no_members_line_even_when_anna_makes_it() {
    let folder = tempfile::tempdir().unwrap();
    init_with(
        folder.path(),
        &[("DOSSIER_AGENT", "anna")],
        &["--code", "P-001"],
    )
    .success();
    assert!(
        !settings(folder.path()).contains("members"),
        "{}",
        settings(folder.path())
    );
}

/// A folder for a project that has no dossier yet: `common::Project::new` would run `init` in it.
fn empty_project() -> common::Project {
    common::Project {
        dir: tempfile::tempdir().unwrap(),
    }
}

/// A team dossier made by anna (`init --team-dossier --code TEAM`) in a folder of its own, whose
/// name has a space and an umlaut. Anna, who made it, is its first member (R-TEAM-7). Returns the
/// folder, which keeps the dossier alive, and the team's DOSSIER folder.
fn team_dossier() -> (tempfile::TempDir, PathBuf) {
    let folder = tempfile::Builder::new()
        .prefix("Team Äpfel ")
        .tempdir()
        .unwrap();
    empty_project()
        .command("anna", TEAM_NOW)
        .current_dir(folder.path())
        .args(["init", "--team-dossier", "--code", "TEAM"])
        .assert()
        .success();
    let dossier = folder.path().join("DOSSIER");
    (folder, dossier)
}

/// A path as dossier.toml and the team's register store it: with / separators.
fn slashed(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "/")
}

/// The team's projects.md as text, or nothing while the team has no such file yet.
fn projects_of(team: &Path) -> String {
    std::fs::read_to_string(team.join("projects.md")).unwrap_or_default()
}

/// `init --code <code> --with-team <team>` run in `project` as `agent`, not yet checked.
fn init_with_team(
    project: &common::Project,
    agent: &str,
    code: &str,
    team: &Path,
) -> assert_cmd::assert::Assert {
    project.run_at(
        agent,
        TEAM_NOW,
        &[
            "init",
            "--code",
            code,
            "--with-team",
            team.to_str().unwrap(),
        ],
    )
}

#[test]
fn init_with_team_links_the_project_and_registers_its_code() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    init_with_team(&project, "anna", "P-002", &team)
        .success()
        .stdout(contains("created DOSSIER for P-002"))
        .stdout(contains("linked to the team dossier"))
        .stdout(contains("registered P-002 in the team dossier"));

    // The project's settings name the team dossier, as `team use` stores it.
    let stored_team = slashed(&team);
    assert!(
        settings(project.root()).contains(&format!("team = \"{stored_team}\"\n")),
        "{}",
        settings(project.root())
    );
    let cfg = dossier::config::load(&project.root().join("DOSSIER")).unwrap();
    assert_eq!(cfg.team.as_deref(), Some(stored_team.as_str()));

    // The team's projects.md has the code, its project folder and the history line.
    let stored_project = slashed(project.root());
    assert!(
        projects_of(&team).contains(&format!(
            "## P-002\n\n- folder: {stored_project}\n- history: 2026-11-20T10:00:00+01:00 anna: registered\n"
        )),
        "{}",
        projects_of(&team)
    );

    // The project's journal records the link, and the team's journal the registration.
    let records = journal_of(project.root());
    let link = records.last().unwrap();
    assert_eq!(link.id, "SETTINGS");
    assert_eq!(link.event, "changed");
    assert_eq!(link.detail, format!("team {stored_team}"));
    let records = dossier::journal::read(&team.join(".state")).unwrap();
    let record = records.last().unwrap();
    assert_eq!(record.time, TEAM_NOW);
    assert_eq!(record.name, "anna");
    assert_eq!(record.command, "init");
    assert_eq!(record.id, "PROJECTS");
    assert_eq!(record.event, "registered");
    assert_eq!(record.detail, "P-002");
}

#[test]
fn init_with_team_as_json_names_the_team_and_the_registration() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    project
        .run_at(
            "anna",
            TEAM_NOW,
            &[
                "init",
                "--code",
                "P-002",
                "--with-team",
                team.to_str().unwrap(),
                "--json",
            ],
        )
        .success()
        .stdout(contains("\"ok\":true"))
        .stdout(contains(format!("\"team\":\"{}\"", slashed(&team))))
        .stdout(contains("\"registered\":true"));
}

#[test]
fn a_code_registered_for_another_folder_is_refused_and_nothing_is_created() {
    let (_team_folder, team) = team_dossier();
    let first = empty_project();
    init_with_team(&first, "anna", "P-002", &team).success();
    let before = projects_of(&team);

    let second = empty_project();
    init_with_team(&second, "anna", "P-002", &team)
        .code(1)
        .stderr(contains(format!(
            "error: P-002 is already registered for {}",
            slashed(first.root())
        )));
    assert!(!second.root().join("DOSSIER").exists());
    assert_eq!(projects_of(&team), before);
}

#[test]
fn the_same_folder_again_adds_nothing_to_the_team() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    init_with_team(&project, "anna", "P-002", &team).success();
    let before = projects_of(&team);

    // The project's dossier is gone, so init runs again in the same folder.
    std::fs::remove_dir_all(project.root().join("DOSSIER")).unwrap();
    init_with_team(&project, "anna", "P-002", &team)
        .success()
        .stdout(contains("P-002 is registered in the team dossier already"));
    assert!(
        project
            .root()
            .join("DOSSIER")
            .join("dossier.toml")
            .is_file()
    );
    assert_eq!(projects_of(&team), before);
}

#[test]
fn a_name_that_is_not_a_team_member_is_refused_and_nothing_is_created() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    project
        .run_at(
            "ben",
            TEAM_NOW,
            &[
                "init",
                "--code",
                "P-002",
                "--with-team",
                team.to_str().unwrap(),
                "--json",
            ],
        )
        .code(1)
        .stdout(contains("\"code\":\"UNKNOWN_NAME\""));
    assert!(!project.root().join("DOSSIER").exists());
    assert!(!projects_of(&team).contains("P-002"));
}

#[test]
fn a_path_that_is_not_a_team_dossier_is_refused_and_nothing_is_created() {
    let project = empty_project();
    let missing = project.root().join("nowhere");
    init_with_team(&project, "anna", "P-002", &missing)
        .code(4)
        .stderr(contains("no team dossier here"));
    assert!(!project.root().join("DOSSIER").exists());

    // A project's dossier is not a team dossier.
    let other = empty_project();
    init(other.root(), &["--code", "P-009"]).success();
    init_with_team(&project, "anna", "P-002", &other.root().join("DOSSIER"))
        .code(1)
        .stderr(contains("is not a team dossier"));
    assert!(!project.root().join("DOSSIER").exists());
}

#[test]
fn a_team_that_cannot_be_read_is_refused_before_anything_is_created() {
    let (_team_folder, team) = team_dossier();
    std::fs::write(
        team.join("projects.md"),
        "<!-- dossier-format: 2 -->\n# Projects of TEAM\n",
    )
    .unwrap();
    let project = empty_project();
    init_with_team(&project, "anna", "P-002", &team)
        .code(4)
        .stderr(contains("this file needs a newer dossier program"));
    assert!(!project.root().join("DOSSIER").exists());
}

#[test]
fn a_team_dossier_cannot_also_be_linked_to_a_team() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &["--code", "P-001", "--team-dossier", "--with-team", "/x"],
    )
    .code(1)
    .stderr(contains(
        "error: a team dossier is not linked to a team: drop --with-team or --team-dossier",
    ));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn a_dry_run_with_team_creates_and_writes_nothing() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    let before = projects_of(&team);
    project
        .command("anna", TEAM_NOW)
        .args([
            "init",
            "--code",
            "P-002",
            "--with-team",
            team.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(contains("dry run: DOSSIER for P-002 would be linked"))
        .stderr(contains("warning: dry run: would create"))
        .stderr(contains("warning: dry run: would write"));
    assert!(!project.root().join("DOSSIER").exists());
    assert_eq!(projects_of(&team), before);
}

#[test]
fn a_dry_run_leaves_an_empty_folder_empty_and_names_what_it_would_create() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-004", "--dry-run"])
        .success()
        .stdout(contains("dry run: DOSSIER for P-004 would be created"))
        .stderr(contains("warning: dry run: would create"))
        .stderr(contains("warning: dry run: would write"))
        .stderr(contains("decisions"))
        .stderr(contains("dossier.toml"))
        .stderr(contains("journal.jsonl"));
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

#[test]
fn a_team_dossier_dry_run_creates_nothing() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &["--code", "TEAM", "--team-dossier", "--dry-run"],
    )
    .success()
    .stdout(contains("dry run: team dossier TEAM would be created"))
    .stderr(contains("warning: dry run: would create"))
    .stderr(contains("lessons"))
    .stderr(contains(".gitignore"));
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

/// A folder for the archive of a finished project, whose name has a space and an umlaut.
fn archive_folder() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("Archiv Äpfel ")
        .tempdir()
        .unwrap()
}

#[test]
fn past_marks_the_settings_right_after_the_code_line() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-003", "--past"])
        .success()
        .stdout(contains("created DOSSIER for P-003"))
        .stdout(contains("finished project: marked as past"));
    assert!(
        settings(folder.path()).starts_with("format = 1\ncode = \"P-003\"\npast = true\n"),
        "{}",
        settings(folder.path())
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(cfg.past);
    assert_eq!(cfg.describes, None);
}

#[test]
fn a_project_without_past_is_not_marked_as_past() {
    let folder = tempfile::tempdir().unwrap();
    init(folder.path(), &["--code", "P-001"])
        .success()
        .stdout(contains("finished project:").not());
    assert!(!settings(folder.path()).contains("past"));
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(!cfg.past);
    assert_eq!(cfg.describes, None);
}

#[test]
fn past_with_a_code_project_marks_both_after_the_code_line() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &["--code", "P-003", "--past", "--code-project"],
    )
    .success();
    assert!(
        settings(folder.path()).contains("code = \"P-003\"\ncode_project = true\npast = true\n"),
        "{}",
        settings(folder.path())
    );
    let cfg = dossier::config::load(&folder.path().join("DOSSIER")).unwrap();
    assert!(cfg.past);
    assert!(cfg.code_project);
}

#[test]
fn in_puts_the_dossier_in_that_folder_and_describes_the_current_one() {
    let project = tempfile::tempdir().unwrap();
    let archive = archive_folder();
    init_with(
        project.path(),
        &[("DOSSIER_AGENT", "anna")],
        &[
            "--code",
            "P-003",
            "--past",
            "--in",
            archive.path().to_str().unwrap(),
        ],
    )
    .success()
    .stdout(contains("DOSSIER is at"))
    .stdout(contains("describes the project folder"));
    assert!(!project.path().join("DOSSIER").exists());

    let dossier = archive.path().join("DOSSIER");
    let text = std::fs::read_to_string(dossier.join("dossier.toml")).unwrap();
    assert!(
        text.contains(&format!(
            "past = true\ndescribes = \"{}\"\n",
            slashed(project.path())
        )),
        "{text}"
    );
    let cfg = dossier::config::load(&dossier).unwrap();
    assert!(cfg.past);
    assert_eq!(
        cfg.describes.as_deref(),
        Some(slashed(project.path()).as_str())
    );

    // The journal of the dossier made in the archive records the creation.
    let records = dossier::journal::read(&dossier.join(".state")).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "anna");
    assert_eq!(records[0].command, "init");
    assert_eq!(records[0].event, "created");
}

#[test]
fn in_as_json_names_the_past_project_and_the_folder_it_describes() {
    let project = tempfile::tempdir().unwrap();
    let archive = archive_folder();
    init(
        project.path(),
        &[
            "--code",
            "P-003",
            "--past",
            "--in",
            archive.path().to_str().unwrap(),
            "--json",
        ],
    )
    .success()
    .stdout(contains("\"past\":true"))
    .stdout(contains(format!(
        "\"describes\":\"{}\"",
        slashed(project.path())
    )));
}

#[test]
fn in_without_past_is_refused_and_nothing_is_created() {
    let project = tempfile::tempdir().unwrap();
    let archive = archive_folder();
    init(
        project.path(),
        &["--code", "P-003", "--in", archive.path().to_str().unwrap()],
    )
    .code(1)
    .stderr(contains(
        "error: --in is for a finished project: add --past",
    ));
    assert!(!archive.path().join("DOSSIER").exists());
    assert!(!project.path().join("DOSSIER").exists());
}

#[test]
fn in_a_folder_that_does_not_exist_is_refused_and_nothing_is_created() {
    let project = tempfile::tempdir().unwrap();
    let missing = project.path().join("nowhere");
    init(
        project.path(),
        &[
            "--code",
            "P-003",
            "--past",
            "--in",
            missing.to_str().unwrap(),
        ],
    )
    .code(1)
    .stderr(contains("the folder for --in does not exist"));
    assert!(!missing.exists());
    assert!(!project.path().join("DOSSIER").exists());
}

#[test]
fn a_team_dossier_cannot_also_be_a_finished_project() {
    let folder = tempfile::tempdir().unwrap();
    init(
        folder.path(),
        &["--code", "P-001", "--team-dossier", "--past"],
    )
    .code(1)
    .stderr(contains(
        "error: a team dossier is not a finished project: drop --past or --team-dossier",
    ));
    assert!(!folder.path().join("DOSSIER").exists());
}

#[test]
fn in_with_team_registers_the_project_folder_it_describes() {
    let (_team_folder, team) = team_dossier();
    let project = empty_project();
    let archive = archive_folder();
    project
        .run_at(
            "anna",
            TEAM_NOW,
            &[
                "init",
                "--code",
                "P-003",
                "--past",
                "--in",
                archive.path().to_str().unwrap(),
                "--with-team",
                team.to_str().unwrap(),
            ],
        )
        .success()
        .stdout(contains("registered P-003 in the team dossier"));

    // The team registers the project folder, which is the folder the dossier describes.
    let stored_project = slashed(project.root());
    assert!(
        projects_of(&team).contains(&format!(
            "## P-003\n\n- folder: {stored_project}\n- history: 2026-11-20T10:00:00+01:00 anna: registered\n"
        )),
        "{}",
        projects_of(&team)
    );
    assert!(!project.root().join("DOSSIER").exists());
    let cfg = dossier::config::load(&archive.path().join("DOSSIER")).unwrap();
    assert!(cfg.past);
    assert_eq!(cfg.team.as_deref(), Some(slashed(&team).as_str()));
    assert_eq!(cfg.describes.as_deref(), Some(stored_project.as_str()));
}

#[cfg(unix)]
#[test]
fn a_finished_project_whose_folder_cannot_be_written_gets_its_dossier_with_in() {
    use std::os::unix::fs::PermissionsExt;
    let project = tempfile::tempdir().unwrap();
    let archive = archive_folder();
    let set_mode = |mode: u32| {
        std::fs::set_permissions(project.path(), std::fs::Permissions::from_mode(mode)).unwrap()
    };
    set_mode(0o555);
    let outcome = init(
        project.path(),
        &[
            "--code",
            "P-003",
            "--past",
            "--in",
            archive.path().to_str().unwrap(),
        ],
    );
    set_mode(0o755);
    outcome.success();
    assert!(!project.path().join("DOSSIER").exists());
    assert!(
        archive
            .path()
            .join("DOSSIER")
            .join("dossier.toml")
            .is_file()
    );
}
