//! The `term` command (docs/design.md section 12, R-TERM-1, format.md §20): adds a term to the
//! glossary, or looks terms up. `term add <term> <meaning> [--symbol] [--unit] [--also <list>]`
//! writes the glossary of the current dossier, the team's with `--team`. A team write from a
//! project is checked against the project's confidential terms first (R-TEAM-3, R-CONF-2). A
//! spelling names one term: a term or other spelling already in the glossary is refused
//! (Code::Exists). `term <word>` prints the terms of the project and the team whose term, other
//! spellings or meaning contain the word, ignoring case; a team term ends in `[team]`.

use crate::cli::Global;
use crate::conf;
use crate::config;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::glossary::{self, Term};
use crate::names::Who;
use crate::output::Output;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The file of a dossier that holds the glossary (format.md §20).
const FILE: &str = "glossary.md";

/// The ID of the glossary file in the journal and the snapshots.
const ITEM: &str = "GLOSSARY";

/// What the command says when it is given neither a word nor `add`.
const NO_WORD: &str = "give a word to look up, or add a term: dossier term add <term> <meaning>";

/// The arguments of `term`: a word to look up, or the subcommand `add`.
#[derive(clap::Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
    /// A word to look up: a term, another spelling of it, or a word of its meaning
    pub word: Option<String>,
}

/// What `term` does besides looking a word up.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a term to the glossary of this dossier (the team's with --team)
    Add {
        /// The term, such as "mass flow"
        term: String,
        /// What the term means
        meaning: String,
        /// The symbol of the term, such as m-dot
        #[arg(long, value_name = "text")]
        symbol: Option<String>,
        /// The unit of the term, such as kg/s
        #[arg(long, value_name = "text")]
        unit: Option<String>,
        /// Other spellings of the term, separated by commas, such as "Massenstrom, mdot"
        #[arg(long, value_name = "list")]
        also: Option<String>,
    },
}

/// A term as the command gives it, checked: the text of each field and its other spellings.
struct Addition {
    term: String,
    meaning: String,
    symbol: Option<String>,
    unit: Option<String>,
    also: Vec<String>,
}

impl Addition {
    /// The term as given, with every text trimmed. The term and the meaning are required; a
    /// symbol or unit that is blank is left out. A text that spans lines is Code::BadValue.
    fn new(
        term: &str,
        meaning: &str,
        symbol: Option<&str>,
        unit: Option<&str>,
        also: Option<&str>,
    ) -> Result<Addition> {
        let term = one_line(term, "the term")?;
        Ok(Addition {
            meaning: one_line(meaning, "the meaning")?,
            symbol: optional(symbol, "the symbol")?,
            unit: optional(unit, "the unit")?,
            also: spellings(&term, also)?,
            term,
        })
    }

    /// The fields of the new block, in the order of format.md §20; `also` is the comma list.
    fn fields(&self) -> Vec<(String, String)> {
        let mut fields = vec![("meaning".to_string(), self.meaning.clone())];
        if let Some(symbol) = &self.symbol {
            fields.push(("symbol".to_string(), symbol.clone()));
        }
        if let Some(unit) = &self.unit {
            fields.push(("unit".to_string(), unit.clone()));
        }
        if !self.also.is_empty() {
            fields.push(("also".to_string(), self.also.join(", ")));
        }
        fields
    }
}

/// A term as a lookup shows it, and whether it is in the team's glossary.
struct Shown {
    term: Term,
    team: bool,
}

