//! Creating an entry inside a transaction.

use crate::approval;
use crate::conf;
use crate::config::{self, Config};
use crate::ctx::Ctx;
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
const TOOL_OWNED: [&str; 10] = [
    "status",
    "created",
    "author",
    "via",
    "approved",
    "valid-from",
    "valid-until",
    "history",
    "imported",
    "source-document",
];

/// Who an entry is by, whether it is approved, whether it is a proposal (a decision that is
/// not approved, status `proposed`), and the document it is imported from, if any.
struct Plan {
    author: String,
    via: Option<String>,
    approved: bool,
    proposed: bool,
    source: Option<String>,
}

/// Creates one entry inside a transaction and returns its ID and file text.
/// Author and status: a person writing: author = person, approved unless `draft`.
/// An LLM with `said_by` naming a person or a plain name of the settings: author = that name, via
/// = the LLM, approved unless `draft`. An LLM without `said_by`: author = its person, via = the
/// LLM; approved for assumption, finding and usage unless `draft`; draft for a lesson, a how-to
/// and a tool (approval::guarded).
/// An LLM writing a decision without `said_by`: Code::NeedsPerson, unless it is a draft: then it
/// is a proposal by the LLM's person, via the LLM (R-WRITE-12: anyone may propose).
/// A decision is a proposal (R-WRITE-12, R-WRITE-13, D21, D28) when it is written with `draft`
/// or its author is not allowed to decide (names::may_decide): status `proposed`, no `approved`
/// and `valid-from`, history lines created and proposed, and the journal detail `proposed`.
/// An entry written from a document (`ctx.from_document`, R-IMPORT-3, R-IMPORT-5) is a draft
/// whoever writes it, with `imported: yes` and `source-document` set to that document.
/// Approved entries get `approved`, `valid-from`, two history lines (created, approved) and, for
/// a guarded kind (approval::guarded), approval::record. Refusing unknown tag values is not part
/// of v0.1. With topics in the settings (R-TOPIC-2, R-TOPIC-3), each topic named in the field
/// `topic` must be one of them (Code::Usage), and an LLM's decision, assumption, finding, lesson,
/// how-to or investigation that is not written from a document must name a topic (Code::Refused).
/// A team write from a project (`ctx.project`) is refused when the entry holds a confidential term
/// of that project, or when the project has no terms and has not stated there are none
/// (conf::gate and conf::check, R-WRITE-6, R-CONF-2). Both are checked before anything is queued.
pub fn create(tx: &mut Tx, new: &New) -> Result<(String, String)> {
    let who = tx.ctx().writer()?.clone();
    let plan = plan_for(&tx.ctx().cfg, &who, new, tx.ctx().from_document.as_deref())?;
    check_topics(tx.ctx(), &who, new)?;
    let time = tx.time().to_string();

    // An entry that would be approved must be complete. This is checked before an ID is given
    // out, so that a refused entry takes no number. The ID does not change what is checked.
    let placeholder = format!("{}-NEW", new.kind.prefix());
    if plan.approved {
        check_for_approval(new.kind, &entry_text(new, &placeholder, &time, &who, &plan))?;
    }

    // The confidential terms of the project, for a team write from a project. The entry is
    // checked with the placeholder ID before an ID is given out, as above; the text is checked
    // again with its ID, which is only known now.
    let terms = project_terms(tx.ctx())?;
    if let Some(terms) = &terms {
        conf::check(terms, &entry_text(new, &placeholder, &time, &who, &plan))?;
    }

    let ctx = tx.ctx();
    let id = if ctx.dry_run {
        placeholder
    } else {
        ids::next(&ctx.dossier, &ctx.state, new.kind)?
    };
    let text = entry_text(new, &id, &time, &who, &plan);
    if let Some(terms) = &terms {
        conf::check(terms, &text)?;
    }
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
    if plan.approved && approval::guarded(new.kind) && !dry_run {
        approval::record(&state, &id, text.as_bytes())?;
    }
    Ok((id, text))
}

