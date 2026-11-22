//! The `show` command: prints one entry as it is stored, with its marks on a first line
//! (design.md section 5, R-READ-1, R-LIFE-6, R-LIFE-7, R-IMPORT-5, R-USE-11). With `--json` it
//! gives every field, every section and every history line. Nothing is changed. An ID with the
//! place `team:` (R-ID-2) shows an entry of the team dossier of the project; its JSON `id` keeps
//! the place.

use crate::approval::{self, Mark};
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::filesum;
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::time;
use chrono::NaiveDate;
use serde_json::{Map, Value, json};
use std::path::{MAIN_SEPARATOR, Path};

/// The place of an entry of the team dossier, as an ID is written from a project (R-ID-2).
const TEAM: &str = "team";

/// The arguments of `show`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The ID of the entry, such as DEC-012; any case, with or without a place prefix. `team:`
    /// before the ID shows an entry of the team dossier
    pub id: String,
}

/// Runs `show`: reads the entry, works out its marks, and prints the marks line (when there are
/// marks) followed by the file text. An ID with the place `team:` is read from the team dossier of
/// the project (Code::NoDossier when there is none). Problems in other files of the dossier are
/// passed on as warnings.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "show")?;
    let team = team_id(&args.id);
    let (target, id) = match team {
        Some(id) => (team_of(&ctx)?, id),
        None => (ctx, args.id.as_str()),
    };
    let (_, warnings) = store::all(&target.dossier);
    let item = store::get(&target.dossier, id)?;
    let shown_id = if team.is_some() {
        format!("{TEAM}:{}", item.entry.id())
    } else {
        item.entry.id()
    };
    let marks = marks(&target, &item);
    let file = item.entry.text();
    let text = if marks.is_empty() {
        file
    } else {
        format!("{}\n{file}", marks.join(" "))
    };
    let mut output = Output::with(text, data(&target, &item, &marks, &shown_id));
    for warning in warnings {
        output = output.warn(shown_warning(&target.root, &warning));
    }
    Ok(output)
}

/// The ID after the place `team:` (any case), or None when the ID has no such place.
fn team_id(id: &str) -> Option<&str> {
    let (place, rest) = id.split_once(':')?;
    place.eq_ignore_ascii_case(TEAM).then_some(rest)
}

/// The team dossier of the project as a context, for an ID with the place `team:` (R-ID-2).
/// Code::NoDossier naming `dossier team use <path>` when the project names no team dossier
/// (R-PLACE-8), or naming the path when the team dossier it names cannot be reached.
fn team_of(ctx: &Ctx) -> Result<Ctx> {
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
    ctx.team_ctx()
}

/// A warning of a dossier read, with the path in it shown relative to `root`, with `/` between its
/// parts. A warning without a path under `root` is kept as it is.
fn shown_warning(root: &Path, warning: &str) -> String {
    let root_text = root.display().to_string();
    let Some(rest) = warning.strip_prefix(root_text.as_str()) else {
        return warning.to_string();
    };
    let rest = rest.strip_prefix(['\\', '/']).unwrap_or(rest);
    match rest.split_once(": ") {
        Some((path, message)) => format!("{}: {message}", path.replace(MAIN_SEPARATOR, "/")),
        None => warning.to_string(),
    }
}

/// The marks of an entry, in this order: its status (design.md section 5); `[due for review]`
/// when its `review` date is before today (R-LIFE-7); `[imported]` when a document was imported
/// into it (R-IMPORT-5); `[changed after approval]` for an approved guarded entry (lesson,
/// how-to or tool) whose text differs from its recorded checksum (R-LIFE-6); `[file changed since
/// approval]` for a tool whose script changed (R-USE-11); then `[rests on <IDs>]` when its
/// `rests-on` field names a wrong assumption.
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
    if due_for_review(entry, time::date(&time::now())) {
        marks.push("[due for review]".to_string());
    }
    if imported(entry) {
        marks.push("[imported]".to_string());
    }
    if entry.kind().is_some_and(approval::guarded)
        && entry.status() == Some(Status::Approved)
        && matches!(
            approval::mark(&ctx.state, &entry.id(), &item.raw),
            Mark::Changed
        )
    {
        marks.push("[changed after approval]".to_string());
    }
    if file_changed(&ctx.root, entry) {
        marks.push("[file changed since approval]".to_string());
    }
    if let Some(mark) = rests_on_mark(entry) {
        marks.push(mark);
    }
    marks
}

/// Whether the `review` date of the entry (format.md §23) is before `today`, both as
/// YYYY-MM-DD (R-LIFE-7). A review value that is not a date gives no mark.
fn due_for_review(entry: &Entry, today: &str) -> bool {
    let review = entry.field("review").and_then(|text| parse_day(&text));
    matches!((review, parse_day(today)), (Some(review), Some(today)) if review < today)
}

/// A date written YYYY-MM-DD (format.md §23, the `review` field), or None.
fn parse_day(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
}

/// Whether the entry was imported from a document: `imported: yes` (format.md §23, R-IMPORT-5).
fn imported(entry: &Entry) -> bool {
    entry
        .field("imported")
        .is_some_and(|value| value.eq_ignore_ascii_case("yes"))
}

/// Whether a tool's script changed since its checksum was recorded (format.md §22, R-USE-11): the
/// `location` is a file that exists, the entry has a `checksum`, and the file's checksum now
/// differs. The location is relative to the project folder, or absolute. A location that is a
/// command name or a URL, a missing checksum, and a file that cannot be read give no mark.
fn file_changed(root: &Path, entry: &Entry) -> bool {
    if entry.kind() != Some(Kind::Tool) {
        return false;
    }
    let (Some(location), Some(recorded)) = (entry.field("location"), entry.field("checksum"))
    else {
        return false;
    };
    let path = root.join(&location);
    !recorded.is_empty()
        && path.is_file()
        && filesum::checksum(&path).is_ok_and(|now| now != recorded)
}

/// `[rests on <IDs>]` from the field `rests-on` (format.md §14), the IDs joined with `, `. None
/// when the field names no ID.
fn rests_on_mark(entry: &Entry) -> Option<String> {
    let ids = entry.list("rests-on");
    (!ids.is_empty()).then(|| format!("[rests on {}]", ids.join(", ")))
}

/// The data of `--json`: the ID (`shown_id`, with `team:` before it for a team entry), title, kind,
/// status, marks, the fields (each non-history field once, with its first value; the history as a
/// list), the sections, and the path of the file relative to the folder of the dossier read.
fn data(ctx: &Ctx, item: &Item, marks: &[String], shown_id: &str) -> Value {
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
        "id": shown_id,
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
