//! The `link` command: adds each of two entries to the other's field of the link type (R-WRITE-9,
//! D61). The type is `related` unless `--as` names `refines` or `contradicts`. An entry that does
//! not name the other yet gets its ID and one history line; an entry that does is left alone, and
//! when both already name each other nothing is written.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::Entry;
use crate::error::{Code, Error, Result};
use crate::names::Who;
use crate::output::Output;
use crate::store::{self, Item};
use crate::tx::{self, Change, Tx};
use serde_json::json;

/// The event of the history line and of the journal line (format.md §6).
const EVENT: &str = "changed";

/// The arguments of `link`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the first entry, such as DEC-012
    pub a: String,
    /// The ID of the other entry, such as FND-007
    pub b: String,
    /// The type of the link: refines (the first entry narrows the second) or contradicts; without
    /// it a plain related link (R-WRITE-9)
    #[arg(long = "as", value_name = "TYPE")]
    pub as_type: Option<String>,
}

/// One type of link (format.md §38): the field written into the first entry, the field written
/// into the second entry, and the name of the type.
#[derive(Clone, Copy)]
struct Kind {
    a: &'static str,
    b: &'static str,
    name: &'static str,
}

/// The type that `--as` names, trimmed and in lower case; without it the type is `related`.
fn kind_of(as_type: Option<&str>) -> Result<Kind> {
    let kind = match as_type.map(|t| t.trim().to_lowercase()).as_deref() {
        None => Kind {
            a: "related",
            b: "related",
            name: "related",
        },
        Some("refines") => Kind {
            a: "refines",
            b: "refined-by",
            name: "refines",
        },
        Some("contradicts") => Kind {
            a: "contradicts",
            b: "contradicts",
            name: "contradicts",
        },
        Some(_) => {
            return Err(Error::new(
                Code::BadValue,
                "--as takes refines or contradicts",
            ));
        }
    };
    Ok(kind)
}

/// Runs `link`: refuses two equal IDs and an unknown type, then adds each entry to the other's
/// field of the type in one transaction. The output is `<A> and <B>: linked` for a plain link,
/// `<A> refines <B>` or `<A> contradicts <B>`, or `already linked` when both entries already name
/// each other in that field.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "link")?;
    if bare_id(&args.a).eq_ignore_ascii_case(bare_id(&args.b)) {
        return Err(Error::new(
            Code::BadValue,
            "an entry cannot be linked to itself",
        ));
    }
    let kind = kind_of(args.as_type.as_deref())?;
    let who = ctx.writer()?.clone();

    tx::write(&ctx, |tx| {
        let item_a = store::get(&tx.ctx().dossier, &args.a)?;
        let item_b = store::get(&tx.ctx().dossier, &args.b)?;
        let id_a = item_a.entry.id();
        let id_b = item_b.entry.id();

        let a_names_b = names(&item_a.entry, kind.a, &id_b);
        let b_names_a = names(&item_b.entry, kind.b, &id_a);
        if a_names_b && b_names_a {
            return Ok(Output::with(
                "already linked",
                json!({"a": id_a, "b": id_b, "as": kind.name, "changed": []}),
            ));
        }

        let mut changed: Vec<String> = Vec::new();
        if !a_names_b {
            relate(tx, &who, &item_a, kind.a, &id_b);
            changed.push(id_a.clone());
        }
        if !b_names_a {
            relate(tx, &who, &item_b, kind.b, &id_a);
            changed.push(id_b.clone());
        }
        let text = match kind.name {
            "related" => format!("{id_a} and {id_b}: linked"),
            name => format!("{id_a} {name} {id_b}"),
        };
        Ok(Output::with(
            text,
            json!({"a": id_a, "b": id_b, "as": kind.name, "changed": changed}),
        ))
    })
}

/// The ID without a place prefix: `P-017:DEC-012` gives `DEC-012`, as `store::get` reads it.
fn bare_id(id: &str) -> &str {
    id.rsplit_once(':').map_or(id, |(_, rest)| rest)
}

/// Whether `field` of the entry names `id`, ignoring case.
fn names(entry: &Entry, field: &str, id: &str) -> bool {
    entry
        .list(field)
        .iter()
        .any(|named| named.eq_ignore_ascii_case(id))
}

/// Queues the entry with `other` added to `field`, after the names already there, and one history
/// line that says which link was made, such as `related FND-001` or `refined by DEC-001`.
fn relate(tx: &mut Tx<'_>, who: &Who, item: &Item, field: &str, other: &str) {
    let mut entry = item.entry.clone();
    let mut list = entry.list(field);
    list.push(other.to_string());
    entry.set_field(field, &list.join(", "));
    let detail = format!("{} {other}", field.replace('-', " "));
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