/// Who the entry is by, whether it is approved, and whether it is a proposal (the rules of the
/// comment on `create`). `said_by` is ignored when a person writes. An LLM's approved decision
/// needs a person; an LLM's draft decision is a proposal of its own person. An entry written from
/// a document (`source`) is a draft whoever writes it: it is never approved and never a proposal.
fn plan_for(cfg: &Config, who: &Who, new: &New, source: Option<&str>) -> Result<Plan> {
    let draft = new.draft || source.is_some();
    let (author, via, approved) = if !who.is_llm {
        (who.person.clone(), None, !draft)
    } else {
        match said_by(new) {
            Some(name) => (name_as_written(cfg, name)?, Some(who.name.clone()), !draft),
            None if new.kind == Kind::Decision && !draft => {
                return Err(Error::new(
                    Code::NeedsPerson,
                    "a decision needs the person who made it: --said-by <person>",
                ));
            }
            None => (
                who.person.clone(),
                Some(who.name.clone()),
                !draft && !approval::guarded(new.kind),
            ),
        }
    };
    // A decision that is a draft, or whose author may not decide, is a proposal: it is not
    // approved (R-WRITE-12, R-WRITE-13). An imported decision is a draft instead.
    let proposed = source.is_none()
        && new.kind == Kind::Decision
        && (new.draft || !names::may_decide(cfg, &author));
    Ok(Plan {
        author,
        via,
        approved: approved && !proposed,
        proposed,
        source: source.map(str::to_string),
    })
}

