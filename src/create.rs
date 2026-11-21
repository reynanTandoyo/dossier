//! Creating an entry inside a transaction.

use crate::approval;
use crate::config::Config;
use crate::entry::new::{self, Draft};
use crate::entry::rules;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::ids;
use crate::names::{self, Who};
use crate::store;
use crate::tx::{Change, Tx};

/// What a command asks to create.
pub struct New {
    pub kind: Kind,
    pub title: String,
    pub fields: Vec<(String, String)>,
    pub sections: Vec<(String, String)>,
    pub said_by: Option<String>,
    pub draft: bool,
}

/// The fields the tool writes itself. A command's field with one of these keys is left out.
const TOOL_OWNED: [&str; 8] = [
    "status",
    "created",
    "author",
    "via",
    "approved",
    "valid-from",
    "valid-until",
    "history",
];

/// Who an entry is by, whether it is approved, and whether it is a proposal (a decision that is
/// not approved, status `proposed`).
struct Plan {
    author: String,
    via: Option<String>,
    approved: bool,
    proposed: bool,
}

/// Creates one entry inside a transaction and returns its ID and file text.
/// Author and status: a person writing: author = person, approved unless `draft`.
/// An LLM with `said_by` naming a person of the settings: author = that person, via = the LLM,
/// approved unless `draft`. An LLM without `said_by`: author = its person, via = the LLM;
/// approved for assumption, finding and usage unless `draft`; draft for a lesson.
/// An LLM writing a decision without `said_by`: Code::NeedsPerson.
/// A decision is a proposal (R-WRITE-12, R-WRITE-13, D21, D28) when it is written with `draft`
/// or its author is not allowed to decide (names::may_decide): status `proposed`, no `approved`
/// and `valid-from`, history lines created and proposed, and the journal detail `proposed`.
/// Approved entries get `approved`, `valid-from`, two history lines (created, approved) and,
/// for a lesson, approval::record. Refusing unknown tag values is not part of v0.1.
pub fn create(tx: &mut Tx, new: &New) -> Result<(String, String)> {
    let who = tx.ctx().writer()?.clone();
    let plan = plan_for(&tx.ctx().cfg, &who, new)?;
    let time = tx.time().to_string();

    // An entry that would be approved must be complete. This is checked before an ID is given
    // out, so that a refused entry takes no number. The ID does not change what is checked.
    let placeholder = format!("{}-NEW", new.kind.prefix());
    if plan.approved {
        check_for_approval(new.kind, &entry_text(new, &placeholder, &time, &who, &plan))?;
    }

    let ctx = tx.ctx();
    let id = if ctx.dry_run {
        placeholder
    } else {
        ids::next(&ctx.dossier, &ctx.state, new.kind)?
    };
    let text = entry_text(new, &id, &time, &who, &plan);
    let path = store::path_for(&ctx.dossier, new.kind, &id, &new.title);
    let state = ctx.state.clone();
    let dry_run = ctx.dry_run;

    let detail = if plan.proposed {
        "proposed"
    } else if plan.approved {
        "approved"
    } else {
        "draft"
    };
    tx.put(
        path,
        None,
        text.clone(),
        Change {
            id: id.clone(),
            event: "created".to_string(),
            detail: detail.to_string(),
        },
    );
    if plan.approved && new.kind == Kind::Lesson && !dry_run {
        approval::record(&state, &id, text.as_bytes())?;
    }
    Ok((id, text))
}

/// Who the entry is by, whether it is approved, and whether it is a proposal (the rules of the
/// comment on `create`). `said_by` is ignored when a person writes. An LLM's decision needs a
/// person.
fn plan_for(cfg: &Config, who: &Who, new: &New) -> Result<Plan> {
    let (author, via, approved) = if !who.is_llm {
        (who.person.clone(), None, !new.draft)
    } else {
        match said_by(new) {
            Some(name) => (
                person_as_written(cfg, name)?,
                Some(who.name.clone()),
                !new.draft,
            ),
            None if new.kind == Kind::Decision => {
                return Err(Error::new(
                    Code::NeedsPerson,
                    "a decision needs the person who made it: --said-by <person>",
                ));
            }
            None => (
                who.person.clone(),
                Some(who.name.clone()),
                !new.draft && new.kind != Kind::Lesson,
            ),
        }
    };
    // A decision that is a draft, or whose author may not decide, is a proposal: it is not
    // approved (R-WRITE-12, R-WRITE-13).
    let proposed = new.kind == Kind::Decision && (new.draft || !names::may_decide(cfg, &author));
    Ok(Plan {
        author,
        via,
        approved: approved && !proposed,
        proposed,
    })
}

