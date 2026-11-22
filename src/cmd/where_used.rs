//! The `where-used` command (docs/design.md section 12, R-USE-4): where and how a software was used
//! in the team's projects, read from the team's usage index (format.md §20). Each block of the
//! index is one run; this command only reads them. Nothing is written.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::paths;
use crate::teamfiles;
use serde_json::json;

/// The `kind` of a team dossier (format.md §26).
const TEAM: &str = "team";

/// The arguments of `where-used`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The software, by name; any case
    pub software: String,
    /// Only the uses whose analysis contains this text; any case
    #[arg(long)]
    pub analysis: Option<String>,
    /// Only the uses whose machine contains this text; any case
    #[arg(long)]
    pub machine: Option<String>,
}

/// One block of the usage index (format.md §20). A value the block does not have, or has empty,
/// is None.
struct Usage {
    key: String,
    date: Option<String>,
    code: Option<String>,
    software: Option<String>,
    version: Option<String>,
    analysis: Option<String>,
    machine: Option<String>,
    howto: Option<String>,
    person: Option<String>,
}

impl Usage {
    /// The block `key` of the index as it is written.
    fn read(file: &BlockFile, key: &str) -> Usage {
        let value = |name: &str| file.field(key, name).filter(|value| !value.is_empty());
        Usage {
            key: key.to_string(),
            date: value("date"),
            code: value("code"),
            software: value("software"),
            version: value("version"),
            analysis: value("analysis"),
            machine: value("machine"),
            howto: value("howto"),
            person: value("person"),
        }
    }

    /// The date, `YYYY-MM-DD`; empty when the block has none.
    fn date(&self) -> &str {
        self.date.as_deref().unwrap_or("")
    }

    /// Whether the software is the one named, and the analysis and the machine contain the texts
    /// given (a text that is not given matches every block).
    fn matches(&self, software: &str, analysis: Option<&str>, machine: Option<&str>) -> bool {
        same(self.software.as_deref(), software)
            && contains(self.analysis.as_deref(), analysis)
            && contains(self.machine.as_deref(), machine)
    }
}

/// Runs `where-used`: the blocks of the team's usage index for the software, newest first, then
/// the count. The data is one object per use.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "where-used")?;
    let software = wanted(&args.software, "the software")?;
    let analysis = args
        .analysis
        .as_deref()
        .map(|text| wanted(text, "--analysis"))
        .transpose()?;
    let machine = args
        .machine
        .as_deref()
        .map(|text| wanted(text, "--machine"))
        .transpose()?;

    let file = index(&ctx)?;
    let mut uses: Vec<Usage> = file
        .keys()
        .iter()
        .map(|key| Usage::read(&file, key))
        .filter(|usage| usage.matches(&software, analysis.as_deref(), machine.as_deref()))
        .collect();
    // Newest date first. The index gets its blocks at the end, so within one day the later
    // block comes first: the file is read backwards before the stable sort by date.
    uses.reverse();
    uses.sort_by(|a, b| b.date().cmp(a.date()));

    let mut lines: Vec<String> = uses.iter().map(line).collect();
    lines.push(count(uses.len()));
    let data: Vec<serde_json::Value> = uses.iter().map(data).collect();
    Ok(Output::with(lines.join("\n"), json!(data)))
}

/// The team's usage index. In the team dossier itself it is this dossier's; in a project it is
/// the index of the project's team dossier. A team that cannot be reached is Code::NoDossier
/// naming its path, and a project without a team is refused by `team_ctx`.
fn index(ctx: &Ctx) -> Result<BlockFile> {
    if ctx.cfg.kind == TEAM {
        let (file, _) = teamfiles::usage(&ctx.dossier, &ctx.cfg.code)?;
        return Ok(file);
    }
    if ctx.team.is_none()
        && let Some(stored) = &ctx.cfg.team
    {
        return Err(Error::new(
            Code::NoDossier,
            format!(
                "{}: the team dossier cannot be reached",
                ctx.root.join(paths::shown(stored)).display()
            ),
        ));
    }
    let team = ctx.team_ctx()?;
    let (file, _) = teamfiles::usage(&team.dossier, &team.cfg.code)?;
    Ok(file)
}

/// A text the command looks for: trimmed, and not empty (Code::BadValue otherwise).
fn wanted(text: &str, what: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            format!("{what} must not be empty"),
        ));
    }
    Ok(text.to_string())
}

/// Whether the value is the text, ignoring case. A missing value is not.
fn same(value: Option<&str>, text: &str) -> bool {
    value.is_some_and(|value| value.to_lowercase() == text.to_lowercase())
}

/// Whether the value contains the text, ignoring case. No text matches every value, a missing
/// value only when no text is given.
fn contains(value: Option<&str>, text: Option<&str>) -> bool {
    match text {
        None => true,
        Some(text) => {
            value.is_some_and(|value| value.to_lowercase().contains(&text.to_lowercase()))
        }
    }
}

/// `<date>  <code>  <software> <version>  <analysis>  <machine>  <person>`, with `  howto <ID>` at
/// the end when the block names how-to. A value the block does not have is `-`.
fn line(usage: &Usage) -> String {
    let mut text = format!(
        "{}  {}  {} {}  {}  {}  {}",
        or_dash(usage.date.as_deref()),
        or_dash(usage.code.as_deref()),
        or_dash(usage.software.as_deref()),
        or_dash(usage.version.as_deref()),
        or_dash(usage.analysis.as_deref()),
        or_dash(usage.machine.as_deref()),
        or_dash(usage.person.as_deref()),
    );
    if let Some(howto) = &usage.howto {
        text.push_str(&format!("  howto {howto}"));
    }
    text
}

/// The value, or `-` when there is none.
fn or_dash(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

/// The data of one use: its key in the index and every value, `null` where the block has none.
fn data(usage: &Usage) -> serde_json::Value {
    json!({
        "key": usage.key,
        "date": usage.date,
        "code": usage.code,
        "software": usage.software,
        "version": usage.version,
        "analysis": usage.analysis,
        "machine": usage.machine,
        "howto": usage.howto,
        "person": usage.person,
    })
}

/// `1 use`, or `<n> uses`.
fn count(n: usize) -> String {
    if n == 1 {
        "1 use".to_string()
    } else {
        format!("{n} uses")
    }
}