/// The name given with `said_by`, trimmed. A blank name counts as none.
fn said_by(new: &New) -> Option<&str> {
    new.said_by
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// The name called `name` (ignoring case), written as the settings write it: a person, a member
/// or a plain name (names::is_known). Code::UnknownName for any other name.
fn name_as_written(cfg: &Config, name: &str) -> Result<String> {
    let unknown = || {
        Error::new(
            Code::UnknownName,
            format!("no person {name} in dossier.toml"),
        )
    };
    if !names::is_known(cfg, name) {
        return Err(unknown());
    }
    let wanted = name.trim().to_lowercase();
    cfg.people
        .iter()
        .map(|person| person.name.as_str())
        .chain(cfg.members.iter().map(String::as_str))
        .chain(cfg.names.iter().map(String::as_str))
        .find(|known| known.to_lowercase() == wanted)
        .map(str::to_string)
        .ok_or_else(unknown)
}

/// The confidential terms of the project that a team write comes from: None when `ctx` is not a
/// team write from a project. The project's code is read from its settings (conf::gate).
fn project_terms(ctx: &Ctx) -> Result<Option<Vec<String>>> {
    let Some(project) = &ctx.project else {
        return Ok(None);
    };
    let code = config::load(project)?.code;
    conf::gate(project, &code).map(Some)
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
    if let Some(source) = &plan.source {
        fields.push(pair("imported", "yes"));
        fields.push(pair("source-document", source));
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

/// The topics that the fields of `new` name: each value of the field `topic` (the key is compared
/// without regard to case), split at commas and trimmed; empty values are left out.
fn named_topics(new: &New) -> Vec<&str> {
    new.fields
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("topic"))
        .flat_map(|(_, value)| value.split(','))
        .map(str::trim)
        .filter(|topic| !topic.is_empty())
        .collect()
}

/// Checks the topics of an entry when the project has a list of topics (R-TOPIC-2, R-TOPIC-3,
/// D50). Each named topic must be on the list, ignoring case: Code::Usage otherwise. An LLM's
/// entry of a kind that needs a topic, not written from a document, must name one: Code::Refused.
fn check_topics(ctx: &Ctx, who: &Who, new: &New) -> Result<()> {
    let topics = &ctx.cfg.topics;
    if topics.is_empty() {
        return Ok(());
    }
    let named = named_topics(new);
    for topic in &named {
        if !topics.iter().any(|known| known.eq_ignore_ascii_case(topic)) {
            return Err(Error::new(
                Code::Usage,
                format!(
                    "unknown topic {topic}; the topics of this project: {} (dossier topics)",
                    topics.join(", ")
                ),
            ));
        }
    }
    let needs_topic = matches!(
        new.kind,
        Kind::Decision
            | Kind::Assumption
            | Kind::Finding
            | Kind::Lesson
            | Kind::Howto
            | Kind::Investigation
    );
    if who.is_llm && ctx.from_document.is_none() && needs_topic && named.is_empty() {
        return Err(Error::new(
            Code::Refused,
            format!(
                "an entry written by an LLM names its topic: --topic <one of {}>",
                topics.join(", ")
            ),
        ));
    }
    Ok(())
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
    use crate::blockfile::BlockFile;
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
            team: None,
            from_document: None,
            project: None,
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
    fn an_llm_decision_without_said_by_needs_a_person_unless_it_is_a_draft() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        let new = New {
            sections: decision_sections(),
            draft: false,
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let error = run(&ctx, &new).expect_err("the decision is refused");
        assert_eq!(error.code, Code::NeedsPerson);
        assert_eq!(
            error.message,
            "a decision needs the person who made it: --said-by <person>"
        );
        // Nothing was given out or written.
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("decisions")));
    }

    #[test]
    fn an_llm_draft_decision_without_said_by_is_a_proposal_of_its_person() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
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
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(entry.field("approved"), None);
        assert_eq!(entry.field("valid-from"), None);
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna via anna-claude: created"),
                format!("{time} anna via anna-claude: proposed"),
            ]
        );
        assert_eq!(
            journal_events(&ctx),
            vec![("created".to_string(), "proposed".to_string())]
        );
        let path = store::path_for(&ctx.dossier, Kind::Decision, &id, "Use the transient runs");
        assert_eq!(fs::read_to_string(path).expect("the file"), text);
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
    fn a_plain_name_can_be_the_person_a_decision_came_from() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.names = vec!["Dr. Weber".to_string()];
        // The name is matched without regard to case and written as the settings have it.
        let new = New {
            sections: decision_sections(),
            said_by: Some("dr. weber".to_string()),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (id, text, time) = run(&ctx, &new).expect("the decision is created");

        assert_eq!(id, "DEC-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("author").as_deref(), Some("Dr. Weber"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
        assert_eq!(
            entry.field("approved"),
            Some(format!("{time} by Dr. Weber"))
        );
        assert_eq!(
            entry.history(),
            vec![
                format!("{time} anna via anna-claude: created"),
                format!("{time} anna via anna-claude: approved"),
            ]
        );
    }

    #[test]
    fn a_member_named_with_said_by_is_the_author_as_the_settings_write_it() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.members = vec!["Tony".to_string()];
        let new = New {
            sections: decision_sections(),
            said_by: Some("TONY".to_string()),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let (_id, text, _time) = run(&ctx, &new).expect("the decision is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.field("author").as_deref(), Some("Tony"));
        assert_eq!(entry.field("via").as_deref(), Some("anna-claude"));
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
                pair("imported", "yes"),
                pair("source-document", "docs/Bericht.pdf"),
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
        assert_eq!(entry.field("imported"), None);
        assert_eq!(entry.field("source-document"), None);
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

    /// A how-to and a tool, each complete enough to approve (invented text): the ID it gets first
    /// and the new entry.
    fn guarded_entries() -> Vec<(&'static str, New)> {
        vec![
            (
                "HOW-001",
                New {
                    fields: vec![pair("software", "ExampleSolver")],
                    sections: vec![
                        pair("Purpose", "An invented purpose for a test."),
                        pair("Steps", "An invented step for a test."),
                    ],
                    ..new_of(Kind::Howto, "Run the invented solver")
                },
            ),
            (
                "TOOL-001",
                New {
                    fields: vec![pair("type", "script")],
                    sections: vec![pair("Purpose", "An invented purpose for a test.")],
                    ..new_of(Kind::Tool, "Invented solver script")
                },
            ),
        ]
    }

    #[test]
    fn an_entry_from_a_document_is_a_draft_whoever_writes_it() {
        let _guard = quiet();
        // The decider rule would approve this decision for anna, and an LLM writes it without
        // said_by: both are drafts, with the document they were drafted from (R-IMPORT-5).
        for (writer, via, label) in [
            ("anna", None, "anna"),
            ("anna-claude", Some("anna-claude"), "anna via anna-claude"),
        ] {
            let (_dir, mut ctx) = project(writer, false);
            ctx.cfg.deciders = vec!["anna".to_string()];
            ctx.from_document = Some("docs/Bericht Ä.pdf".to_string());
            let new = New {
                sections: decision_sections(),
                ..new_of(Kind::Decision, "Use the transient runs")
            };
            let (id, text, time) = run(&ctx, &new).expect("the imported decision is created");

            assert_eq!(id, "DEC-001");
            let entry = Entry::parse(&text).expect("the text is an entry");
            assert_eq!(entry.status(), Some(Status::Draft));
            assert_eq!(entry.field("author").as_deref(), Some("anna"));
            assert_eq!(entry.field("via").as_deref(), via);
            assert_eq!(entry.field("imported").as_deref(), Some("yes"));
            assert_eq!(
                entry.field("source-document").as_deref(),
                Some("docs/Bericht Ä.pdf")
            );
            assert_eq!(entry.field("approved"), None);
            assert_eq!(entry.field("valid-from"), None);
            assert_eq!(entry.history(), vec![format!("{time} {label}: created")]);
            assert_eq!(
                journal_events(&ctx),
                vec![("created".to_string(), "draft".to_string())]
            );
            assert!(!ctx.state.join("approved.json").exists());
            let path = store::path_for(&ctx.dossier, Kind::Decision, &id, "Use the transient runs");
            assert_eq!(fs::read_to_string(path).expect("the file"), text);
        }
    }

    #[test]
    fn an_imported_finding_is_a_draft_though_a_finding_is_approved_at_once() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna", false);
        ctx.from_document = Some("docs/Bericht Ä.pdf".to_string());
        let new = New {
            sections: vec![pair("Observation", "An invented observation for a test.")],
            ..new_of(Kind::Finding, "Invented observation")
        };
        let (id, text, _time) = run(&ctx, &new).expect("the imported finding is created");

        assert_eq!(id, "FND-001");
        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Draft));
        assert_eq!(entry.field("approved"), None);
        assert_eq!(entry.field("imported").as_deref(), Some("yes"));
        assert_eq!(
            entry.field("source-document").as_deref(),
            Some("docs/Bericht Ä.pdf")
        );
    }

    #[test]
    fn an_approved_how_to_or_tool_by_a_person_records_its_checksum() {
        let _guard = quiet();
        for (id_wanted, new) in guarded_entries() {
            let (_dir, ctx) = project("anna", false);
            let (id, text, _time) = run(&ctx, &new).expect("the entry is created");

            assert_eq!(id, id_wanted);
            let entry = Entry::parse(&text).expect("the text is an entry");
            assert_eq!(entry.status(), Some(Status::Approved));
            assert!(entry.field("approved").is_some());
            assert!(ctx.state.join("approved.json").exists());
            assert!(matches!(
                approval::mark(&ctx.state, &id, text.as_bytes()),
                approval::Mark::Unchanged
            ));
            // A change to the text after approval is a change.
            let changed = text.replace("An invented purpose for a test.", "An invented purpose.");
            assert_ne!(changed, text);
            assert!(matches!(
                approval::mark(&ctx.state, &id, changed.as_bytes()),
                approval::Mark::Changed
            ));
        }
    }

    #[test]
    fn an_llm_how_to_or_tool_without_said_by_is_a_draft_and_is_not_recorded() {
        let _guard = quiet();
        for (id_wanted, new) in guarded_entries() {
            let (_dir, ctx) = project("anna-claude", false);
            let (id, text, time) = run(&ctx, &new).expect("the draft is created");

            assert_eq!(id, id_wanted);
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
            assert!(!ctx.state.join("approved.json").exists());
        }
    }

    /// The topics of the project in the topic tests (invented).
    fn test_topics() -> Vec<String> {
        vec!["pumps".to_string(), "rig".to_string()]
    }

    /// The sections that make a finding complete (invented text).
    fn observation() -> Vec<(String, String)> {
        vec![pair("Observation", "An invented observation for a test.")]
    }

    #[test]
    fn with_topics_an_llms_finding_without_a_topic_is_refused() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.topics = test_topics();
        let new = New {
            sections: observation(),
            ..new_of(Kind::Finding, "Invented observation")
        };
        let error = run(&ctx, &new).expect_err("the finding names no topic");

        assert_eq!(error.code, Code::Refused);
        assert_eq!(
            error.message,
            "an entry written by an LLM names its topic: --topic <one of pumps, rig>"
        );
        // Refused before anything is queued: no ID is given out and no entry is written.
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join(Kind::Finding.folder())));
    }

    #[test]
    fn with_topics_an_llms_finding_with_a_listed_topic_is_written_as_given() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.topics = test_topics();
        let new = New {
            fields: vec![pair("topic", "Pumps")],
            sections: observation(),
            ..new_of(Kind::Finding, "Invented observation")
        };
        let (_id, text, _time) = run(&ctx, &new).expect("the finding is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.field("topic").as_deref(), Some("Pumps"));
    }

    #[test]
    fn with_topics_an_unknown_topic_is_a_usage_error_for_a_person_and_an_llm() {
        let _guard = quiet();
        for writer in ["anna", "anna-claude"] {
            let (_dir, mut ctx) = project(writer, false);
            ctx.cfg.topics = test_topics();
            let new = New {
                fields: vec![pair("topic", "valves")],
                sections: observation(),
                ..new_of(Kind::Finding, "Invented observation")
            };
            let error = run(&ctx, &new).expect_err("valves is not a topic");

            assert_eq!(error.code, Code::Usage);
            assert_eq!(
                error.message,
                "unknown topic valves; the topics of this project: pumps, rig (dossier topics)"
            );
            assert!(dir_is_empty(&ctx.dossier.join(Kind::Finding.folder())));
        }
    }

    #[test]
    fn with_topics_a_persons_finding_without_a_topic_is_written() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna", false);
        ctx.cfg.topics = test_topics();
        let new = New {
            sections: observation(),
            ..new_of(Kind::Finding, "Invented observation")
        };
        let (_id, text, _time) = run(&ctx, &new).expect("the finding is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Approved));
        assert_eq!(entry.field("topic"), None);
    }

    #[test]
    fn with_topics_an_llms_tool_entry_without_a_topic_is_written() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.topics = test_topics();
        let new = new_of(Kind::Tool, "Invented solver script");
        let (_id, text, _time) = run(&ctx, &new).expect("the tool entry is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Draft));
        assert_eq!(entry.field("topic"), None);
    }

    #[test]
    fn with_topics_an_llms_finding_from_a_document_without_a_topic_is_written() {
        let _guard = quiet();
        let (_dir, mut ctx) = project("anna-claude", false);
        ctx.cfg.topics = test_topics();
        ctx.from_document = Some("docs/Bericht Ä.pdf".to_string());
        let new = New {
            sections: observation(),
            ..new_of(Kind::Finding, "Invented observation")
        };
        let (_id, text, _time) = run(&ctx, &new).expect("the imported finding is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.status(), Some(Status::Draft));
        assert_eq!(entry.field("topic"), None);
    }

    #[test]
    fn without_topics_in_the_settings_an_llms_finding_without_a_topic_is_written() {
        let _guard = quiet();
        let (_dir, ctx) = project("anna-claude", false);
        assert!(ctx.cfg.topics.is_empty());
        let new = New {
            sections: observation(),
            ..new_of(Kind::Finding, "Invented observation")
        };
        let (_id, text, _time) = run(&ctx, &new).expect("the finding is created");

        let entry = Entry::parse(&text).expect("the text is an entry");
        assert_eq!(entry.field("topic"), None);
    }

    /// A team dossier as `project` makes it, and a project of the code P-001 written from it. The
    /// project's DOSSIER folder holds `terms` as its confidential.md, or no file when `terms` is
    /// None. The context is the team's, and `ctx.project` is the project's DOSSIER folder, as a
    /// team write from a project has it. The folder names have a space and a non-ASCII letter.
    fn team_write_from(
        writer: &str,
        terms: Option<String>,
    ) -> (tempfile::TempDir, tempfile::TempDir, Ctx) {
        let (team, mut ctx) = project(writer, false);
        let folder = tempfile::Builder::new()
            .prefix("Projekt Öl ")
            .tempdir()
            .expect("temporary folder");
        let dossier = folder.path().join("DOSSIER");
        fs::create_dir_all(&dossier).expect("the project DOSSIER folder");
        fs::write(dossier.join("dossier.toml"), config::default_text("P-001"))
            .expect("dossier.toml");
        if let Some(text) = terms {
            fs::write(dossier.join("confidential.md"), text).expect("confidential.md");
        }
        ctx.project = Some(dossier);
        (team, folder, ctx)
    }

    /// A project's confidential.md, built as BlockFile text: a block for each term, with a
    /// history line.
    fn terms_text(terms: &[&str]) -> String {
        let mut file = BlockFile::empty("Confidential terms of P-001");
        for term in terms {
            file.add_block(
                term,
                &[pair("history", "2026-11-02T09:00:00+01:00 anna: added")],
            );
        }
        file.text()
    }

    /// A project's confidential.md in which a person stated that there are no terms.
    fn none_text() -> String {
        let mut file = BlockFile::empty("Confidential terms of P-001");
        file.add_block(
            conf::NONE,
            &[pair("stated", "2026-11-02T09:00:00+01:00 by anna")],
        );
        file.text()
    }

    #[test]
    fn a_team_write_from_a_project_with_a_term_in_the_title_is_refused() {
        let _guard = quiet();
        let (_team, _folder, ctx) = team_write_from("anna", Some(terms_text(&["Nordwind"])));
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Nordwind pilot")
        };
        let error = run(&ctx, &new).expect_err("the title holds a term");
        assert_eq!(error.code, Code::Refused);
        assert_eq!(error.message, "confidential terms found: Nordwind (line 2)");
        // Refused before anything is queued: no ID is given out and no entry is written.
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("decisions")));
    }

    #[test]
    fn a_team_write_from_a_project_with_a_term_in_the_text_is_refused() {
        let _guard = quiet();
        let (_team, _folder, ctx) = team_write_from("anna", Some(terms_text(&["Nordwind"])));
        let new = New {
            sections: vec![
                pair("Decision", "The Nordwind pilot goes ahead."),
                pair("Reason", "An invented reason for a test."),
            ],
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let error = run(&ctx, &new).expect_err("the text holds a term");
        assert_eq!(error.code, Code::Refused);
        assert!(
            error
                .message
                .starts_with("confidential terms found: Nordwind (line "),
            "{}",
            error.message
        );
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("decisions")));
    }

    #[test]
    fn a_team_write_from_a_project_without_terms_or_a_statement_is_refused() {
        let _guard = quiet();
        let (_team, _folder, ctx) = team_write_from("anna", None);
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Use the transient runs")
        };
        let error = run(&ctx, &new).expect_err("the project has no terms and no statement");
        assert_eq!(error.code, Code::Refused);
        assert!(
            error
                .message
                .starts_with("this project has no confidential terms yet"),
            "{}",
            error.message
        );
        assert!(!ctx.state.join("ids.json").exists());
        assert!(dir_is_empty(&ctx.dossier.join("decisions")));
    }

    #[test]
    fn a_team_write_from_a_project_is_written_when_no_term_is_in_it() {
        let _guard = quiet();
        // A project that states it has no terms, and a project whose term is not in the entry.
        for terms in [none_text(), terms_text(&["Nordwind"])] {
            let (_team, _folder, ctx) = team_write_from("anna", Some(terms));
            let new = New {
                sections: decision_sections(),
                ..new_of(Kind::Decision, "Use the transient runs")
            };
            let (id, text, _time) = run(&ctx, &new).expect("the decision is created");

            assert_eq!(id, "DEC-001");
            let entry = Entry::parse(&text).expect("the text is an entry");
            assert_eq!(entry.status(), Some(Status::Approved));
            let path = store::path_for(&ctx.dossier, Kind::Decision, &id, "Use the transient runs");
            assert_eq!(fs::read_to_string(path).expect("the file"), text);
        }
    }

    #[test]
    fn a_project_write_is_not_checked_against_confidential_terms() {
        let _guard = quiet();
        // Only a team write from a project is checked: a project's own entry may hold any term.
        let (_dir, ctx) = project("anna", false);
        assert!(ctx.project.is_none());
        let new = New {
            sections: decision_sections(),
            ..new_of(Kind::Decision, "Nordwind pilot")
        };
        let (id, _text, _time) = run(&ctx, &new).expect("the decision is created");
        assert_eq!(id, "DEC-001");
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
