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

/// Name from `flag`, else DOSSIER_AGENT, else TODO_AGENT. None: Code::Usage.
/// Not a person and not an LLM in cfg: Code::UnknownName, message lists the known names.
pub fn resolve(cfg: &Config, flag: Option<&str>) -> Result<Who> {
    let name = flag
        .and_then(given)
        .or_else(|| env_name("DOSSIER_AGENT"))
        .or_else(|| env_name("TODO_AGENT"))
        .ok_or_else(|| {
            Error::new(
                Code::Usage,
                "this command needs a name: --agent <name> or DOSSIER_AGENT",
            )
        })?;

    // A name that is both a person and an LLM is the person.
    if let Some(person) = find_person(cfg, &name) {
        return Ok(Who {
            name: person.name.clone(),
            person: person.name.clone(),
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
    Err(unknown(cfg, &name))
}

/// Whether `name` is one of the people of cfg.
pub fn is_person(cfg: &Config, name: &str) -> bool {
    find_person(cfg, name).is_some()
}

/// True when cfg.deciders is empty or names `person` (ignoring case).
pub fn may_decide(cfg: &Config, person: &str) -> bool {
    cfg.deciders.is_empty() || cfg.deciders.iter().any(|decider| same(decider, person))
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

fn find_person<'a>(cfg: &'a Config, name: &str) -> Option<&'a Person> {
    cfg.people.iter().find(|person| same(&person.name, name))
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

/// The refusal for a name that is neither a person nor an LLM: every known name is listed.
fn unknown(cfg: &Config, name: &str) -> Error {
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
            "'{name}' is neither a person nor an LLM in dossier.toml. People: {people}. \
             LLMs: {llms}; add one with: dossier people add <person> --llm <name>"
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
            code_project: false,
            brief_lines: 60,
            status_lines: 60,
            draft_days: 14,
            lock_wait_seconds: 30,
            ssh_timeout_seconds: 10,
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
        // environment meanwhile. A worker's shell may set TODO_AGENT, so both start removed.
        unsafe {
            std::env::remove_var("DOSSIER_AGENT");
            std::env::remove_var("TODO_AGENT");
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
}
