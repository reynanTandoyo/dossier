//! The `map` command (docs/design.md section 9, R-CODE-1): the map of a code project, kept in
//! `DOSSIER/map.md` (format.md §17). With no subcommand it prints the map. `map set` describes a
//! folder or package, and `map remove` marks one detached. Every change adds a history line with
//! who and when, and a journal line (D29).

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::mapfile;
use crate::output::Output;
use crate::paths;
use crate::tx::{self, Change};
use serde_json::{Value, json};

/// What `map` prints when the map has no described folder yet.
const NO_MAP: &str = "no map yet; add one with: dossier map set <path> \"<what it is for>\"";

/// The arguments of `map`. With no subcommand the map is printed.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// What `map` does besides printing the map.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Describes a folder or package of the code: what it is responsible for, and where to start
    /// reading. A folder that is already described gets the new purpose and start.
    Set {
        /// The folder or package, relative to the current folder or the project folder
        path: String,
        /// What this part is responsible for, such as "time integration and the linear solver"
        purpose: String,
        /// Where to begin reading, such as a file name; an empty text removes it
        #[arg(long)]
        start: Option<String>,
    },
    /// Takes a folder or package out of the map; its description and history stay in the file
    Remove {
        /// The folder or package, relative to the current folder or the project folder
        path: String,
    },
}

/// Runs `map`: prints the map when there is no subcommand, else sets or removes the description
/// of one folder.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "map")?;
    match &args.action {
        None => print_map(&ctx),
        Some(Action::Set {
            path,
            purpose,
            start,
        }) => set(&ctx, path, purpose, start.as_deref()),
        Some(Action::Remove { path }) => remove(&ctx, path),
    }
}

/// Prints the map: one line per described folder (`mapfile::lines`), and the same folders as
/// data. Without any folder, the line says how to start the map.
fn print_map(ctx: &Ctx) -> Result<Output> {
    let (file, _) = mapfile::load(&ctx.dossier, &ctx.cfg.code)?;
    let lines = mapfile::lines(&file);
    if lines.is_empty() {
        return Ok(Output::with(NO_MAP, json!([])));
    }
    Ok(Output::with(lines.join("\n"), Value::Array(rows(&file))))
}

/// The described folders as data, in the order of `mapfile::lines`: {path, purpose, start}, with
/// null for a field the block does not have.
fn rows(file: &BlockFile) -> Vec<Value> {
    described(file)
        .into_iter()
        .map(|key| {
            json!({
                "path": key,
                "purpose": present(file, &key, "purpose"),
                "start": present(file, &key, "start"),
            })
        })
        .collect()
}

/// The keys of the blocks that are printed: sorted by path, without the empty key, without the
/// detached blocks. This is the selection `mapfile::lines` makes.
fn described(file: &BlockFile) -> Vec<String> {
    let mut keys: Vec<String> = file
        .keys()
        .into_iter()
        .filter(|key| !key.is_empty())
        .collect();
    keys.sort();
    keys.dedup();
    keys.retain(|key| present(file, key, "detached").is_none());
    keys
}

/// `map set`: describes one folder. A folder with no block gets a new one, with `set` in its
/// history. A described folder gets the new purpose and start, with `changed`. A detached one is
/// used again: its `detached` field goes, and the history line is `set`. Nothing is written when
/// the block would not change.
fn set(ctx: &Ctx, typed: &str, purpose: &str, start: Option<&str>) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let stored = stored_path(ctx, typed)?;
    let purpose = purpose.trim();
    if purpose.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "a folder needs a purpose: say what this part is responsible for",
        ));
    }
    let start = start.map(str::trim);
    if !ctx.root.join(paths::shown(&stored)).exists() {
        return Err(Error::new(
            Code::NotFound,
            format!("no folder or file at {stored}"),
        ));
    }

    tx::write(ctx, |tx| {
        let (file, old) = mapfile::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let key = stored.as_str();
        let mut next = file.clone();
        let event = if !has_block(&file, key) {
            let mut fields = vec![("purpose".to_string(), purpose.to_string())];
            if let Some(start) = start {
                fields.push(("start".to_string(), start.to_string()));
            }
            next.add_block(key, &fields);
            "set"
        } else {
            let reused = is_detached(&file, key);
            if reused {
                next.set_field(key, "detached", "");
            }
            next.set_field(key, "purpose", purpose);
            if let Some(start) = start {
                next.set_field(key, "start", start);
            }
            if reused { "set" } else { "changed" }
        };
        if next.text() == file.text() {
            return Ok(no_change(&stored));
        }
        let detail = if event == "changed" {
            changed_fields(&file, &next, key)
        } else {
            String::new()
        };
        next.add_history(key, tx.time(), &who, event, &detail);

        let path = tx.ctx().dossier.join(mapfile::FILE);
        tx.put_raw(
            path,
            old,
            next.text().into_bytes(),
            Change {
                id: mapfile::ITEM.to_string(),
                event: event.to_string(),
                detail: stored.clone(),
            },
        );
        Ok(Output::with(
            format!("{stored}: {event}"),
            json!({"path": stored, "event": event}),
        ))
    })
}