impl Shown {
    /// `<term>`, then ` (<symbol>, <unit>)` with the parts that are there, then `: <meaning>`,
    /// then `  [also: <spellings>]`, then `  [team]` for a team term. An empty part is left out.
    /// The same line as `glossary` prints.
    fn line(&self) -> String {
        let term = &self.term;
        let mut line = term.term.clone();
        let facts: Vec<&str> = [term.symbol.as_deref(), term.unit.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        if !facts.is_empty() {
            line.push_str(&format!(" ({})", facts.join(", ")));
        }
        if !term.meaning.is_empty() {
            line.push_str(&format!(": {}", term.meaning));
        }
        if !term.also.is_empty() {
            line.push_str(&format!("  [also: {}]", term.also.join(", ")));
        }
        if self.team {
            line.push_str("  [team]");
        }
        line
    }

    /// The term as JSON: a value it does not have is `null`, and `also` is a list.
    fn data(&self) -> serde_json::Value {
        let term = &self.term;
        json!({
            "term": term.term,
            "meaning": (!term.meaning.is_empty()).then_some(term.meaning.as_str()),
            "symbol": term.symbol,
            "unit": term.unit,
            "also": term.also,
            "team": self.team,
        })
    }

    /// Whether the term, one of its other spellings or its meaning contains `needle`, which is
    /// lower case, ignoring case.
    fn matches(&self, needle: &str) -> bool {
        let term = &self.term;
        std::iter::once(term.term.as_str())
            .chain(term.also.iter().map(String::as_str))
            .chain(std::iter::once(term.meaning.as_str()))
            .any(|text| text.to_lowercase().contains(needle))
    }
}

/// Runs `term`: adds a term, or looks up a word. A lookup needs no name, so it is not refused
/// for a name that is not in the settings: it reads the glossary as the `glossary` command does.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    match (&args.action, &args.word) {
        (
            Some(Action::Add {
                term,
                meaning,
                symbol,
                unit,
                also,
            }),
            _,
        ) => {
            let ctx = Ctx::load(g, "term")?;
            let who = ctx.writer()?.clone();
            let addition = Addition::new(
                term,
                meaning,
                symbol.as_deref(),
                unit.as_deref(),
                also.as_deref(),
            )?;
            add(&ctx, &who, &addition)
        }
        (None, Some(word)) => {
            let word = word.trim();
            if word.is_empty() {
                return Err(Error::new(Code::Usage, NO_WORD));
            }
            let ctx = Ctx::load(g, "glossary")?;
            lookup(&ctx, word)
        }
        (None, None) => Err(Error::new(Code::Usage, NO_WORD)),
    }
}

/// Adds the term to the glossary of the context's dossier in one transaction. A team write from a
/// project first runs conf::gate and conf::check on the new block (R-TEAM-3). A term or spelling
/// that the glossary has already is Code::Exists. The new block gets its fields and the history
/// line `added`, with who and when.
fn add(ctx: &Ctx, who: &Who, addition: &Addition) -> Result<Output> {
    let fields = addition.fields();
    tx::write(ctx, |tx| {
        let project = tx.ctx().project.clone();
        if let Some(project) = project {
            let code = config::load(&project)?.code;
            let terms = conf::gate(&project, &code)?;
            conf::check(&terms, &block_text(addition, &fields))?;
        }

        let dossier = tx.ctx().dossier.clone();
        let code = tx.ctx().cfg.code.clone();
        let team = tx.ctx().cfg.kind == TEAM;
        let (mut file, old) = glossary::load(&dossier, &code)?;
        if let Some(message) = clash(&glossary::terms(&file), addition) {
            return Err(Error::new(Code::Exists, message));
        }
        let now = tx.time().to_string();
        file.add_block(&addition.term, &fields);
        file.add_history(&addition.term, &now, who, "added", "");
        tx.put_raw(
            dossier.join(FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: ITEM.to_string(),
                event: "added".to_string(),
                detail: addition.term.clone(),
            },
        );
        let data = json!({
            "term": addition.term,
            "status": "added",
            "team": team,
            "meaning": addition.meaning,
            "symbol": addition.symbol,
            "unit": addition.unit,
            "also": addition.also,
        });
        Ok(Output::with(format!("{}: added", addition.term), data))
    })
}

