//! Who acts: the name of a person or an LLM, from `--agent` or the environment.

use crate::config::{Config, Person};
use crate::error::{Code, Error, Result};

/// The name of whoever acts, and the person it acts for.
#[derive(Debug, Clone, PartialEq)]
pub struct Who {
    pub name: String,
    pub person: String,
    pub is_llm: bool,
}

/// The name from `flag`, DOSSIER_AGENT, TODO_AGENT, USERNAME, USER (empty counts as unset), then
/// a person (a member or a [[people]] name), an LLM registered for a person, or `<person> -
/// <anything>` (format.md §21). A plain name of `names` is known but never writes:
/// Code::UnknownName. No name at all: Code::Usage.
pub fn resolve(cfg: &Config, flag: Option<&str>) -> Result<Who> {
    let name = flag
        .and_then(given)
        .or_else(|| env_name("DOSSIER_AGENT"))
        .or_else(|| env_name("TODO_AGENT"))
        .or_else(|| env_name("USERNAME"))
        .or_else(|| env_name("USER"))
        .ok_or_else(|| {
            Error::new(
                Code::Usage,
                "this command needs a name: --agent <name> or DOSSIER_AGENT",
            )
        })?;

    // A name that is a person is that person, even when it contains " - ".
    if let Some(person) = person_name(cfg, &name) {
        return Ok(Who {
            name: person.to_string(),
            person: person.to_string(),
            is_llm: false,
        });
    }
    if let Some((person, llm)) = find_llm(cfg, &name) {
        return Ok(Who {
            name: llm.to_string(),
            person: person.name.clone(),
            is_llm: true,
        });
    }
    let llm_of_person = split_llm(&name).and_then(|(person, _)| person_name(cfg, person));
    if let Some(person) = llm_of_person {
        return Ok(Who {
            name: name.clone(),
            person: person.to_string(),
            is_llm: true,
        });
    }
    if is_plain(cfg, &name) {
        return Err(plain(&name));
    }
    Err(unknown(cfg, &name))
}

/// Whether `name` is a person: a member or a [[people]] name, ignoring case (format.md §21).
pub fn is_person(cfg: &Config, name: &str) -> bool {
    person_name(cfg, name).is_some()
}

/// A person or a plain name of `names`: may be an author, a source or --said-by.
pub fn is_known(cfg: &Config, name: &str) -> bool {
    is_person(cfg, name) || is_plain(cfg, name)
}

/// True when cfg.deciders is empty or names `person` (ignoring case).
pub fn may_decide(cfg: &Config, person: &str) -> bool {
    cfg.deciders.is_empty() || cfg.deciders.iter().any(|decider| same(decider, person))
}

/// The gate of a command that carries out a person's decision (R-SAID-1, R-SAID-4).
/// Ok(false): a person runs the command. Ok(true): an LLM runs it for its person.
/// `said_by` is trimmed; empty counts as none. `command` is the name used in the messages.
pub fn said_by_gate(who: &Who, said_by: Option<&str>, command: &str) -> Result<bool> {
    gate(who, said_by, command, crate::role::current())
}

/// `said_by_gate` with the role given, so that the cases can be tested without the environment.
fn gate(who: &Who, said_by: Option<&str>, command: &str, role: crate::role::Role) -> Result<bool> {
    let said_by = said_by.and_then(given);
    if !who.is_llm {
        return match said_by {
            Some(said_by) if !same(&said_by, &who.person) => Err(Error::new(
                Code::Usage,
                format!("--said-by is for an LLM; {} is a person", who.name),
            )),
            _ => Ok(false),
        };
    }
    if role == crate::role::Role::Worker {
        return Err(Error::new(
            Code::Refused,
            "a worker decides nothing: say it in your todo note",
        ));
    }
    match said_by {
        None => Err(Error::new(
            Code::NeedsPerson,
            format!(
                "{command} carries out a person's decision: ask your person, and with their yes \
                 run it again with --said-by {}",
                who.person
            ),
        )),
        Some(said_by) if !same(&said_by, &who.person) => Err(Error::new(
            Code::NeedsPerson,
            format!(
                "{} acts for {} only; --said-by names {said_by}",
                who.name, who.person
            ),
        )),
        Some(_) => Ok(true),
    }
}

