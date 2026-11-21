//! The `value` command (docs/design.md section 9, format.md §17): a key value of the project, set,
//! shown, approved and rejected (R-VALUE-1 to R-VALUE-3). A person's value comes into force at
//! once. An LLM's value waits for a person as a proposal, unless the LLM names the person who says
//! it (`--said-by`) or the source is a path attached to a task. Every change adds a history line
//! with who and when; a change of the value in force names the usage records and entries that use
//! or mention it.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names::{self, Who};
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::term;
use crate::tx::{self, Change, Tx};
use crate::values;
use serde_json::json;
use std::path::Path;

/// The arguments of `value`.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

/// What `value` does.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Sets a value: it comes into force for a person, or for an LLM with --said-by or an attached
    /// --source; otherwise it is proposed for a person to approve
    Set {
        /// The name of the value, such as design.massflow
        name: String,
        /// The value with its unit, such as "12.4 kg/s": the unit is the last word
        value: String,
        /// A value with no unit, such as "air, ideal gas"
        #[arg(long)]
        dimensionless: bool,
        /// Where the value comes from: a path attached to a task, a document, or a person
        #[arg(long, value_name = "text")]
        source: Option<String>,
        /// A note on the value
        #[arg(long, value_name = "text")]
        note: Option<String>,
        /// Marks the value for brief and status
        #[arg(long)]
        key: bool,
        /// The person an LLM's value is said for, so that it comes into force
        #[arg(long = "said-by", value_name = "person")]
        said_by: Option<String>,
    },
    /// Shows the fields and the history of a value
    Show {
        /// The name of the value
        name: String,
    },
    /// Approves the proposed value, which comes into force (a person at a terminal)
    Approve {
        /// The name of the value
        name: String,
    },
    /// Rejects the proposed value (a person at a terminal)
    Reject {
        /// The name of the value
        name: String,
    },
}

/// What `set` was given, as the command line gave it.
struct Setting<'a> {
    name: &'a str,
    value: &'a str,
    dimensionless: bool,
    source: Option<&'a str>,
    note: Option<&'a str>,
    key: bool,
    said_by: Option<&'a str>,
}

/// The fields a value in force can take besides its value: the ones the command was given.
#[derive(Default)]
struct Details {
    source: Option<String>,
    note: Option<String>,
    key: bool,
}

/// What one change wrote: the status word for the data, the event and detail of the journal, the
/// first output line, the value as shown, and whether the value in force changed.
struct Written {
    status: &'static str,
    event: &'static str,
    detail: String,
    line: String,
    shown: String,
    moved: bool,
}

/// Runs `value`: one of set, show, approve and reject.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "value")?;
    match &args.action {
        Action::Set {
            name,
            value,
            dimensionless,
            source,
            note,
            key,
            said_by,
        } => set(
            &ctx,
            g,
            &Setting {
                name,
                value,
                dimensionless: *dimensionless,
                source: source.as_deref(),
                note: note.as_deref(),
                key: *key,
                said_by: said_by.as_deref(),
            },
        ),
        Action::Show { name } => show(&ctx, name),
        Action::Approve { name } => approve(&ctx, g, name),
        Action::Reject { name } => reject(&ctx, g, name),
    }
}