/// Prints the terms of the project's glossary and the team's that contain `word`, sorted by term
/// ignoring case (a project term before a team term that differs only in case). A team term ends
/// in `[team]`. Nothing found is the text `no term matches <word>`, not an error. With `--team`
/// the team dossier is the one read. A project whose team dossier cannot be reached gets a warning.
fn lookup(ctx: &Ctx, word: &str) -> Result<Output> {
    let needle = word.to_lowercase();
    let mut shown: Vec<Shown> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    if ctx.cfg.kind == TEAM {
        shown.extend(read(&ctx.dossier, &ctx.cfg.code, true)?);
    } else {
        shown.extend(read(&ctx.dossier, &ctx.cfg.code, false)?);
        if ctx.team.is_some() {
            let team = ctx.team_ctx()?;
            shown.extend(read(&team.dossier, &team.cfg.code, true)?);
        } else if let Some(stored) = &ctx.cfg.team {
            warnings.push(format!(
                "{stored}: the team dossier cannot be reached, so its terms are not searched"
            ));
        }
    }
    shown.retain(|shown| shown.matches(&needle));
    shown.sort_by(|a, b| {
        a.term
            .term
            .to_lowercase()
            .cmp(&b.term.term.to_lowercase())
            .then(a.team.cmp(&b.team))
    });

    let text = if shown.is_empty() {
        format!("no term matches {word}")
    } else {
        shown.iter().map(Shown::line).collect::<Vec<_>>().join("\n")
    };
    let data = json!({
        "word": word,
        "count": shown.len(),
        "terms": shown.iter().map(Shown::data).collect::<Vec<_>>(),
    });
    let mut output = Output::with(text, data);
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// The terms of the glossary of `dossier` in file order, each marked as the team's when `team`
/// is true. `code` names the file when it is missing.
fn read(dossier: &Path, code: &str, team: bool) -> Result<Vec<Shown>> {
    let (file, _) = glossary::load(dossier, code)?;
    Ok(glossary::terms(&file)
        .into_iter()
        .map(|term| Shown { term, team })
        .collect())
}

/// The new block as the confidential check reads it: its heading line and its field lines, so
/// that a term found in any field, the term itself included, is found on its own line.
fn block_text(addition: &Addition, fields: &[(String, String)]) -> String {
    std::iter::once(format!("## {}", addition.term))
        .chain(
            fields
                .iter()
                .map(|(name, value)| format!("- {name}: {value}")),
        )
        .collect::<Vec<_>>()
        .join("\n")
}

/// The message of Code::Exists when the new term or one of its spellings is a term of the glossary
/// already, or a spelling of one: a spelling names one term. None when all of them are new.
fn clash(known: &[Term], addition: &Addition) -> Option<String> {
    let spellings =
        std::iter::once(addition.term.as_str()).chain(addition.also.iter().map(String::as_str));
    for spelling in spellings {
        for existing in known {
            if same(&existing.term, spelling) {
                return Some(format!("{spelling} is already a term of the glossary"));
            }
            if existing.also.iter().any(|other| same(other, spelling)) {
                return Some(format!(
                    "{spelling} is already a spelling of {}",
                    existing.term
                ));
            }
        }
    }
    None
}

/// Two spellings are the same when they are equal ignoring surrounding spaces and case.
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// The text trimmed: Code::BadValue when it is empty or spans lines (a term is one line).
fn one_line(text: &str, what: &str) -> Result<String> {
    let text = text.trim();
    if text.contains(['\r', '\n']) {
        return Err(bad(format!("{what} is one line")));
    }
    if text.is_empty() {
        return Err(bad(format!("{what} is empty")));
    }
    Ok(text.to_string())
}

/// The optional text trimmed; a missing or blank one is None.
fn optional(text: Option<&str>, what: &str) -> Result<Option<String>> {
    match text {
        Some(text) if !text.trim().is_empty() => one_line(text, what).map(Some),
        _ => Ok(None),
    }
}

/// The other spellings of a term from its comma list: each trimmed, without empty ones, without
/// the term itself and without repeats, ignoring case.
fn spellings(term: &str, list: Option<&str>) -> Result<Vec<String>> {
    let mut found: Vec<String> = Vec::new();
    for part in list.unwrap_or_default().split(',') {
        if part.trim().is_empty() {
            continue;
        }
        let spelling = one_line(part, "a spelling")?;
        if !same(&spelling, term) && !found.iter().any(|known| same(known, &spelling)) {
            found.push(spelling);
        }
    }
    Ok(found)
}

/// A failure for a term the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
