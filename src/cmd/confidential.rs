//! The `confidential` command (docs/design.md section 12, format.md §24): the confidential terms of
//! the project. Anyone in the project may add a term, an LLM too (D32, R-CONF-1), and `list` shows
//! the terms and whether a person stated there are none. A person at a terminal removes a term or
//! states that there are none (D32, R-CONF-5). Scanning comes later. The terms belong to the
//! project, so `--team` is refused.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::conf;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::names::Who;
use crate::output::Output;
use crate::term;
use crate::tx::{self, Change};
use serde_json::json;

/// The file of the project dossier that holds the terms (format.md §24).
const FILE: &str = "confidential.md";

/// The ID of the terms file in the journal and the snapshots.
const ITEM: &str = "CONFIDENTIAL";

/// What `--team` gets: the terms are the project's, not the team's.
const TEAM_REFUSED: &str = "confidential terms belong to a project; run without --team";

/// The arguments of `confidential`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `confidential` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a confidential term of the project; anyone may add one, also an LLM
    Add {
        /// The term, such as NT-450 or a customer's name
        term: String,
    },
    /// Removes a confidential term of the project; a person at a terminal
    Remove {
        /// The term as listed; any spelling of its separators and case finds it
        term: String,
    },
    /// Lists the confidential terms, and whether a person stated there are none
    List,
    /// States that the project has no confidential terms; a person at a terminal, with --yes
    None {
        /// Confirms the statement; without it nothing is written
        #[arg(long)]
        yes: bool,
    },
}

/// Runs `confidential`. `--team` is refused before anything is loaded, for every subcommand.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    if g.team {
        return Err(Error::new(Code::Usage, TEAM_REFUSED));
    }
    let ctx = Ctx::load(g, "confidential")?;
    match &args.action {
        Action::Add { term } => add(&ctx, term),
        Action::Remove { term } => remove(&ctx, term),
        Action::List => list(&ctx),
        Action::None { yes } => none(&ctx, *yes),
    }
}

/// Adds a term. A blank term, a term with a line break, and the statement that there are none are
/// refused before anything is written. A term already listed, in any spelling of its separators
/// and case (see `same_term`), is refused (Code::Exists). A removed term is added again by taking
/// its `removed` field away. Every addition gets a history line with who and when, and a journal
/// line.
fn add(ctx: &Ctx, term: &str) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let term = term.trim();
    if conf::normalize(term).is_empty() {
        return Err(bad("the term is empty"));
    }
    if term.contains(['\r', '\n']) {
        return Err(bad("a confidential term is one line"));
    }
    if same_term(term, conf::NONE) {
        return Err(bad(format!(
            "{term} is the statement that there are none, not a term"
        )));
    }

    tx::write(ctx, |tx| {
        let (mut file, old) = conf::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        if let Some(key) = conf::terms(&file).iter().find(|key| same_term(key, term)) {
            return Err(Error::new(
                Code::Exists,
                format!("{key} is already a confidential term"),
            ));
        }
        let now = tx.time().to_string();
        let stored = match file.keys().into_iter().find(|key| same_term(key, term)) {
            Some(key) => {
                file.set_field(&key, "removed", "");
                key
            }
            None => {
                file.add_block(term, &[]);
                term.to_string()
            }
        };
        file.add_history(&stored, &now, &who, "added", "");
        tx.put_raw(
            tx.ctx().dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: "added".to_string(),
                detail: stored.clone(),
            },
        );
        Ok(Output::with(
            format!("{stored}: added"),
            json!({"term": stored, "status": "added"}),
        ))
    })
}

/// Removes a term, for a person at a terminal (D32). The term is found in any spelling of its
/// separators and case, as `add` finds it. A term that is not listed, or is listed and already
/// removed, is Code::NotFound and nothing is written. The block keeps its history and gets the
/// field `removed: <time> by <person>` and a history line `removed` (format.md §24).
fn remove(ctx: &Ctx, term: &str) -> Result<Output> {
    let who = acting_person(ctx, "confidential remove")?;
    let term = term.trim();
    if conf::normalize(term).is_empty() {
        return Err(bad("the term is empty"));
    }

    tx::write(ctx, |tx| {
        let (mut file, old) = conf::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let key = listed_key(&file, term)?;
        let now = tx.time().to_string();
        file.set_field(&key, "removed", &format!("{now} by {}", who.person));
        file.add_history(&key, &now, &who, "removed", "");
        tx.put_raw(
            tx.ctx().dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: "removed".to_string(),
                detail: key.clone(),
            },
        );
        Ok(Output::with(
            format!("{key}: removed"),
            json!({"term": key, "status": "removed"}),
        ))
    })
}