/// For commands that run before or without people in the settings (init, people add): the name
/// from `flag`, DOSSIER_AGENT or TODO_AGENT, else the operating-system user (USERNAME, USER),
/// else "unknown"; person = name; is_llm = false. Never an error.
pub fn unchecked(flag: Option<&str>) -> Who {
    let name = flag
        .and_then(given)
        .or_else(|| env_name("DOSSIER_AGENT"))
        .or_else(|| env_name("TODO_AGENT"))
        .or_else(|| env_name("USERNAME"))
        .or_else(|| env_name("USER"))
        .unwrap_or_else(|| "unknown".to_string());
    Who {
        name: name.clone(),
        person: name,
        is_llm: false,
    }
}

/// "anna" or "anna via anna-claude".
pub fn label(who: &Who) -> String {
    if who.is_llm {
        format!("{} via {}", who.person, who.name)
    } else {
        who.person.clone()
    }
}

/// The value without surrounding spaces, or None when nothing is left of it.
fn given(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// The environment variable `key` as a name, or None when it is unset or empty.
fn env_name(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|value| given(&value))
}

/// Names are compared ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The name of the person called `name` as the settings write it: a [[people]] name first, then
/// a member. Ignores case and surrounding spaces.
fn person_name<'a>(cfg: &'a Config, name: &str) -> Option<&'a str> {
    let name = name.trim();
    cfg.people
        .iter()
        .map(|person| person.name.as_str())
        .chain(cfg.members.iter().map(String::as_str))
        .find(|known| same(known, name))
}

/// Whether `name` is a plain name of `names`: known, but it does not write.
fn is_plain(cfg: &Config, name: &str) -> bool {
    let name = name.trim();
    cfg.names.iter().any(|plain| same(plain, name))
}

/// `<person> - <anything>` split at its first " - ": both parts trimmed, neither of them empty.
fn split_llm(name: &str) -> Option<(&str, &str)> {
    let (person, rest) = name.split_once(" - ")?;
    let (person, rest) = (person.trim(), rest.trim());
    (!person.is_empty() && !rest.is_empty()).then_some((person, rest))
}

/// The LLM called `name` and the person it writes for.
fn find_llm<'a>(cfg: &'a Config, name: &str) -> Option<(&'a Person, &'a str)> {
    cfg.people.iter().find_map(|person| {
        person
            .llms
            .iter()
            .find(|llm| same(llm, name))
            .map(|llm| (person, llm.as_str()))
    })
}

/// The refusal for a plain name: it is known, but it cannot write.
fn plain(name: &str) -> Error {
    Error::new(
        Code::UnknownName,
        format!(
            "'{name}' is a plain name in dossier.toml: it may be an author or a source, but it \
             cannot write. Write as a member (an operating-system user) or as an LLM named \
             <person> - <anything>"
        ),
    )
}

