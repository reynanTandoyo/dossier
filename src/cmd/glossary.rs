//! The `glossary` command (docs/design.md section 12, R-TERM-1): every term of the project's
//! glossary and of the team's, one line each, sorted by term ignoring case, then `<n> terms`. A
//! team term ends in `[team]`. With `--team` the team dossier is the one read, so only its terms
//! are listed.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::Result;
use crate::glossary::{self, Term};
use crate::output::Output;
use serde_json::json;
use std::path::Path;

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The arguments of `glossary`.
#[derive(clap::Args, Debug)]
pub struct Args {}

/// A term as the listing shows it, and whether it is in the team's glossary.
struct Shown {
    term: Term,
    team: bool,
}

impl Shown {
    /// `<term>`, then ` (<symbol>, <unit>)` with the parts that are there, then `: <meaning>`,
    /// then `  [also: <spellings>]`, then `  [team]` for a team term. An empty part is left out.
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
}

/// Runs `glossary`: the terms of the project's glossary and the team's, sorted by term ignoring
/// case (a project term before a team term that differs only in case), then the count. A project
/// whose team dossier cannot be reached lists its own terms and gives a warning.
pub fn run(_args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "glossary")?;
    let mut shown: Vec<Shown> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    if ctx.cfg.kind == TEAM {
        // The team dossier itself, or `--team` from a project: its terms only.
        shown.extend(read(&ctx.dossier, &ctx.cfg.code, true)?);
    } else {
        shown.extend(read(&ctx.dossier, &ctx.cfg.code, false)?);
        if ctx.team.is_some() {
            let team = ctx.team_ctx()?;
            shown.extend(read(&team.dossier, &team.cfg.code, true)?);
        } else if let Some(stored) = &ctx.cfg.team {
            warnings.push(format!(
                "{stored}: the team dossier cannot be reached, so its terms are not listed"
            ));
        }
    }
    shown.sort_by(|a, b| {
        a.term
            .term
            .to_lowercase()
            .cmp(&b.term.term.to_lowercase())
            .then(a.team.cmp(&b.team))
    });

    let mut lines: Vec<String> = shown.iter().map(Shown::line).collect();
    lines.push(count(shown.len()));
    let data = json!({
        "count": shown.len(),
        "terms": shown.iter().map(Shown::data).collect::<Vec<_>>(),
    });
    let mut output = Output::with(lines.join("\n"), data);
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

/// `1 term`, or `<n> terms`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 term".to_string()
    } else {
        format!("{n} terms")
    }
}