/// The name given with `said_by`, trimmed. A blank name counts as none.
fn said_by(new: &New) -> Option<&str> {
    new.said_by
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// The person called `name` (ignoring case), written as the settings write the name.
fn person_as_written(cfg: &Config, name: &str) -> Result<String> {
    let unknown = || {
        Error::new(
            Code::UnknownName,
            format!("no person {name} in dossier.toml"),
        )
    };
    if !names::is_person(cfg, name) {
        return Err(unknown());
    }
    cfg.people
        .iter()
        .find(|person| person.name.to_lowercase() == name.to_lowercase())
        .map(|person| person.name.clone())
        .ok_or_else(unknown)
}

/// The text of the entry with `id`: the tool's fields, the command's fields, then the history
/// lines. The history lines name the writer and carry the time of the transaction.
fn entry_text(new: &New, id: &str, time: &str, who: &Who, plan: &Plan) -> String {
    let label = names::label(who);
    let status = if plan.proposed {
        Status::Proposed
    } else if plan.approved {
        Status::Approved
    } else {
        Status::Draft
    };
    let mut fields = vec![
        pair("status", status.name()),
        pair("created", time),
        pair("author", &plan.author),
    ];
    if let Some(via) = &plan.via {
        fields.push(pair("via", via));
    }
    if plan.approved {
        fields.push(pair("approved", &format!("{time} by {}", plan.author)));
        fields.push(pair("valid-from", time));
    }
    fields.extend(
        new.fields
            .iter()
            .filter(|(key, _)| !is_tool_owned(key))
            .cloned(),
    );
    fields.push(pair("history", &format!("{time} {label}: created")));
    if plan.proposed {
        fields.push(pair("history", &format!("{time} {label}: proposed")));
    } else if plan.approved {
        fields.push(pair("history", &format!("{time} {label}: approved")));
    }
    let draft = Draft {
        kind: new.kind,
        id: id.to_string(),
        title: new.title.clone(),
        fields,
        sections: new.sections.clone(),
    };
    new::render(&draft)
}

/// A field as a pair of owned strings.
fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}

/// Whether the tool writes this field itself (keys are compared without regard to case).
fn is_tool_owned(key: &str) -> bool {
    TOOL_OWNED
        .iter()
        .any(|owned| owned.eq_ignore_ascii_case(key))
}