/// The refusal for a name that is no member, person, LLM or plain name: every known name is
/// listed, and the form of an LLM.
fn unknown(cfg: &Config, name: &str) -> Error {
    let members = if cfg.members.is_empty() {
        "none".to_string()
    } else {
        cfg.members.join(", ")
    };
    let people = if cfg.people.is_empty() {
        "none".to_string()
    } else {
        let names: Vec<&str> = cfg.people.iter().map(|p| p.name.as_str()).collect();
        names.join(", ")
    };
    let llms: Vec<String> = cfg
        .people
        .iter()
        .flat_map(|person| {
            person
                .llms
                .iter()
                .map(move |llm| format!("{llm} (for {})", person.name))
        })
        .collect();
    let llms = if llms.is_empty() {
        "none".to_string()
    } else {
        llms.join(", ")
    };
    Error::new(
        Code::UnknownName,
        format!(
            "'{name}' is neither a person nor an LLM in dossier.toml. Members: {members}. \
             People: {people}. LLMs: {llms}; add one with: dossier people add <person> --llm <name>. \
             An LLM is named <person> - <anything>."
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Tags;

    /// Two people as dossier.toml writes them: Anna (LLM anna-claude) and Ben (LLM ben-codex).
    fn team() -> Config {
        Config {
            format: 1,
            code: "P-001".to_string(),
            tags: Tags {
                machine: Vec::new(),
                analysis: Vec::new(),
                software: Vec::new(),
            },
            people: vec![
                Person {
                    name: "Anna".to_string(),
                    llms: vec!["anna-claude".to_string()],
                },
                Person {
                    name: "Ben".to_string(),
                    llms: vec!["ben-codex".to_string()],
                },
            ],
            deciders: Vec::new(),
            roles: Vec::new(),
            topics: Vec::new(),
            code_project: false,
            brief_lines: 60,
            status_lines: 60,
            draft_days: 14,
            take_days: 14,
            lock_wait_seconds: 30,
            ssh_timeout_seconds: 10,
            kind: "project".to_string(),
            team: None,
            members: Vec::new(),
            names: Vec::new(),
            past: false,
            describes: None,
        }
    }

    fn who(name: &str, person: &str, is_llm: bool) -> Who {
        Who {
            name: name.to_string(),
            person: person.to_string(),
            is_llm,
        }
    }

    #[test]
    fn a_person_resolves_to_itself_with_the_name_as_written_in_the_settings() {
        let found = resolve(&team(), Some("anna")).expect("a person resolves");
        assert_eq!(found, who("Anna", "Anna", false));
    }

    #[test]
    fn an_llm_resolves_to_the_person_it_writes_for() {
        let found = resolve(&team(), Some("BEN-CODEX")).expect("an LLM resolves");
        assert_eq!(found, who("ben-codex", "Ben", true));
    }

    #[test]
    fn a_name_that_is_both_a_person_and_an_llm_is_the_person() {
        let mut cfg = team();
        cfg.people[0].llms.push("Ben".to_string());
        let found = resolve(&cfg, Some("ben")).expect("the person resolves");
        assert_eq!(found, who("Ben", "Ben", false));
    }

    #[test]
    fn an_unknown_name_is_refused_and_every_known_name_is_listed() {
        let error = resolve(&team(), Some("carl")).expect_err("carl is refused");
        assert_eq!(error.code, Code::UnknownName);
        assert_eq!(error.exit(), 1);
        assert!(error.message.contains("'carl'"), "{}", error.message);
        assert!(error.message.contains("Anna, Ben"), "{}", error.message);
        assert!(
            error.message.contains("anna-claude (for Anna)"),
            "{}",
            error.message
        );
        assert!(
            error.message.contains("ben-codex (for Ben)"),
            "{}",
            error.message
        );
        assert!(
            error
                .message
                .contains("add one with: dossier people add <person> --llm <name>"),
            "{}",
            error.message
        );
    }

    #[test]
    fn with_no_people_the_refusal_says_none_and_still_gives_the_hint() {
        let cfg = Config {
            people: Vec::new(),
            ..team()
        };
        let error = resolve(&cfg, Some("carl")).expect_err("carl is refused");
        assert_eq!(error.code, Code::UnknownName);
        assert!(error.message.contains("People: none"), "{}", error.message);
        assert!(error.message.contains("LLMs: none"), "{}", error.message);
        assert!(
            error.message.contains("dossier people add"),
            "{}",
            error.message
        );
    }

    #[test]
    fn is_person_ignores_case_and_is_false_for_an_llm_or_an_unknown_name() {
        let cfg = team();
        assert!(is_person(&cfg, "ANNA"));
        assert!(!is_person(&cfg, "anna-claude"));
        assert!(!is_person(&cfg, "carl"));
    }

    #[test]
    fn without_a_name_it_is_a_usage_error_and_the_environment_supplies_one() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment meanwhile. A worker's shell may set TODO_AGENT, and the operating-system
        // user is a name too, so all four start removed.
        unsafe {
            std::env::remove_var("DOSSIER_AGENT");
            std::env::remove_var("TODO_AGENT");
            std::env::remove_var("USERNAME");
            std::env::remove_var("USER");
        }
        let cfg = team();

        let error = resolve(&cfg, None).expect_err("no name is refused");
        assert_eq!(error.code, Code::Usage);
        assert_eq!(error.exit(), 2);
        assert_eq!(
            error.message,
            "this command needs a name: --agent <name> or DOSSIER_AGENT"
        );
        // An empty flag counts as no flag.
        let error = resolve(&cfg, Some("")).expect_err("empty flag is refused");
        assert_eq!(error.code, Code::Usage);

        // TODO_AGENT is the fallback.
        // SAFETY: ENV_LOCK is held (see above).
        unsafe { std::env::set_var("TODO_AGENT", "anna-claude") };
        let found = resolve(&cfg, None).expect("TODO_AGENT gives the name");
        assert_eq!(found, who("anna-claude", "Anna", true));

        // DOSSIER_AGENT comes before TODO_AGENT.
        // SAFETY: ENV_LOCK is held (see above).
        unsafe { std::env::set_var("DOSSIER_AGENT", "Ben") };
        let found = resolve(&cfg, None).expect("DOSSIER_AGENT gives the name");
        assert_eq!(found, who("Ben", "Ben", false));

        // An empty or blank DOSSIER_AGENT counts as missing, so TODO_AGENT is used.
        // SAFETY: ENV_LOCK is held (see above).
        unsafe { std::env::set_var("DOSSIER_AGENT", "   ") };
        let found = resolve(&cfg, None).expect("TODO_AGENT gives the name");
        assert_eq!(found, who("anna-claude", "Anna", true));

        // The flag comes before the environment, and an empty flag falls back to it.
        let found = resolve(&cfg, Some("ben-codex")).expect("the flag gives the name");
        assert_eq!(found, who("ben-codex", "Ben", true));
        let found = resolve(&cfg, Some("")).expect("an empty flag falls back");
        assert_eq!(found, who("anna-claude", "Anna", true));

        // SAFETY: ENV_LOCK is held (see above).
        unsafe {
            std::env::remove_var("DOSSIER_AGENT");
            std::env::remove_var("TODO_AGENT");
            std::env::remove_var("USERNAME");
            std::env::remove_var("USER");
        }
    }

    /// Sets or removes an environment variable. The caller holds `crate::ENV_LOCK`.
    fn set_env(key: &str, value: Option<std::ffi::OsString>) {
        // SAFETY: the caller holds crate::ENV_LOCK, so no other unit test reads or changes the
        // environment meanwhile.
        unsafe {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn may_decide_is_true_for_everyone_when_no_decider_is_named() {
        let cfg = team();
        assert!(may_decide(&cfg, "Anna"));
        assert!(may_decide(&cfg, "carl"));
    }

    #[test]
    fn may_decide_is_true_only_for_the_named_deciders_ignoring_case() {
        let cfg = Config {
            deciders: vec!["Anna".to_string()],
            ..team()
        };
        assert!(may_decide(&cfg, "anna"));
        assert!(may_decide(&cfg, "ANNA"));
        assert!(!may_decide(&cfg, "Ben"));
        assert!(!may_decide(&cfg, "anna-claude"));
    }

    #[test]
    fn a_person_needs_no_said_by_and_its_own_name_is_accepted() {
        let anna = who("anna", "anna", false);
        let ran = gate(&anna, None, "approve", crate::role::Role::Planner).expect("a person runs");
        assert!(!ran);
        let ran = gate(&anna, Some("Anna"), "approve", crate::role::Role::Planner)
            .expect("its own name is accepted");
        assert!(!ran);
    }

    #[test]
    fn a_person_with_another_said_by_is_a_usage_error() {
        let anna = who("anna", "anna", false);
        let error = gate(&anna, Some("ben"), "approve", crate::role::Role::Planner)
            .expect_err("another name is refused");
        assert_eq!(error.code, Code::Usage);
        assert_eq!(error.exit(), 2);
        assert_eq!(error.message, "--said-by is for an LLM; anna is a person");
    }

    #[test]
    fn an_llm_as_a_worker_is_refused_with_or_without_said_by() {
        let llm = who("anna - claude", "anna", true);
        for said_by in [None, Some("anna")] {
            let error = gate(&llm, said_by, "approve", crate::role::Role::Worker)
                .expect_err("a worker decides nothing");
            assert_eq!(error.code, Code::Refused, "{said_by:?}");
            assert_eq!(
                error.message,
                "a worker decides nothing: say it in your todo note"
            );
        }
    }

    #[test]
    fn an_llm_without_said_by_needs_a_person_and_names_the_person_to_ask() {
        let llm = who("anna - claude", "anna", true);
        for said_by in [None, Some("  ")] {
            let error = gate(&llm, said_by, "approve", crate::role::Role::Planner)
                .expect_err("an LLM needs its person");
            assert_eq!(error.code, Code::NeedsPerson, "{said_by:?}");
            assert!(error.message.contains("approve"), "{}", error.message);
            assert!(
                error.message.contains("--said-by anna"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn an_llm_with_another_person_in_said_by_is_refused() {
        let llm = who("anna - claude", "anna", true);
        let error = gate(&llm, Some("ben"), "approve", crate::role::Role::Planner)
            .expect_err("an LLM acts for its person only");
        assert_eq!(error.code, Code::NeedsPerson);
        assert!(
            error.message.contains("acts for anna only"),
            "{}",
            error.message
        );
        assert!(error.message.contains("ben"), "{}", error.message);
    }

    #[test]
    fn an_llm_with_its_person_in_any_case_and_spaces_runs_for_it() {
        let llm = who("anna - claude", "anna", true);
        let ran = gate(&llm, Some(" ANNA "), "approve", crate::role::Role::Planner)
            .expect("the LLM runs for its person");
        assert!(ran);
    }

    #[test]
    fn unchecked_takes_the_name_from_the_flag_or_the_environment_and_never_fails() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        const KEYS: [&str; 4] = ["DOSSIER_AGENT", "TODO_AGENT", "USERNAME", "USER"];
        let saved: Vec<Option<std::ffi::OsString>> = KEYS.iter().map(std::env::var_os).collect();
        for key in KEYS {
            set_env(key, None);
        }
        // Nothing names anyone: "unknown", and no error.
        assert_eq!(unchecked(None), who("unknown", "unknown", false));

        // The operating-system user comes after the dossier and todo names.
        set_env("USER", Some("tester".into()));
        assert_eq!(unchecked(None), who("tester", "tester", false));
        set_env("USERNAME", Some("Windows Tester".into()));
        assert_eq!(
            unchecked(None),
            who("Windows Tester", "Windows Tester", false)
        );
        set_env("TODO_AGENT", Some("anna-claude".into()));
        assert_eq!(unchecked(None), who("anna-claude", "anna-claude", false));
        set_env("DOSSIER_AGENT", Some("Ben".into()));
        assert_eq!(unchecked(None), who("Ben", "Ben", false));

        // The flag comes first, and a blank flag falls back to the environment.
        assert_eq!(unchecked(Some("carl")), who("carl", "carl", false));
        assert_eq!(unchecked(Some("  ")), who("Ben", "Ben", false));

        for (key, value) in KEYS.into_iter().zip(saved) {
            set_env(key, value);
        }
    }

    #[test]
    fn label_is_the_person_or_the_person_via_the_llm() {
        let person = who("Anna", "Anna", false);
        assert_eq!(label(&person), "Anna");
        let llm = who("anna-claude", "Anna", true);
        assert_eq!(label(&llm), "Anna via anna-claude");
    }

    #[test]
    fn a_member_writes_as_itself_with_the_name_as_written_in_the_settings() {
        let cfg = Config {
            members: vec!["Tony".to_string()],
            ..team()
        };
        assert_eq!(
            resolve(&cfg, Some("tony")).expect("a member resolves"),
            who("Tony", "Tony", false)
        );
        assert!(is_person(&cfg, "TONY"));
    }

    #[test]
    fn a_member_with_a_dash_and_any_text_is_an_llm_of_that_member() {
        let cfg = Config {
            members: vec!["tony".to_string()],
            ..team()
        };
        let found = resolve(&cfg, Some("tony - claude opus 5.5 high")).expect("an LLM resolves");
        assert_eq!(found, who("tony - claude opus 5.5 high", "tony", true));
        // The name is kept as given (trimmed at its ends); the person is as the settings write it.
        let found = resolve(&cfg, Some("  TONY  -  claude  ")).expect("an LLM resolves");
        assert_eq!(found, who("TONY  -  claude", "tony", true));
    }

    #[test]
    fn a_name_that_is_a_person_wins_over_the_dash_form() {
        let cfg = Config {
            members: vec!["tony".to_string(), "tony - lab".to_string()],
            ..team()
        };
        assert_eq!(
            resolve(&cfg, Some("tony - lab")).expect("the person resolves"),
            who("tony - lab", "tony - lab", false)
        );
        assert_eq!(
            resolve(&cfg, Some("tony - claude")).expect("an LLM resolves"),
            who("tony - claude", "tony", true)
        );
    }

    #[test]
    fn a_dash_name_with_an_unknown_person_or_an_empty_part_is_refused() {
        let cfg = Config {
            members: vec!["tony".to_string()],
            ..team()
        };
        for name in ["carl - claude", "tony -", "- claude", "tony - "] {
            let error = resolve(&cfg, Some(name)).expect_err("refused");
            assert_eq!(error.code, Code::UnknownName, "{name}");
        }
        let error = resolve(&cfg, Some("carl - claude")).expect_err("carl is refused");
        assert!(
            error.message.contains("'carl - claude'"),
            "{}",
            error.message
        );
        assert!(error.message.contains("Members: tony"), "{}", error.message);
        assert!(
            error.message.contains("<person> - <anything>"),
            "{}",
            error.message
        );
    }

    #[test]
    fn a_plain_name_is_known_but_is_not_a_writer() {
        let cfg = Config {
            names: vec!["Dr. Weber".to_string()],
            ..team()
        };
        assert!(is_known(&cfg, "dr. weber"));
        assert!(!is_person(&cfg, "Dr. Weber"));
        let error = resolve(&cfg, Some("Dr. Weber")).expect_err("a plain name does not write");
        assert_eq!(error.code, Code::UnknownName);
        assert!(error.message.contains("cannot write"), "{}", error.message);
        // Nothing before the dash of `Dr. Weber - claude` is a person, so it is no LLM either.
        let error = resolve(&cfg, Some("Dr. Weber - claude")).expect_err("refused");
        assert_eq!(error.code, Code::UnknownName);
    }

    #[test]
    fn is_known_is_a_person_or_a_plain_name_and_never_an_llm() {
        let cfg = Config {
            members: vec!["tony".to_string()],
            names: vec!["Dr. Weber".to_string()],
            ..team()
        };
        assert!(is_known(&cfg, "Anna"));
        assert!(is_known(&cfg, "TONY"));
        assert!(is_known(&cfg, "Dr. Weber"));
        assert!(!is_known(&cfg, "anna-claude"));
        assert!(!is_known(&cfg, "carl"));
    }

    #[test]
    fn without_a_name_the_operating_system_user_is_the_default() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        const KEYS: [&str; 4] = ["DOSSIER_AGENT", "TODO_AGENT", "USERNAME", "USER"];
        let saved: Vec<Option<std::ffi::OsString>> = KEYS.iter().map(std::env::var_os).collect();
        for key in KEYS {
            set_env(key, None);
        }
        let cfg = Config {
            members: vec!["tony".to_string()],
            ..team()
        };

        // USERNAME is the operating-system user (Windows); its case does not matter.
        set_env("USERNAME", Some("TONY".into()));
        assert_eq!(
            resolve(&cfg, None).expect("USERNAME gives the name"),
            who("tony", "tony", false)
        );
        // An empty USERNAME counts as unset, so USER is used.
        set_env("USERNAME", Some("  ".into()));
        set_env("USER", Some("anna-claude".into()));
        assert_eq!(
            resolve(&cfg, None).expect("USER gives the name"),
            who("anna-claude", "Anna", true)
        );
        // An operating-system user that is no member, person or LLM is refused.
        set_env("USER", Some("carl".into()));
        let error = resolve(&cfg, None).expect_err("carl is refused");
        assert_eq!(error.code, Code::UnknownName);
        // The flag and then DOSSIER_AGENT come before the operating-system user.
        assert_eq!(
            resolve(&cfg, Some("tony")).expect("the flag gives the name"),
            who("tony", "tony", false)
        );
        set_env("DOSSIER_AGENT", Some("Ben".into()));
        assert_eq!(
            resolve(&cfg, None).expect("DOSSIER_AGENT gives the name"),
            who("Ben", "Ben", false)
        );

        for (key, value) in KEYS.into_iter().zip(saved) {
            set_env(key, value);
        }
    }
}
