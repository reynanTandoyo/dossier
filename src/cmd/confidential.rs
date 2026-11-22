//! The `confidential` command (docs/design.md section 12, format.md §24): the confidential terms
//! of the project. Anyone in the project may add a term, an LLM too (D32, R-CONF-1), and `list`
//! shows the terms and whether a person stated there are none. Removing, stating none and scanning
//! come later.

use crate::cli::Global;
use crate::conf;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::tx::{self, Change};
use serde_json::json;

/// The file of the project dossier that holds the terms (format.md §24).
const FILE: &str = "confidential.md";

/// The ID of the terms file in the journal and the snapshots.
const ITEM: &str = "CONFIDENTIAL";

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
    /// Lists the confidential terms, and whether a person stated there are none
    List,
}

/// Runs `confidential`: add or list.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "confidential")?;
    match &args.action {
        Action::Add { term } => add(&ctx, term),
        Action::List => list(&ctx),
    }
}

/// Adds a term. A blank term, a term with a line break, and the statement that there are none are
/// refused before anything is written. A term already listed, in any spelling of its separators
/// and case, is refused (Code::Exists). A removed term is added again by taking its `removed`
/// field away. Every addition gets a history line with who and when, and a journal line.
fn add(ctx: &Ctx, term: &str) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let term = term.trim();
    if conf::normalize(term).is_empty() {
        return Err(bad("the term is empty"));
    }
    if term.contains(['\r', '\n']) {
        return Err(bad("a confidential term is one line"));
    }
    if conf::normalize(term) == conf::normalize(conf::NONE) {
        return Err(bad(format!(
            "{term} is the statement that there are none, not a term"
        )));
    }

    tx::write(ctx, |tx| {
        let (mut file, old) = conf::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let now = tx.time().to_string();
        let wanted = conf::normalize(term);
        let stored = match file
            .keys()
            .into_iter()
            .find(|key| conf::normalize(key) == wanted)
        {
            Some(key) if conf::terms(&file).contains(&key) => {
                return Err(Error::new(
                    Code::Exists,
                    format!("{key} is already a confidential term"),
                ));
            }
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
