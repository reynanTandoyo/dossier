//! The settings of a project: dossier.toml (format.md §10).

use crate::error::{Code, Error, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A person of the project, and the LLMs that write for them.
pub struct Person {
    pub name: String,
    pub llms: Vec<String>,
}

/// The tag values the project allows.
pub struct Tags {
    pub machine: Vec<String>,
    pub analysis: Vec<String>,
    pub software: Vec<String>,
}

/// The settings of a project.
pub struct Config {
    pub format: u32,
    pub code: String,
    pub tags: Tags,
    pub people: Vec<Person>,
    /// The people who may decide (format.md §19); empty: everyone listed decides.
    pub deciders: Vec<String>,
    /// Roles added to the roles of a task's files (format.md §17, §19).
    pub roles: Vec<String>,
    /// True after `init --code-project` (format.md §19).
    pub code_project: bool,
    pub brief_lines: usize,
    pub status_lines: usize,
    pub draft_days: i64,
    pub lock_wait_seconds: u64,
    pub ssh_timeout_seconds: u64,
}

/// The name of the settings file inside the DOSSIER folder.
const FILE: &str = "dossier.toml";

/// The newest format of dossier.toml this program reads (format.md §10).
const NEWEST_FORMAT: u32 = 1;

/// Only the format. It is read before the rest, so that a newer file is reported as TOO_NEW
/// and not as an unknown key.
#[derive(Deserialize)]
struct Probe {
    format: u32,
}

/// dossier.toml as written: format.md §10. Unknown keys are errors.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    format: u32,
    code: String,
    #[serde(default)]
    deciders: Vec<String>,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    code_project: bool,
    #[serde(default)]
    tags: TagsFile,
    #[serde(default)]
    people: Vec<PersonFile>,
    #[serde(default)]
    limits: LimitsFile,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
struct TagsFile {
    machine: Vec<String>,
    analysis: Vec<String>,
    software: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonFile {
    name: String,
    #[serde(default)]
    llms: Vec<String>,
}

/// The limits table. A key that is left out takes its value from `Default`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
struct LimitsFile {
    brief_lines: usize,
    status_lines: usize,
    draft_days: i64,
    lock_wait_seconds: u64,
    ssh_timeout_seconds: u64,
}

impl Default for LimitsFile {
    fn default() -> LimitsFile {
        LimitsFile {
            brief_lines: 60,
            status_lines: 60,
            draft_days: 14,
            lock_wait_seconds: 30,
            ssh_timeout_seconds: 10,
        }
    }
}

/// The `name` line of a `[[people]]` table, read on its own.
#[derive(Deserialize)]
struct NameLine {
    name: String,
}

/// Reads the settings of `dossier`. An unknown key: Code::Config.
pub fn load(dossier: &Path) -> Result<Config> {
    let path = settings_path(dossier);
    parse(&read_text(&path)?, &path)
}

/// The text that `init` writes for a project with `code`.
pub fn default_text(code: &str) -> String {
    let quoted_code = quoted(code);
    format!(
        "format = 1\n\
         code = {quoted_code}\n\
         \n\
         [tags]\n\
         machine = []\n\
         analysis = []\n\
         software = []\n\
         \n\
         [limits]\n\
         brief_lines = 60\n\
         status_lines = 60\n\
         draft_days = 14\n\
         lock_wait_seconds = 30\n\
         ssh_timeout_seconds = 10\n"
    )
}

/// Adds a person, or extends a person's LLMs, and keeps the other lines.
pub fn add_person(dossier: &Path, person: &str, llms: &[String]) -> Result<()> {
    let path = settings_path(dossier);
    let text = read_text(&path)?;
    let cfg = parse(&text, &path)?;
    let edited = match cfg.people.iter().find(|known| same(&known.name, person)) {
        Some(known) => {
            let all = merged(&known.llms, llms);
            if all.len() == known.llms.len() {
                return Ok(());
            }
            with_llms(&text, person, &llms_array(&all), &path)?
        }
        None => {
            let eol = line_ending(&text);
            let list = llms_array(&merged(&[], llms));
            let name = quoted(person);
            format!(
                "{}{eol}[[people]]{eol}name = {name}{eol}llms = {list}{eol}",
                terminated(&text, eol)
            )
        }
    };
    // The edited file must still be a valid settings file before anything is written.
    parse(&edited, &path)?;
    std::fs::write(&path, edited)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))
}