/// `map remove`: marks one described folder detached, with the time and the name of who did it.
/// Its purpose and start stay in the block. The folder itself need not exist any more, so that a
/// folder that is gone can be taken out of the map.
fn remove(ctx: &Ctx, typed: &str) -> Result<Output> {
    let who = ctx.writer()?.clone();
    let stored = stored_path(ctx, typed)?;

    tx::write(ctx, |tx| {
        let (file, old) = mapfile::load(&tx.ctx().dossier, &tx.ctx().cfg.code)?;
        let key = stored.as_str();
        if !has_block(&file, key) {
            return Err(Error::new(
                Code::NotFound,
                format!("the map has no {stored}"),
            ));
        }
        if is_detached(&file, key) {
            return Ok(no_change(&stored));
        }
        let mut next = file.clone();
        next.set_field(key, "detached", &format!("{} by {}", tx.time(), who.name));
        next.add_history(key, tx.time(), &who, "detached", "");

        let path = tx.ctx().dossier.join(mapfile::FILE);
        tx.put_raw(
            path,
            old,
            next.text().into_bytes(),
            Change {
                id: mapfile::ITEM.to_string(),
                event: "detached".to_string(),
                detail: stored.clone(),
            },
        );
        Ok(Output::with(
            format!("{stored}: detached"),
            json!({"path": stored, "event": "detached"}),
        ))
    })
}

/// The stored path of a folder typed on the command line: relative to the project folder, with
/// `/` (format.md §17). The project folder itself cannot be described, and a path with a line
/// break would break the file, so both are refused (Code::BadValue).
fn stored_path(ctx: &Ctx, typed: &str) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let stored = paths::stored(&ctx.root, &cwd, typed)?;
    if stored.is_empty() {
        return Err(Error::new(
            Code::BadValue,
            "the map describes folders inside the project folder, not the project folder itself",
        ));
    }
    if stored.contains(['\r', '\n']) {
        return Err(Error::new(Code::BadValue, "a path is one line"));
    }
    Ok(stored)
}

/// Whether the file has a block with this key.
fn has_block(file: &BlockFile, key: &str) -> bool {
    file.keys().iter().any(|block| block == key)
}

/// Whether the block has a `detached` field.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    present(file, key, "detached").is_some()
}

/// The text of a field of a block when it is not empty: format.md §5 leaves an empty field out.
fn present(file: &BlockFile, key: &str, name: &str) -> Option<String> {
    file.field(key, name).filter(|value| !value.is_empty())
}

/// The detail of a `changed` history line: the purpose and start that differ between the two
/// versions of the block, as `field purpose, field start`.
fn changed_fields(before: &BlockFile, after: &BlockFile, key: &str) -> String {
    ["purpose", "start"]
        .into_iter()
        .filter(|name| before.field(key, name) != after.field(key, name))
        .map(|name| format!("field {name}"))
        .collect::<Vec<String>>()
        .join(", ")
}

/// The result when the block would not change: nothing is written.
fn no_change(stored: &str) -> Output {
    Output::with(
        format!("{stored}: no change"),
        json!({"path": stored, "event": null}),
    )
}
