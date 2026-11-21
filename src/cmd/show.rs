//! The `show` command: prints one entry as it is stored, with its marks on a first line
//! (design.md section 5, R-READ-1, R-LIFE-6). With `--json` it gives every field, every section
//! and every history line. Nothing is changed.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Kind, Status};
use crate::error::Result;
use crate::output::Output;
use crate::store::{self, Item};
use serde_json::{Map, Value, json};
use std::path::Path;

/// The arguments of `show`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012; any case, with or without a place prefix
    pub id: String,
}

/// Runs `show`: reads the entry, works out its marks, and prints the marks line (when there are
/// marks) followed by the file text. Problems in other files of the dossier are passed on as
/// warnings.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "show")?;
    let (_, warnings) = store::all(&ctx.dossier);
    let item = store::get(&ctx.dossier, &args.id)?;
    let marks = marks(&ctx, &item);
    let file = item.entry.text();
    let text = if marks.is_empty() {
        file
    } else {
        format!("{}\n{file}", marks.join(" "))
    };
    let mut output = Output::with(text, data(&ctx, &item, &marks));
    for warning in warnings {
        output = output.warn(warning);
    }
    Ok(output)
}

/// The marks of an entry, in the order of design.md section 5: its status, then
/// `[changed after approval]` for an approved lesson whose text differs from its recorded
/// checksum.
fn marks(ctx: &Ctx, item: &Item) -> Vec<String> {
    let entry = &item.entry;
    let mut marks = Vec::new();
    match entry.status() {
        Some(Status::Draft) => marks.push("[draft]".to_string()),
        Some(Status::Rejected) => marks.push("[rejected]".to_string()),
        Some(Status::Superseded) => marks.push(
            match entry.field("superseded-by").filter(|by| !by.is_empty()) {
                Some(by) => format!("[superseded by {by}]"),
                None => "[superseded]".to_string(),
            },
        ),
        Some(Status::Retired) => marks.push("[retired]".to_string()),
        Some(Status::Proposed) => marks.push("[proposed]".to_string()),
        Some(Status::Declined) => marks.push("[declined]".to_string()),
        Some(Status::Approved) | None => {}
    }
    if entry.kind() == Some(Kind::Lesson)
        && entry.status() == Some(Status::Approved)
        && matches!(
            approval::mark(&ctx.state, &entry.id(), &item.raw),
            Mark::Changed
        )
    {
        marks.push("[changed after approval]".to_string());
    }
    marks
}

/// The data of `--json`: the ID, title, kind, status, marks, the fields (each non-history field
/// once, with its first value; the history as a list), the sections, and the path of the file
/// relative to the project folder.
fn data(ctx: &Ctx, item: &Item, marks: &[String]) -> Value {
    let entry = &item.entry;
    let mut fields = Map::new();
    for (key, value) in entry.fields() {
        if key != "history" && !fields.contains_key(&key) {
            fields.insert(key, Value::String(value));
        }
    }
    fields.insert("history".to_string(), json!(entry.history()));
    let mut sections = Map::new();
    for (name, body) in entry.sections() {
        if !sections.contains_key(&name) {
            sections.insert(name, Value::String(body));
        }
    }
    json!({
        "id": entry.id(),
        "title": entry.title(),
        "kind": entry.kind().map(Kind::name),
        "status": entry.status().map(Status::name),
        "marks": marks,
        "fields": fields,
        "sections": sections,
        "path": relative(&ctx.root, &item.path),
    })
}

/// The path below the project folder, with `/` between its parts on every system.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