/// Sets a value: in force (a person, or an LLM with --said-by or an attached source), or proposed.
/// The name, the value and the people named are checked before anything is written.
fn set(ctx: &Ctx, g: &Global, setting: &Setting) -> Result<Output> {
    let who = names::resolve(&ctx.cfg, g.agent.as_deref())?;
    if !values::valid_name(setting.name) {
        return Err(bad(format!(
            "a value name has lower-case letters, digits, '.', '_' and '-', and starts with a letter: {}",
            setting.name
        )));
    }
    let (value, unit) = values::split(setting.value, setting.dimensionless)?;
    if let Some(person) = setting
        .said_by
        .filter(|person| !names::is_person(&ctx.cfg, person))
    {
        return Err(Error::new(
            Code::UnknownName,
            format!("--said-by names a person of dossier.toml, and {person} is not one"),
        ));
    }
    let attached = setting.source.and_then(|source| attached_key(ctx, source));
    let in_force = !who.is_llm || setting.said_by.is_some() || attached.is_some();
    if !in_force && unit.is_none() {
        return Err(bad(format!(
            "a proposed value needs a unit: a value with no unit comes into force only when a person sets it, or an LLM gives --said-by or an attached --source; {} has no unit",
            setting.value.trim()
        )));
    }
    let details = Details {
        source: attached.or_else(|| setting.source.map(|source| source.trim().to_string())),
        note: setting.note.map(|note| note.trim().to_string()),
        key: setting.key,
    };

    let output = tx::write(ctx, |tx| {
        let (mut file, old) = values::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let now = tx.time().to_string();
        let written = if in_force {
            come_into_force(
                &mut file,
                setting.name,
                &value,
                unit.as_deref(),
                &details,
                &now,
                &who,
            )
        } else {
            Some(propose(
                &mut file,
                setting.name,
                &value,
                unit.as_deref(),
                &now,
                &who,
            ))
        };
        commit(tx, setting.name, file, old, written)
    })?;
    if !in_force && (setting.source.is_some() || setting.note.is_some() || setting.key) {
        return Ok(output.warn(
            "source, note and key are kept when a person sets the value; a proposal keeps only the value and its unit",
        ));
    }
    Ok(output)
}

/// Shows the fields and the history lines of a value, or Code::NotFound.
fn show(ctx: &Ctx, name: &str) -> Result<Output> {
    let (file, _) = values::load(&ctx.dossier, &ctx.cfg.code)?;
    let value = values::get(&file, name)
        .ok_or_else(|| Error::new(Code::NotFound, format!("no value named {name}")))?;
    let mut lines = vec![name.to_string()];
    lines.extend(
        file.fields(name)
            .iter()
            .map(|(field, text)| format!("  {field}: {text}")),
    );
    let data = json!({
        "name": name,
        "value": value.value,
        "unit": value.unit,
        "source": value.source,
        "note": value.note,
        "key": value.key,
        "valid_from": value.valid_from,
        "proposed": value.proposed,
        "history": file.history(name),
    });
    Ok(Output::with(lines.join("\n"), data))
}

/// Approves the proposed value: it comes into force, named by the person who approves it.
fn approve(ctx: &Ctx, g: &Global, name: &str) -> Result<Output> {
    let who = names::resolve(&ctx.cfg, g.agent.as_deref())?;
    person_at_terminal(&who, "value approve")?;
    tx::write(ctx, |tx| {
        let (mut file, old) = values::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let now = tx.time().to_string();
        let written = approve_proposal(&mut file, name, &now, &who)?;
        commit(tx, name, file, old, written)
    })
}

/// Rejects the proposed value: the proposal is removed and its history line kept.
fn reject(ctx: &Ctx, g: &Global, name: &str) -> Result<Output> {
    let who = names::resolve(&ctx.cfg, g.agent.as_deref())?;
    person_at_terminal(&who, "value reject")?;
    tx::write(ctx, |tx| {
        let (mut file, old) = values::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let now = tx.time().to_string();
        let written = reject_proposal(&mut file, name, &now, &who)?;
        commit(tx, name, file, old, Some(written))
    })
}

/// Writes the value file when the change wrote something, and says what it did. `None` is no
/// change. A change of the value in force also names the usage records and entries that use or
/// mention the value (R-VALUE-3), as warnings.
fn commit(
    tx: &mut Tx<'_>,
    name: &str,
    file: BlockFile,
    old: Option<Vec<u8>>,
    written: Option<Written>,
) -> Result<Output> {
    let Some(written) = written else {
        return Ok(Output::with(
            format!("{name}: no change"),
            json!({"name": name, "status": "no change"}),
        ));
    };
    let dossier = tx.ctx().dossier.clone();
    let warnings = if written.moved {
        warnings_for(&dossier, name)
    } else {
        Vec::new()
    };
    tx.put_raw(
        dossier.join(values::FILE),
        old,
        file.text().into_bytes(),
        Change {
            id: values::ITEM.to_string(),
            event: written.event.to_string(),
            detail: written.detail,
        },
    );
    let output = Output::with(
        written.line,
        json!({"name": name, "status": written.status, "value": written.shown}),
    );
    Ok(warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning)))
}