/// Lists the terms, one per line. Then, when no term is listed and a person stated there are none,
/// the statement with who and when; otherwise the count.
fn list(ctx: &Ctx) -> Result<Output> {
    let (file, _) = conf::load(&ctx.dossier, &ctx.cfg.code)?;
    let terms = conf::terms(&file);
    let stated = conf::stated_none(&file);
    let data = json!({"terms": terms, "none_stated": stated});
    let mut lines = terms.clone();
    if terms.is_empty() && stated {
        let value = file.field(conf::NONE, "stated").unwrap_or_default();
        lines.push(stated_line(&value));
    } else {
        lines.push(count(terms.len()));
    }
    Ok(Output::with(lines.join("\n"), data))
}

/// States that the project has no confidential terms (R-CONF-5, format.md §24), for a person at a
/// terminal with `--yes`. Refused while a term is listed. The block `(no confidential terms)` gets
/// the field `stated: <time> by <person>` and a history line `stated`; stating again moves the
/// time on.
fn none(ctx: &Ctx, yes: bool) -> Result<Output> {
    let who = acting_person(ctx, "confidential none")?;
    if !yes {
        return Err(Error::new(
            Code::Usage,
            "confidential none states that there are none; add --yes",
        ));
    }

    tx::write(ctx, |tx| {
        let (mut file, old) = conf::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        if !conf::terms(&file).is_empty() {
            return Err(Error::new(
                Code::Refused,
                "this project has confidential terms; remove the terms first",
            ));
        }
        let now = tx.time().to_string();
        file.add_block(conf::NONE, &[]);
        file.set_field(conf::NONE, "stated", &format!("{now} by {}", who.person));
        file.add_history(conf::NONE, &now, &who, "stated", "");
        tx.put_raw(
            tx.ctx().dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: "stated".to_string(),
                detail: conf::NONE.to_string(),
            },
        );
        Ok(Output::with(
            format!("{}: stated", conf::NONE),
            json!({"term": conf::NONE, "status": "stated"}),
        ))
    })
}

/// The name that acts, when it is a person at a terminal. An LLM is Code::NeedsPerson, and a run
/// with no terminal to answer at is Code::NeedsTerminal (D32). `what` names the subcommand in the
/// refusal.
fn acting_person(ctx: &Ctx, what: &str) -> Result<Who> {
    let who = ctx.writer()?.clone();
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("{what} is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    Ok(who)
}

/// The key of the listed term that `term` names, in any spelling (`same_term`). Code::NotFound
/// when no listed term is: the message says whether the term is removed, or is not a term at all.
fn listed_key(file: &BlockFile, term: &str) -> Result<String> {
    if let Some(key) = conf::terms(file)
        .into_iter()
        .find(|key| same_term(key, term))
    {
        return Ok(key);
    }
    let removed = file
        .keys()
        .into_iter()
        .find(|key| key.as_str() != conf::NONE && same_term(key, term));
    Err(match removed {
        Some(key) => Error::new(Code::NotFound, format!("{key} is already removed")),
        None => Error::new(Code::NotFound, format!("{term} is not a confidential term")),
    })
}

/// Whether two terms are the same term: equal after `conf::normalize`, or equal once every white
/// space, '-' and '_' is removed from the lower-cased text. This is the comparison of `conf::find`
/// (format.md §24), so that a term refused here is one the check would find in the text.
fn same_term(a: &str, b: &str) -> bool {
    conf::normalize(a) == conf::normalize(b) || packed(a) == packed(b)
}

/// Lower case with every white space, '-' and '_' removed: the spelling of a term without its
/// separators. The same rule as `conf::find` uses, which keeps it private.
fn packed(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|&c| !(c.is_whitespace() || c == '-' || c == '_'))
        .collect()
}

/// The statement that there are none, with who and when, from the field `<time> by <person>`
/// (format.md §24): `no confidential terms stated by anna at <time>`.
fn stated_line(value: &str) -> String {
    match value.split_once(" by ") {
        Some((when, who)) => format!("no confidential terms stated by {who} at {when}"),
        None => format!("no confidential terms stated at {value}"),
    }
}

/// `1 term`, or `<n> terms`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 term".to_string()
    } else {
        format!("{n} terms")
    }
}

/// A failure for a term the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