fn settings_path(dossier: &Path) -> PathBuf {
    dossier.join(FILE)
}

fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|error| Error::new(Code::Io, format!("{}: {error}", path.display())))
}

fn config_error(path: &Path, error: impl std::fmt::Display) -> Error {
    Error::new(Code::Config, format!("{}: {error}", path.display()))
}

/// The settings in `text`, which is the content of `path`.
fn parse(text: &str, path: &Path) -> Result<Config> {
    // format.md §2: a byte-order mark at the start is accepted.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let probe: Probe = toml::from_str(text).map_err(|error| config_error(path, error))?;
    if probe.format > NEWEST_FORMAT {
        return Err(Error::new(
            Code::TooNew,
            "this dossier needs a newer dossier program",
        ));
    }
    let file: File = toml::from_str(text).map_err(|error| config_error(path, error))?;
    Ok(Config {
        format: file.format,
        code: file.code,
        tags: Tags {
            machine: file.tags.machine,
            analysis: file.tags.analysis,
            software: file.tags.software,
        },
        people: file
            .people
            .into_iter()
            .map(|person| Person {
                name: person.name,
                llms: person.llms,
            })
            .collect(),
        deciders: file.deciders,
        roles: file.roles,
        code_project: file.code_project,
        brief_lines: file.limits.brief_lines,
        status_lines: file.limits.status_lines,
        draft_days: file.limits.draft_days,
        lock_wait_seconds: file.limits.lock_wait_seconds,
        ssh_timeout_seconds: file.limits.ssh_timeout_seconds,
    })
}

/// A TOML basic string holding `text`: quotes, backslashes and control characters escaped.
fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A TOML array of strings, such as `["anna-claude", "anna-codex"]`.
fn llms_array(llms: &[String]) -> String {
    let items: Vec<String> = llms.iter().map(|llm| quoted(llm)).collect();
    format!("[{}]", items.join(", "))
}

/// Names are compared ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// `existing` with each name of `new` that it does not have yet (ignoring case) added at the end.
fn merged(existing: &[String], new: &[String]) -> Vec<String> {
    let mut all = existing.to_vec();
    for llm in new {
        if !all.iter().any(|known| same(known, llm)) {
            all.push(llm.clone());
        }
    }
    all
}

/// The line ending for new lines: CRLF when the file's last complete line has one (format.md §2).
fn line_ending(text: &str) -> &'static str {
    let last_complete = text
        .split_inclusive('\n')
        .rev()
        .find(|line| line.ends_with('\n'));
    if last_complete.is_some_and(|line| line.ends_with("\r\n")) {
        "\r\n"
    } else {
        "\n"
    }
}