/// Brings a value into force. A new block gets its value, unit (or `dimensionless: yes`), the
/// fields given, `valid-from` and the history line `set — <value>[ <unit>]`; an existing block gets
/// the same fields set, and its pending proposal removed. A value that is already in force gets
/// only the fields given that differ, with the history line `changed — <fields>`. None when
/// nothing differs.
fn come_into_force(
    file: &mut BlockFile,
    name: &str,
    value: &str,
    unit: Option<&str>,
    details: &Details,
    now: &str,
    who: &Who,
) -> Option<Written> {
    let before = values::get(file, name);
    let shown = display(value, unit);
    let in_force = before
        .as_ref()
        .filter(|old| !old.value.is_empty())
        .map(|old| (old.value.clone(), old.unit.clone()));
    let moved = in_force.as_ref() != Some(&(value.to_string(), unit.map(str::to_string)));
    let pending = before.as_ref().is_some_and(|old| old.proposed.is_some());

    if moved || pending {
        let mut fields: Vec<(&str, &str)> = vec![
            ("value", value),
            ("unit", unit.unwrap_or("")),
            ("dimensionless", if unit.is_none() { "yes" } else { "" }),
            ("valid-from", now),
        ];
        if let Some(source) = details.source.as_deref() {
            fields.push(("source", source));
        }
        if let Some(note) = details.note.as_deref() {
            fields.push(("note", note));
        }
        if details.key {
            fields.push(("key", "yes"));
        }
        if before.is_some() {
            for (field, text) in &fields {
                file.set_field(name, field, text);
            }
            file.set_field(name, "proposed", "");
        } else {
            let owned: Vec<(String, String)> = fields
                .iter()
                .map(|(field, text)| (field.to_string(), text.to_string()))
                .collect();
            file.add_block(name, &owned);
        }
        file.add_history(name, now, who, "set", &shown);
        return Some(Written {
            status: "set",
            event: "set",
            detail: format!("{name}: {shown}"),
            line: format!("{name}: set — {shown}"),
            shown,
            moved,
        });
    }

    // The value in force stays; only the fields given may differ.
    let old = before?;
    let mut changed: Vec<&str> = Vec::new();
    if let Some(source) = details
        .source
        .as_deref()
        .filter(|source| old.source.as_deref().unwrap_or("") != *source)
    {
        file.set_field(name, "source", source);
        changed.push("source");
    }
    if let Some(note) = details
        .note
        .as_deref()
        .filter(|note| old.note.as_deref().unwrap_or("") != *note)
    {
        file.set_field(name, "note", note);
        changed.push("note");
    }
    if details.key && !old.key {
        file.set_field(name, "key", "yes");
        changed.push("key");
    }
    if changed.is_empty() {
        return None;
    }
    let list = changed.join(", ");
    file.add_history(name, now, who, "changed", &list);
    Some(Written {
        status: "changed",
        event: "changed",
        detail: format!("{name}: {list}"),
        line: format!("{name}: changed — {list}"),
        shown,
        moved: false,
    })
}

/// Proposes a value: the `proposed` field `<value>[ <unit>] by <who> at <time>` and the history
/// line `proposed — <value>[ <unit>]`. The value in force stays.
fn propose(
    file: &mut BlockFile,
    name: &str,
    value: &str,
    unit: Option<&str>,
    now: &str,
    who: &Who,
) -> Written {
    let shown = display(value, unit);
    let proposal = format!("{shown} by {} at {now}", who.name);
    if values::get(file, name).is_some() {
        file.set_field(name, "proposed", &proposal);
    } else {
        file.add_block(name, &[("proposed".to_string(), proposal)]);
    }
    file.add_history(name, now, who, "proposed", &shown);
    Written {
        status: "proposed",
        event: "proposed",
        detail: format!("{name}: {shown}"),
        line: format!("{name}: proposed — {shown} (waiting for a person)"),
        shown,
        moved: false,
    }
}

/// Approves the proposal: its value and unit come into force, as `come_into_force` makes them.
fn approve_proposal(
    file: &mut BlockFile,
    name: &str,
    now: &str,
    who: &Who,
) -> Result<Option<Written>> {
    let proposal = pending(file, name)?;
    let (value, unit) = values::split(proposal_value(&proposal)?, false)?;
    let written = come_into_force(
        file,
        name,
        &value,
        unit.as_deref(),
        &Details::default(),
        now,
        who,
    );
    Ok(written.map(|written| Written {
        status: "approved",
        line: format!("{name}: approved — {}", written.shown),
        ..written
    }))
}