/// Code::MissingField when the entry text cannot be approved: every problem of rules::problems,
/// in order.
fn check_for_approval(kind: Kind, text: &str) -> Result<()> {
    let entry = Entry::parse(text)?;
    let problems = rules::problems(&entry);
    if problems.is_empty() {
        return Ok(());
    }
    let what: Vec<String> = problems.into_iter().map(|problem| problem.what).collect();
    Err(Error::new(
        Code::MissingField,
        format!("cannot approve {}: {}", kind.name(), what.join("; ")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::ctx::Ctx;
    use crate::output::Output;
    use crate::tx;
    use std::fs;
    use std::sync::MutexGuard;

    /// Holds ENV_LOCK for the whole test. Creating an entry reads DOSSIER_NOW, which the time
    /// tests set, so the tests of this module wait for them.
    fn quiet() -> MutexGuard<'static, ()> {
        crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// A project folder with its DOSSIER folder: the four kind folders, the state folder, and
    /// the people anna (LLM anna-claude) and ben (LLM ben-codex). The context acts as `writer`.
    /// The folder name has a space and a non-ASCII letter. The temporary folder is returned so
    /// that it lives as long as the test.
    fn project(writer: &str, dry_run: bool) -> (tempfile::TempDir, Ctx) {
        let dir = tempfile::Builder::new()
            .prefix("Projekt Äpfel ")
            .tempdir()
            .expect("temporary folder");
        let dossier = dir.path().join("DOSSIER");
        for kind in Kind::all() {
            fs::create_dir_all(dossier.join(kind.folder())).expect("kind folder");
        }
        let state = dossier.join(".state");
        fs::create_dir_all(&state).expect("state folder");
        fs::write(dossier.join("dossier.toml"), config::default_text("P-001"))
            .expect("dossier.toml");
        config::add_person(&dossier, "anna", &["anna-claude".to_string()]).expect("add anna");
        config::add_person(&dossier, "ben", &["ben-codex".to_string()]).expect("add ben");
        let cfg = config::load(&dossier).expect("the settings load");
        let who = names::resolve(&cfg, Some(writer)).expect("the writer is known");
        let ctx = Ctx {
            root: dir.path().to_path_buf(),
            dossier,
            state,
            cfg,
            who: Some(who),
            json: false,
            dry_run,
            command: "add".to_string(),
        };
        (dir, ctx)
    }

    /// A new entry of `kind` with no fields, sections, name or draft flag.
    fn new_of(kind: Kind, title: &str) -> New {
        New {
            kind,
            title: title.to_string(),
            fields: Vec::new(),
            sections: Vec::new(),
            said_by: None,
            draft: false,
        }
    }

    /// The sections that make a decision complete (invented text).
    fn decision_sections() -> Vec<(String, String)> {
        vec![
            pair("Decision", "An invented decision for a test."),
            pair("Reason", "An invented reason for a test."),
        ]
    }

    /// Runs `create` in a transaction. Returns the ID, the text and the time of the transaction,
    /// read inside the transaction.
    fn run(ctx: &Ctx, new: &New) -> Result<(String, String, String)> {
        let mut made = None;
        tx::write(ctx, |tx| {
            let time = tx.time().to_string();
            let (id, text) = create(tx, new)?;
            made = Some((id, text, time));
            Ok(Output::text("created"))
        })?;
        Ok(made.expect("the transaction ran"))
    }

    #[test]
    fn a_persons_decision_is_approved_with_no_via() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna", false);
        // said_by is ignored when a person writes.
        let new = New {
            sections: decision_sections(),
            said_by: Some("ben".to_string()),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the decision is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via"), None);
        assert_eq!(entry.field("created"), Some(time.clone()));
        assert_eq!(entry.field("approved"), Some(format!("{time} by anna")));
        assert_eq!(entry.field("valid-from"), Some(time.clone()));
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna: created"),
                format!("{time} anna: approved"),
            ]
        );
        assert!(problems_of(&entry).is_empty());
        let path = store::path_for(&ctx.dossier, Kind::Decision, &id, "Use the transient runs");
        assert_eq!(fs::read_to_string(path).expect("the file"), text);
    }

    #[test]
    fn an_llm_decision_without_said_by_needs_a_person_even_as_a_draft() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        for draft in [false, true] {
            let new = New {
                sections: decision_sections(),
                draft,
                ..new_of(Kind::Decision, "Use the transient runs")
            };
            let error = run(&ctx, &new).expect_err("the decision is refused");
            assert_eq!(error.code, Code::NeedsPerson);
            assert_eq!(
                error.message,
                "a decision needs the person who made it: --said-by <person>"
            );
        }
        // Nothing was given out or written.
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("decisions")));
    }

    #[test]
    fn an_llm_decision_with_said_by_is_approved_for_that_person() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        // The name is matched without regard to case and written as the settings have it.
        let new = New {
            sections: decision_sections(),
            said_by: Some("ANNA".to_string()),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the decision is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("approved"), Some(format!("{time} by anna")));
        assert_eq!(entry.field("valid-from"), Some(time.clone()));
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna via anna-claude: created"),
                format!("{time} anna via anna-claude: approved"),
            ]
        );
    }

    #[test]
    fn an_llm_said_by_must_name_a_person() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        for name in ["carl", "anna-claude"] {
            let new = New {
                sections: decision_sections(),
                said_by: Some(name.to_string()),
                ..new_of(Kind::Decision, "Use the transient runs")
            };
            let error = run(&ctx, &new).expect_err("the name is refused");
            assert_eq!(error.code, Code::UnknownName);
            assert_eq!(error.message, format!("no person {name} in dossier.toml"));
        }
    }

    #[test]
    fn an_llm_lesson_without_said_by_is_a_draft_and_is_not_recorded() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        // A draft may lack the sections a lesson needs for approval.
        let new = New {
            fields: vec![pair("confidence", "simulated")],
            sections: vec![pair("Statement", "An invented statement for a test.")],
            ..new_of(Kind::Lesson, "Steady runs overpredict")
        };
        let (id, text, time) = run(&ctx, &new).expect("the lesson is created");

        assert_eq!(id, "LES-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Draft));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("approved"), None);
        assert_eq!(entry.field("valid-from"), None);
        assert_eq!(
            entry.history(),
            vec![format!("{time} anna via anna-claude: created")]
        );
        assert!(!problems_of(&entry).is_empty());
        assert!(!ctx.state.join("approved.json").exists());
    }

    #[test]
    fn an_approved_lesson_missing_valid_for_is_refused() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna", false);
        let new = New {
            fields: vec![pair("confidence", "measured")],
            sections: vec![
                pair("Statement", "An invented statement for a test."),
                pair("Evidence", "An invented evidence for a test."),
            ],
            ..new_of(Kind::Lesson, "Steady runs overpredict")
        };
        let error = run(&ctx, &new).expect_err("the lesson cannot be approved");
        assert_eq!(error.code, Code::MissingField);
        assert_eq!(
            error.message,
            "cannot approve lesson: missing section Valid for"
        );
        // Refused before an ID was given out, so nothing was written.
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("lessons")));
    }

    #[test]
    fn every_missing_part_of_an_approved_entry_is_listed() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna", false);
        // A lesson with no confidence and no sections: every problem is listed, in order.
        let new = New {
            sections: vec![pair("Statement", "An invented statement for a test.")],
            ..new_of(Kind::Lesson, "Steady runs overpredict")
        };
        let error = run(&ctx, &new).expect_err("the lesson cannot be approved");
        assert_eq!(error.code, Code::MissingField);
        assert_eq!(
            error.message,
            "cannot approve lesson: missing section Valid for; missing section Evidence; \
             missing field confidence"
        );
    }

    #[test]
    fn an_approved_lesson_is_recorded_for_its_checksum() {
        let _guard = quiet();
        // An LLM writing for another person: the author is that person, via the LLM.
        let (_dir, ctx) = project("ben-codex", false);
        let new = New {
            fields: vec![pair("confidence", "measured")],
            sections: vec![
                pair("Statement", "An invented statement for a test."),
                pair("Valid for", "An invented range for a test."),
                pair("Evidence", "An invented evidence for a test."),
            ],
            said_by: Some("anna".to_string()),
            ..new_of(Kind::Lesson, "Steady runs overpredict")
        };
        let (id, text, time) = run(&ctx, &new).expect("the lesson is created");

        assert_eq!(id, "LES-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via").as_deref(), Some("ben-codex"));
        assert_eq!(entry.field("approved"), Some(format!("{time} by anna")));
        assert!(matches!(
            approval::mark(&ctx.state, &id, text.as_bytes()),
            approval::Mark::Unchanged
        ));
    }

    #[test]
    fn the_command_cannot_set_the_fields_the_tool_writes() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna", false);
        let new = New {
            fields: vec![
                pair("status", "rejected"),
                pair("author", "ben"),
                pair("history", "2001-01-01T00:00:00+00:00 ben: created"),
                pair("task", "T-042"),
            ],
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (_id, text, time) = run(&ctx, &new).expect("the decision is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("task").as_deref(), Some("T-042"));
        assert_eq!(entry.history().len(), 2);
        assert_eq!(entry.history()[0], format!("{time} anna: created"));
    }

    #[test]
    fn a_dry_run_gives_an_id_and_writes_nothing() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna", true);
        let new = New {
            fields: vec![pair("confidence", "measured")],
            sections: vec![
                pair("Statement", "An invented statement for a test."),
                pair("Valid for", "An invented range for a test."),
                pair("Evidence", "An invented evidence for a test."),
            ],
            ..new_of(Kind::Lesson, "Steady runs overpredict")
        };
        let (id, text, _time) = run(&ctx, &new).expect("the dry run creates the text");

        assert_eq!(id, "LES-NEW");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert!(!ctx.state.join("ids.json").exists());
        assert!(!ctx.state.join("approved.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("lessons")));
    }

    #[test]
    fn a_decision_by_someone_who_may_not_decide_is_a_proposal() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("ben", false);
        ctx.cfg.deciders = vec!["anna".to_string()];
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the proposal is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Proposed));
        assert_eq!(entry.field("author").as_deref(), Some("ben"));
        assert_eq!(entry.field("via"), None);
        assert_eq!(entry.field("approved"), None);
        assert_eq!(entry.field("valid-from"), None);
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} ben: created"),
                format!("{time} ben: proposed"),
            ]
        );
        assert_eq!(
            journal_events(&ctx),
            vec![("created".to_string(), "proposed".to_string())]
        );
    }

    #[test]
    fn a_decision_by_a_decider_is_approved_when_deciders_are_named() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna", false);
        ctx.cfg.deciders = vec!["anna".to_string()];
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the decision is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("approved"), Some(format!("{time} by anna")));
        assert_eq!(entry.field("valid-from"), Some(time.clone()));
        assert_eq!(
            journal_events(&ctx),
            vec![("created".to_string(), "approved".to_string())]
        );
    }

    #[test]
    fn with_no_deciders_named_everyone_decides() {
        let _guard = quiet();
        let (_dir, ctx) = project("ben", false);
        assert!(ctx.cfg.deciders.is_empty());
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (_id, text, time) = run(&ctx, &new).expect("the decision is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("approved"), Some(format!("{time} by ben")));
    }

    #[test]
    fn a_draft_decision_is_a_proposal_even_for_a_decider() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna", false);
        ctx.cfg.deciders = vec!["anna".to_string()];
        let new = New {
            sections: decision_sections(),
            draft: true,
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the proposal is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Proposed));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("approved"), None);
        assert_eq!(entry.field("valid-from"), None);
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna: created"),
                format!("{time} anna: proposed"),
            ]
        );
        assert_eq!(
            journal_events(&ctx),
            vec![("created".to_string(), "proposed".to_string())]
        );
    }

    #[test]
    fn an_llm_decision_for_someone_who_may_not_decide_is_a_proposal() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.deciders = vec!["anna".to_string()];
        let new = New {
            sections: decision_sections(),
            said_by: Some("ben".to_string()),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (_id, text, time) = run(&ctx, &new).expect("the proposal is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Proposed));
        assert_eq!(entry.field("author").as_deref(), Some("ben"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("approved"), None);
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna via anna-claude: created"),
                format!("{time} anna via anna-claude: proposed"),
            ]
        );
    }

    #[test]
    fn a_usage_record_is_approved_at_once_even_when_an_llm_writes_it() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        // The decider rule does not apply to a usage record.
        ctx.cfg.deciders = vec!["anna".to_string()];
        let new = New {
            fields: vec![
                pair("software", "ExampleSolver"),
                pair("version", "2406"),
                pair("task", "T-042"),
            ],
            ..new_of(Kind::Usage, "ExampleSolver 2406 run of speedline point 7")
        };
        let (id, text, time) = run(&ctx, &new).expect("the usage record is created");

        assert_eq!(id, "USE-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.kind(), Some(Kind::Usage));
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("anna"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("approved"), Some(format!("{time} by anna")));
        assert!(problems_of(&entry).is_empty());
        assert_eq!(
            journal_events(&ctx),
            vec![("created".to_string(), "approved".to_string())]
        );
    }

    /// The (event, detail) of every journal line of the project, oldest first.
    fn journal_events(ctx: &Ctx) -> Vec<(String, String)> {
        crate::journal::read(&ctx.state)
            .expect("the journal")
            .into_iter()
            .map(|record| (record.event, record.detail))
            .collect()
    }

    /// The problems of an entry, as their texts.
    fn problems_of(entry: &Entry) -> Vec<String> {
        rules::problems(entry)
            .into_iter()
            .map(|problem| problem.what)
            .collect()
    }

    /// Whether a folder has no entries in it.
    fn dir_is_empty(folder: &std::path::Path) -> bool {
        fs::read_dir(folder).expect("the folder").next().is_none()
    }
}
