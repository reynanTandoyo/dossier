//! The `link` command: adds each of two entries to the other's `related` field (R-WRITE-9). An
//! entry that does not name the other yet gets its ID and one history line; an entry that does is
//! left alone, and when both already name each other nothing is written.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Entry;
use crate::error::{Code, Error, Result};
use crate::names::Who;
use crate::output::Output;
use crate::store::{self, Item};
use crate::tx::{self, Change, Tx};
use serde_json::json;

/// The field that lists the related entries (format.md §5).
const RELATED: &str = "related";

/// The event of the history line and of the journal line (format.md §6).
const EVENT: &str = "changed";

/// The arguments of `link`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the first entry, such as DEC-012
    pub a: String,
    /// The ID of the other entry, such as FND-007
    pub b: String,
}

/// Runs `link`: refuses two equal IDs, then adds each entry to the other's related field in one
/// transaction. The output is `<A> and <B>: linked`, or `already linked` when both entries
/// already name each other.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "link")?;
    if bare_id(&args.a).eq_ignore_ascii_case(bare_id(&args.b)) {
        return Err(Error::new(
            Code::BadValue,
            "an entry cannot be linked to itself",
        ));
    }
    let who = ctx.writer()?.clone();

    tx::write(&ctx, |tx| {
        let item_a = store::get(&tx.ctx().dossier, &args.a)?;
        let item_b = store::get(&tx.ctx().dossier, &args.b)?;
        let id_a = item_a.entry.id();
        let id_b = item_b.entry.id();

        let a_names_b = names(&item_a.entry, &id_b);
        let b_names_a = names(&item_b.entry, &id_a);
        if a_names_b && b_names_a {
            return Ok(Output::with(
                "already linked",
                json!({"a": id_a, "b": id_b, "changed": []}),
            ));
        }

        let mut changed: Vec<String> = Vec::new();
        if !a_names_b {
            relate(tx, &who, &item_a, &id_b);
            changed.push(id_a.clone());
        }
        if !b_names_a {
            relate(tx, &who, &item_b, &id_a);
            changed.push(id_b.clone());
        }
        let text = format!("{id_a} and {id_b}: linked");
        Ok(Output::with(
            text,
            json!({"a": id_a, "b": id_b, "changed": changed}),
        ))
    })
}

/// The ID without a place prefix: `P-017:DEC-012` gives `DEC-012`, as `store::get` reads it.
fn bare_id(id: &str) -> &str {
    id.rsplit_once(':').map_or(id, |(_, rest)| rest)
}

/// Whether the related field of the entry names `id`, ignoring case.
fn names(entry: &Entry, id: &str) -> bool {
    entry
        .list(RELATED)
        .iter()
        .any(|related| related.eq_ignore_ascii_case(id))
}

/// Queues the entry with `other` added to its related field, after the names already there, and
/// one history line that says which entry was related.
fn relate(tx: &mut Tx<'_>, who: &Who, item: &Item, other: &str) {
    let mut entry = item.entry.clone();
    let mut list = entry.list(RELATED);
    list.push(other.to_string());
    entry.set_field(RELATED, &list.join(", "));
    let detail = format!("related {other}");
    entry.add_history(tx.time(), who, EVENT, &detail);
    tx.put(
        item.path.clone(),
        Some(item),
        entry.text(),
        Change {
            id: entry.id(),
            event: EVENT.to_string(),
            detail,
        },
    );
}