/// The line ending of `line`: CRLF, LF, or nothing for a last line without one.
fn line_end(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

/// `line` with `eol` added when it has no line ending.
fn terminated(line: &str, eol: &str) -> String {
    if line.ends_with('\n') {
        line.to_string()
    } else {
        format!("{line}{eol}")
    }
}

/// A `[[people]]` header, spaces and a trailing comment allowed.
fn is_people_header(line: &str) -> bool {
    let head: String = line
        .split('#')
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    head == "[[people]]"
}

/// Any table header, such as `[limits]` or `[[people]]`.
fn is_table_header(line: &str) -> bool {
    line.trim_start().starts_with('[')
}

/// A `key = value` line for `key`.
fn key_is(line: &str, key: &str) -> bool {
    line.trim_start()
        .strip_prefix(key)
        .is_some_and(|rest| rest.trim_start().starts_with('='))
}

/// Whether `line` is `name = "<person>"`, in any TOML spelling of the string.
fn named(line: &str, person: &str) -> bool {
    toml::from_str::<NameLine>(line).is_ok_and(|parsed| same(&parsed.name, person))
}

/// The text with the `llms` line of the `[[people]]` table named `person` set to `value`. A
/// table without an `llms` line gets one after its `name` line. Every other line is kept.
fn with_llms(text: &str, person: &str, value: &str, path: &Path) -> Result<String> {
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let eol = line_ending(text);
    let headers: Vec<usize> = (0..lines.len())
        .filter(|&at| is_people_header(&lines[at]))
        .collect();
    for start in headers {
        let end = (start + 1..lines.len())
            .find(|&at| is_table_header(&lines[at]))
            .unwrap_or(lines.len());
        let Some(name) = (start + 1..end).find(|&at| key_is(&lines[at], "name")) else {
            continue;
        };
        if !named(&lines[name], person) {
            continue;
        }
        match (start + 1..end).find(|&at| key_is(&lines[at], "llms")) {
            Some(at) => {
                let old = lines[at].clone();
                let indent: String = old.chars().take_while(|c| c.is_whitespace()).collect();
                // What follows the array stays: a trailing comment and the line ending.
                let tail = match array_end(&old) {
                    Some(end) => &old[end..],
                    None => line_end(&old),
                };
                lines[at] = format!("{indent}llms = {value}{tail}");
            }
            None => {
                lines[name] = terminated(&lines[name], eol);
                lines.insert(name + 1, format!("llms = {value}{eol}"));
            }
        }
        return Ok(lines.concat());
    }
    Err(Error::new(
        Code::Config,
        format!("{}: no [[people]] table named {person}", path.display()),
    ))
}

/// The byte index just after the `]` that closes the array of the `key = [...]` line `line`.
/// None when the array does not close on this line. Brackets inside strings, and a comment
/// before the closing bracket, do not count.
fn array_end(line: &str) -> Option<usize> {
    let start = line.find('=')? + 1;
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (at, c) in line[start..].char_indices() {
        if let Some(open) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' && open == '"' {
                escaped = true;
            } else if c == open {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '[' => depth += 1,
            ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(start + at + 1);
                }
            }
            '#' => return None,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dossier folder holding `text` as dossier.toml.
    fn dossier_with(text: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temporary folder");
        std::fs::write(dir.path().join(FILE), text).expect("write dossier.toml");
        dir
    }

    fn settings(dir: &tempfile::TempDir) -> String {
        std::fs::read_to_string(dir.path().join(FILE)).expect("read dossier.toml")
    }

    fn loaded(dir: &tempfile::TempDir) -> Config {
        load(dir.path()).expect("dossier.toml loads")
    }

    fn refused(dir: &tempfile::TempDir) -> Error {
        load(dir.path()).err().expect("dossier.toml is refused")
    }

    #[test]
    fn default_text_is_the_text_of_format_section_10() {
        assert_eq!(
            default_text("P-017"),
            "format = 1\n\
             code = \"P-017\"\n\
             \n\
             [tags]\n\
             machine = []\n\
             analysis = []\n\
             software = []\n\
             \n\
             [limits]\n\
             brief_lines = 60\n\
             status_lines = 60\n\
             draft_days = 14\n\
             lock_wait_seconds = 30\n\
             ssh_timeout_seconds = 10\n"
        );
    }

    #[test]
    fn default_text_loads_with_the_defaults() {
        let dir = dossier_with(&default_text("P-017"));
        let cfg = loaded(&dir);
        assert_eq!(cfg.format, 1);
        assert_eq!(cfg.code, "P-017");
        assert!(cfg.tags.machine.is_empty());
        assert!(cfg.tags.analysis.is_empty());
        assert!(cfg.tags.software.is_empty());
        assert!(cfg.people.is_empty());
        assert!(cfg.deciders.is_empty());
        assert!(cfg.roles.is_empty());
        assert!(!cfg.code_project);
        assert_eq!(cfg.brief_lines, 60);
        assert_eq!(cfg.status_lines, 60);
        assert_eq!(cfg.draft_days, 14);
        assert_eq!(cfg.lock_wait_seconds, 30);
        assert_eq!(cfg.ssh_timeout_seconds, 10);
    }

    #[test]
    fn missing_limits_take_the_defaults() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"\n");
        let cfg = loaded(&dir);
        assert_eq!(cfg.brief_lines, 60);
        assert_eq!(cfg.status_lines, 60);
        assert_eq!(cfg.draft_days, 14);
        assert_eq!(cfg.lock_wait_seconds, 30);
        assert_eq!(cfg.ssh_timeout_seconds, 10);
    }

    #[test]
    fn a_limit_that_is_set_replaces_its_default_only() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"\n[limits]\ndraft_days = 3\n");
        let cfg = loaded(&dir);
        assert_eq!(cfg.draft_days, 3);
        assert_eq!(cfg.brief_lines, 60);
    }

    #[test]
    fn people_and_their_llms_are_read() {
        let dir = dossier_with(
            "format = 1\ncode = \"P-001\"\n\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\", \"anna-codex\"]\n\n[[people]]\nname = \"ben\"\n",
        );
        let cfg = loaded(&dir);
        assert_eq!(cfg.people.len(), 2);
        assert_eq!(cfg.people[0].name, "anna");
        assert_eq!(cfg.people[0].llms, ["anna-claude", "anna-codex"]);
        assert_eq!(cfg.people[1].name, "ben");
        assert!(cfg.people[1].llms.is_empty());
    }

    #[test]
    fn the_example_of_format_section_10_loads() {
        let dir = dossier_with(
            "format = 1\n\
             code = \"P-017\"\n\
             \n\
             [tags]\n\
             machine = [\"axial compressor\"]\n\
             analysis = [\"steady RANS\"]\n\
             software = []\n\
             \n\
             [[people]]\n\
             name = \"anna\"\n\
             llms = [\"anna-claude\", \"anna-codex\"]\n\
             \n\
             [limits]\n\
             brief_lines = 60\n\
             status_lines = 60\n\
             draft_days = 14\n\
             lock_wait_seconds = 30\n\
             ssh_timeout_seconds = 10\n",
        );
        let cfg = loaded(&dir);
        assert_eq!(cfg.format, 1);
        assert_eq!(cfg.code, "P-017");
        assert_eq!(cfg.tags.machine, ["axial compressor"]);
        assert_eq!(cfg.tags.analysis, ["steady RANS"]);
        assert!(cfg.tags.software.is_empty());
        assert_eq!(cfg.people[0].name, "anna");
        assert_eq!(cfg.people[0].llms, ["anna-claude", "anna-codex"]);
        assert_eq!(cfg.draft_days, 14);
    }

    #[test]
    fn the_example_of_format_section_19_loads_its_three_settings() {
        let dir = dossier_with(
            "format = 1\ncode = \"P-017\"\ndeciders = [\"anna\"]\nroles = [\"postprocessing\"]\ncode_project = false\n",
        );
        let cfg = loaded(&dir);
        assert_eq!(cfg.deciders, ["anna"]);
        assert_eq!(cfg.roles, ["postprocessing"]);
        assert!(!cfg.code_project);
        let dir = dossier_with("format = 1\ncode = \"P-017\"\ncode_project = true\n");
        assert!(loaded(&dir).code_project);
    }

    #[test]
    fn a_setting_of_the_wrong_type_is_a_config_error() {
        let dir = dossier_with("format = 1\ncode = \"P-017\"\ncode_project = \"yes\"\n");
        assert_eq!(refused(&dir).code, Code::Config);
        let dir = dossier_with("format = 1\ncode = \"P-017\"\ndeciders = \"anna\"\n");
        assert_eq!(refused(&dir).code, Code::Config);
    }

    #[test]
    fn a_byte_order_mark_at_the_start_is_accepted() {
        let dir = dossier_with(&format!("\u{feff}{}", default_text("P-001")));
        assert_eq!(loaded(&dir).code, "P-001");
    }

    #[test]
    fn an_unknown_top_level_key_is_a_config_error_naming_the_file() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"\ncolour = \"red\"\n");
        let error = refused(&dir);
        assert_eq!(error.code, Code::Config);
        assert!(error.message.contains("dossier.toml"), "{}", error.message);
        assert!(error.message.contains("colour"), "{}", error.message);
    }

    #[test]
    fn an_unknown_key_inside_a_table_is_a_config_error() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"\n[limits]\nbrief = 3\n");
        let error = refused(&dir);
        assert_eq!(error.code, Code::Config);
        assert!(error.message.contains("brief"), "{}", error.message);
    }

    #[test]
    fn a_syntax_error_names_the_file_and_the_line() {
        let dir = dossier_with("format = 1\ncode = P-001\n");
        let error = refused(&dir);
        assert_eq!(error.code, Code::Config);
        let path = dir.path().join(FILE).display().to_string();
        assert!(error.message.starts_with(&path), "{}", error.message);
        assert!(error.message.contains("line 2"), "{}", error.message);
    }

    #[test]
    fn a_missing_code_is_a_config_error() {
        let dir = dossier_with("format = 1\n");
        assert_eq!(refused(&dir).code, Code::Config);
    }

    #[test]
    fn format_2_is_too_new() {
        let dir = dossier_with("format = 2\ncode = \"P-001\"\n");
        let error = refused(&dir);
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(error.message, "this dossier needs a newer dossier program");
    }

    #[test]
    fn a_newer_format_is_too_new_even_with_keys_this_program_does_not_know() {
        let dir = dossier_with("format = 2\ncode = \"P-001\"\n[retention]\ndays = 3\n");
        assert_eq!(refused(&dir).code, Code::TooNew);
    }

    #[test]
    fn a_missing_file_is_an_io_error() {
        let dir = tempfile::tempdir().expect("temporary folder");
        assert_eq!(refused(&dir).code, Code::Io);
    }

    #[test]
    fn a_code_with_quotes_and_backslashes_is_escaped() {
        let code = "A \"B\"\\C";
        let dir = dossier_with(&default_text(code));
        assert_eq!(loaded(&dir).code, code);
    }

    #[test]
    fn add_person_twice_gives_one_person_with_both_llms() {
        let dir = dossier_with(&default_text("P-001"));
        add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect("first add");
        add_person(
            dir.path(),
            "Anna",
            &["anna-codex".to_string(), "ANNA-CLAUDE".to_string()],
        )
        .expect("second add");
        let cfg = loaded(&dir);
        assert_eq!(cfg.people.len(), 1);
        assert_eq!(cfg.people[0].name, "anna");
        assert_eq!(cfg.people[0].llms, ["anna-claude", "anna-codex"]);
    }

    #[test]
    fn add_person_appends_a_blank_line_and_the_table_and_keeps_the_lines_before() {
        let dir = dossier_with(&default_text("P-001"));
        let before = settings(&dir);
        add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect("add");
        let after = settings(&dir);
        assert!(after.starts_with(&before));
        assert_eq!(
            &after[before.len()..],
            "\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\"]\n"
        );
    }

    #[test]
    fn add_person_rewrites_only_the_llms_line_of_that_person() {
        let text = "format = 1\ncode = \"P-001\"\n# a comment\n\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\"]  # her LLM\n\n[[people]]\nname = \"ben\"\nllms = [\"ben-codex\"]\n\n[limits]\nbrief_lines = 60\n";
        let dir = dossier_with(text);
        add_person(dir.path(), "anna", &["anna-codex".to_string()]).expect("add");
        assert_eq!(
            settings(&dir),
            text.replace(
                "llms = [\"anna-claude\"]  # her LLM",
                "llms = [\"anna-claude\", \"anna-codex\"]  # her LLM"
            )
        );
    }

    #[test]
    fn add_person_with_llms_already_known_leaves_the_file_as_it_is() {
        let text = format!(
            "{}\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\"]\n",
            default_text("P-001").trim_end()
        );
        let dir = dossier_with(&text);
        add_person(dir.path(), "anna", &["Anna-Claude".to_string()]).expect("add");
        assert_eq!(settings(&dir), text);
    }

    #[test]
    fn add_person_adds_llms_to_a_person_that_has_none() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"\n\n[[people]]\nname = \"ben\"\n");
        add_person(dir.path(), "ben", &["ben-codex".to_string()]).expect("add");
        assert_eq!(
            settings(&dir),
            "format = 1\ncode = \"P-001\"\n\n[[people]]\nname = \"ben\"\nllms = [\"ben-codex\"]\n"
        );
        assert_eq!(loaded(&dir).people[0].llms, ["ben-codex"]);
    }

    #[test]
    fn add_person_to_a_file_without_a_final_line_break_ends_the_last_line_first() {
        let dir = dossier_with("format = 1\ncode = \"P-001\"");
        add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect("add");
        assert_eq!(
            settings(&dir),
            "format = 1\ncode = \"P-001\"\n\n[[people]]\nname = \"anna\"\nllms = [\"anna-claude\"]\n"
        );
    }

    #[test]
    fn add_person_uses_crlf_when_the_file_does() {
        let dir = dossier_with(&default_text("P-001").replace('\n', "\r\n"));
        let before = settings(&dir);
        add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect("add");
        let after = settings(&dir);
        assert!(after.starts_with(&before));
        let added = &after[before.len()..];
        assert_eq!(
            added,
            "\r\n[[people]]\r\nname = \"anna\"\r\nllms = [\"anna-claude\"]\r\n"
        );
    }

    #[test]
    fn add_person_refuses_a_file_it_cannot_parse_and_writes_nothing() {
        let text = "format = 1\ncode = \"P-001\"\ncolour = \"red\"\n";
        let dir = dossier_with(text);
        let error =
            add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect_err("add refused");
        assert_eq!(error.code, Code::Config);
        assert_eq!(settings(&dir), text);
    }

    #[test]
    fn add_person_refuses_a_newer_format_and_writes_nothing() {
        let text = "format = 2\ncode = \"P-001\"\n";
        let dir = dossier_with(text);
        let error =
            add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect_err("add refused");
        assert_eq!(error.code, Code::TooNew);
        assert_eq!(settings(&dir), text);
    }

    #[test]
    fn add_person_reads_a_bracket_inside_a_string_as_part_of_the_string() {
        let dir = dossier_with(&format!(
            "{}\n[[people]]\nname = \"anna\"\nllms = [\"a]b\"]  # her LLMs\n",
            default_text("P-001").trim_end()
        ));
        add_person(dir.path(), "anna", &["anna-codex".to_string()]).expect("add");
        assert!(settings(&dir).ends_with("llms = [\"a]b\", \"anna-codex\"]  # her LLMs\n"));
        assert_eq!(loaded(&dir).people[0].llms, ["a]b", "anna-codex"]);
    }

    #[test]
    fn add_person_refuses_a_multi_line_llms_list_and_writes_nothing() {
        let text = format!(
            "{}\n[[people]]\nname = \"anna\"\nllms = [\n  \"anna-claude\",\n]\n",
            default_text("P-001").trim_end()
        );
        let dir = dossier_with(&text);
        let error =
            add_person(dir.path(), "anna", &["anna-codex".to_string()]).expect_err("add refused");
        assert_eq!(error.code, Code::Config);
        assert_eq!(settings(&dir), text);
    }

    #[test]
    fn add_person_keeps_a_byte_order_mark() {
        let dir = dossier_with(&format!("\u{feff}{}", default_text("P-001")));
        add_person(dir.path(), "anna", &["anna-claude".to_string()]).expect("add");
        let after = settings(&dir);
        assert!(after.starts_with('\u{feff}'));
        assert_eq!(loaded(&dir).people[0].llms, ["anna-claude"]);
    }
}