/// Rejects the proposal: the field is removed and the history line `rejected — <value>[ <unit>]`
/// is added.
fn reject_proposal(file: &mut BlockFile, name: &str, now: &str, who: &Who) -> Result<Written> {
    let proposal = pending(file, name)?;
    let shown = proposal_value(&proposal)?.to_string();
    file.set_field(name, "proposed", "");
    file.add_history(name, now, who, "rejected", &shown);
    Ok(Written {
        status: "rejected",
        event: "rejected",
        detail: format!("{name}: {shown}"),
        line: format!("{name}: rejected — {shown}"),
        shown,
        moved: false,
    })
}

/// The text of the pending proposal of a value, or Code::NotFound when there is none.
fn pending(file: &BlockFile, name: &str) -> Result<String> {
    values::get(file, name)
        .and_then(|value| value.proposed)
        .ok_or_else(|| Error::new(Code::NotFound, format!("{name} has no proposed value")))
}

/// The `<value>[ <unit>]` part of a proposal `<value>[ <unit>] by <who> at <time>`.
fn proposal_value(proposal: &str) -> Result<&str> {
    proposal
        .rfind(" at ")
        .and_then(|at| proposal[..at].rfind(" by ").map(|by| &proposal[..by]))
        .ok_or_else(|| {
            Error::new(
                Code::Parse,
                format!("values.md: the proposal '{proposal}' is not '<value> by <who> at <time>'"),
            )
        })
}

/// The warnings of a change of the value in force (R-VALUE-3): `used by <ID>` for each usage
/// record whose Parameters name `{<name>}`, then `named in <ID>` for each other entry whose text
/// mentions the name.
fn warnings_for(dossier: &Path, name: &str) -> Vec<String> {
    let (items, _) = store::all(dossier);
    let reference = format!("{{{name}}}");
    let mut warnings = Vec::new();
    let mut used: Vec<String> = Vec::new();
    for item in &items {
        let in_parameters = item
            .entry
            .section("Parameters")
            .is_some_and(|body| body.contains(&reference));
        if item.entry.kind() == Some(Kind::Usage) && in_parameters {
            let id = item.entry.id();
            warnings.push(format!("used by {id}"));
            used.push(id);
        }
    }
    for item in &items {
        let id = item.entry.id();
        if !used.contains(&id) && mentions(&item.entry.text(), name) {
            warnings.push(format!("named in {id}"));
        }
    }
    warnings
}

/// Whether `text` names the value `name`: the name as a whole word, ignoring case. A letter or
/// digit next to it means another name, such as design.massflow2 for design.massflow.
fn mentions(text: &str, name: &str) -> bool {
    let text = text.to_lowercase();
    let name = name.to_lowercase();
    text.match_indices(&name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + name.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric())
            && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

/// The path, as stored, when `typed` is a path attached to a task and not detached (R-VALUE-2).
fn attached_key(ctx: &Ctx, typed: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let stored = paths::stored(&ctx.root, &cwd, typed).ok()?;
    let attached = filelist::tasks(&ctx.dossier).iter().any(|task| {
        filelist::load(&ctx.dossier, task).is_ok_and(|(file, _)| {
            let detached = file
                .field(&stored, "detached")
                .is_some_and(|at| !at.is_empty());
            file.keys().contains(&stored) && !detached
        })
    });
    attached.then_some(stored)
}

/// Refuses an LLM (Code::NeedsPerson) and a run with no terminal (Code::NeedsTerminal).
fn person_at_terminal(who: &Who, command: &str) -> Result<()> {
    if who.is_llm {
        return Err(Error::new(
            Code::NeedsPerson,
            format!("{command} is for a person; {} is an LLM", who.name),
        ));
    }
    if !term::is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    Ok(())
}

/// The value with its unit, as the file and the output show it: `12.4 kg/s`, or `air, ideal gas`.
fn display(value: &str, unit: Option<&str>) -> String {
    match unit {
        Some(unit) => format!("{value} {unit}"),
        None => value.to_string(),
    }
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
