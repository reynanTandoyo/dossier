//! The `runs` command (docs/design.md section 9, R-USE-15): the approved usage records, newest first,
//! each with the parameters that differ from the run before it with the same model. With
//! `--compare <ID> <ID>` it prints the fields and parameters that differ between two usage records.
//!
//! Time order is `created` (the instant it names; a record without one comes first), then the
//! number of the ID, so records made at the same time keep the order they were recorded in. The
//! newest first list is that order reversed. The run a change is measured against is the previous
//! approved usage record with the same model, whichever filters are given.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind, Status};
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::paths;
use crate::store::{self, Item};
use crate::time;
use chrono::{DateTime, FixedOffset};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{MAIN_SEPARATOR, Path};

/// The fields that are not compared: the history lines are the record of changes, not of the run.
const SKIPPED_FIELD: &str = "history";

/// The arguments of `runs`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the runs of this task, such as T-042
    #[arg(long, value_name = "task")]
    pub task: Option<String>,
    /// Only the runs of this simulation file, as a path
    #[arg(long, value_name = "path")]
    pub model: Option<String>,
    /// Only the runs of this software, such as ExampleSolver
    #[arg(long, value_name = "name")]
    pub software: Option<String>,
    /// Compares two usage records: each field and each parameter that differs
    #[arg(
        long,
        num_args = 2,
        value_names = ["ID", "ID"],
        conflicts_with_all = ["task", "model", "software"]
    )]
    pub compare: Option<Vec<String>>,
}

/// A value that differs between two records: `before` and `after` are the values, None when the
/// record does not give one. In a list of changes `before` is the previous run, in a comparison
/// the first ID.
struct Difference {
    name: String,
    before: Option<String>,
    after: Option<String>,
}

/// Runs `runs`: the runs the filters choose, or the comparison of two records. The warnings of
/// reading the dossier are passed on.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "runs")?;
    let (items, warnings) = store::all(&ctx.dossier);
    let output = match &args.compare {
        Some(ids) => compare(&ctx.dossier, ids)?,
        None => list(&ctx, &items, args)?,
    };
    Ok(warnings.iter().fold(output, |output, warning| {
        output.warn(shown_warning(&ctx.root, warning))
    }))
}

/// The list: the approved usage records the filters choose, newest first. Each run has its line,
/// a `changed:` line per parameter that differs from the previous run with the same model, and
/// an `outputs:` line. The data is one object per run.
fn list(ctx: &Ctx, items: &[Item], args: &Args) -> Result<Output> {
    let cwd = std::env::current_dir()?;
    let model = args
        .model
        .as_deref()
        .map(|typed| paths::stored(&ctx.root, &cwd, typed.trim()))
        .transpose()?;
    let runs = in_time_order(items);
    let changes = changes_by_run(&runs);

    let mut lines = Vec::new();
    let mut data = Vec::new();
    for (index, item) in runs.iter().enumerate().rev() {
        let entry = &item.entry;
        if !chosen(entry, args, model.as_deref()) {
            continue;
        }
        let outputs = outputs_of(entry);
        lines.push(line(entry, &changes[index], &outputs));
        data.push(row(entry, &changes[index], &outputs));
    }
    let text = if lines.is_empty() {
        "no runs".to_string()
    } else {
        lines.join("\n")
    };
    Ok(Output::with(text, json!(data)))
}

/// The comparison of two usage records: each field that differs (the title included), then each
/// parameter that differs, one `<name>: <first> | <second>` line each. The data has both IDs and
/// the two lists.
fn compare(dossier: &Path, ids: &[String]) -> Result<Output> {
    let [first, second] = ids else {
        return Err(Error::new(
            Code::Usage,
            "--compare takes two usage record IDs",
        ));
    };
    let a = usage(dossier, first)?;
    let b = usage(dossier, second)?;

    let fields: Vec<Difference> = field_names(&a, &b)
        .into_iter()
        .filter_map(|name| {
            let (before, after) = if name == "title" {
                (Some(a.title()), Some(b.title()))
            } else {
                (a.field(&name), b.field(&name))
            };
            (before != after).then_some(Difference {
                name,
                before,
                after,
            })
        })
        .collect();
    let parameters = parameter_changes(&a, &b);

    let lines: Vec<String> = fields
        .iter()
        .chain(&parameters)
        .map(|difference| {
            format!(
                "{}: {} | {}",
                difference.name,
                dash(difference.before.as_deref()),
                dash(difference.after.as_deref())
            )
        })
        .collect();
    let text = if lines.is_empty() {
        "no differences".to_string()
    } else {
        lines.join("\n")
    };
    let data = json!({
        "a": a.id(),
        "b": b.id(),
        "fields": differences_data(&fields),
        "parameters": differences_data(&parameters),
    });
    Ok(Output::with(text, data))
}

/// The usage record `id` (any case, with or without a place prefix). Another kind is refused.
fn usage(dossier: &Path, id: &str) -> Result<Entry> {
    let entry = store::get(dossier, id)?.entry;
    if entry.kind() == Some(Kind::Usage) {
        Ok(entry)
    } else {
        Err(Error::new(
            Code::BadValue,
            format!("{} is not a usage record", entry.id()),
        ))
    }
}

/// The approved usage records in time order, oldest first: by `created` (a record without a valid
/// one comes first), then by the number of the ID.
fn in_time_order(items: &[Item]) -> Vec<&Item> {
    let mut runs: Vec<&Item> = items
        .iter()
        .filter(|item| {
            item.entry.kind() == Some(Kind::Usage) && item.entry.status() == Some(Status::Approved)
        })
        .collect();
    runs.sort_by(|a, b| {
        created_at(&a.entry)
            .cmp(&created_at(&b.entry))
            .then_with(|| number(&a.entry.id()).cmp(&number(&b.entry.id())))
            .then_with(|| a.entry.id().cmp(&b.entry.id()))
    });
    runs
}

/// The changes of each run in time order: for a run with a model, the parameters that differ from
/// the previous run with the same model (models compared ignoring case). A run without a model,
/// or the first run with its model, has none.
fn changes_by_run(runs: &[&Item]) -> Vec<Vec<Difference>> {
    let mut latest: HashMap<String, usize> = HashMap::new();
    let mut changes = Vec::with_capacity(runs.len());
    for (index, item) in runs.iter().enumerate() {
        let found = match model_key(&item.entry) {
            Some(key) => match latest.insert(key, index) {
                Some(earlier) => parameter_changes(&runs[earlier].entry, &item.entry),
                None => Vec::new(),
            },
            None => Vec::new(),
        };
        changes.push(found);
    }
    changes
}

/// The model of a run, lower case, when it names one.
fn model_key(entry: &Entry) -> Option<String> {
    entry
        .field("model")
        .filter(|model| !model.is_empty())
        .map(|model| model.to_lowercase())
}

/// The parameters that differ between `before` and `after`: a parameter that is set in one of them
/// only differs. The names come in the order of `before`, then the names only `after` has.
fn parameter_changes(before: &Entry, after: &Entry) -> Vec<Difference> {
    let earlier = parameters(before);
    let later = parameters(after);
    let mut names: Vec<String> = Vec::new();
    for (name, _) in earlier.iter().chain(&later) {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names
        .into_iter()
        .filter_map(|name| {
            let old = value_of(&earlier, &name);
            let new = value_of(&later, &name);
            (old != new).then_some(Difference {
                name,
                before: old,
                after: new,
            })
        })
        .collect()
}

/// The `name = value` lines of the Parameters section, as pairs. A line without `=` is skipped.
fn parameters(entry: &Entry) -> Vec<(String, String)> {
    let text = entry.section("Parameters").unwrap_or_default();
    text.lines()
        .filter_map(|line| {
            let (name, value) = line.split_once('=')?;
            let name = name.trim();
            (!name.is_empty()).then(|| (name.to_string(), value.trim().to_string()))
        })
        .collect()
}

/// The value of a parameter: the last one of that name, when there is one.
fn value_of(parameters: &[(String, String)], name: &str) -> Option<String> {
    parameters
        .iter()
        .rev()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
}

/// The lines of the Outputs section, trimmed, without the empty ones.
fn outputs_of(entry: &Entry) -> Vec<String> {
    let text = entry.section("Outputs").unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// The names of the fields of two records: `title` first, then the fields of the first record in
/// file order, then those only the second one has. The history is left out.
fn field_names(a: &Entry, b: &Entry) -> Vec<String> {
    let mut names = vec!["title".to_string()];
    for (key, _) in a.fields().into_iter().chain(b.fields()) {
        if key != SKIPPED_FIELD && !names.contains(&key) {
            names.push(key);
        }
    }
    names
}

/// The text of one run: `<date>  <author>  <ID>  <software> <version>  <model>`, then a
/// `changed:` line per change and an `outputs:` line when it has outputs. A value the record
/// does not give is `-`.
fn line(entry: &Entry, changes: &[Difference], outputs: &[String]) -> String {
    let date = entry
        .field("created")
        .map_or_else(|| "-".to_string(), |stamp| time::date(&stamp).to_string());
    let mut lines = vec![format!(
        "{date}  {}  {}  {} {}  {}",
        dash(entry.field("author").as_deref()),
        entry.id(),
        dash(entry.field("software").as_deref()),
        dash(entry.field("version").as_deref()),
        dash(entry.field("model").as_deref()),
    )];
    for change in changes {
        lines.push(format!(
            "  changed: {} {} -> {}",
            change.name,
            dash(change.before.as_deref()),
            dash(change.after.as_deref())
        ));
    }
    if !outputs.is_empty() {
        lines.push(format!("  outputs: {}", outputs.join(", ")));
    }
    lines.join("\n")
}

/// The data of one run: its fields, its changes and its outputs. A value the record does not
/// give is `null`.
fn row(entry: &Entry, changes: &[Difference], outputs: &[String]) -> Value {
    json!({
        "id": entry.id(),
        "created": entry.field("created"),
        "author": entry.field("author"),
        "via": entry.field("via"),
        "software": entry.field("software"),
        "version": entry.field("version"),
        "task": entry.field("task"),
        "model": entry.field("model"),
        "changed": differences_data(changes),
        "outputs": outputs,
    })
}

/// The data of a list of differences: one object per difference.
fn differences_data(differences: &[Difference]) -> Vec<Value> {
    differences
        .iter()
        .map(|difference| {
            json!({
                "name": difference.name,
                "before": difference.before,
                "after": difference.after,
            })
        })
        .collect()
}

/// Whether a run passes the filters: the task, the software and the model are equal ignoring
/// case, each only when it is given.
fn chosen(entry: &Entry, args: &Args, model: Option<&str>) -> bool {
    same_field(entry, "task", args.task.as_deref())
        && same_field(entry, "software", args.software.as_deref())
        && same_field(entry, "model", model)
}

/// Whether the field `key` equals `wanted` ignoring case; always true when nothing is wanted.
fn same_field(entry: &Entry, key: &str, wanted: Option<&str>) -> bool {
    wanted.is_none_or(|wanted| {
        entry
            .field(key)
            .is_some_and(|value| same(&value, wanted.trim()))
    })
}

/// Whether two texts are the same, ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The `created` field as an instant, when it is a time stamp (format.md §7).
fn created_at(entry: &Entry) -> Option<DateTime<FixedOffset>> {
    entry
        .field("created")
        .and_then(|stamp| DateTime::parse_from_rfc3339(&stamp).ok())
}

/// The number of an ID: 3 for USE-003, 0 when there is none.
fn number(id: &str) -> u64 {
    id.rsplit('-')
        .next()
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

/// The text of an optional value, `-` when it is not given.
fn dash(value: Option<&str>) -> &str {
    value.unwrap_or("-")
}

/// A warning of `store::all` (`<path>: <message>`) with its path shown relative to the project
/// folder, with `/` between the parts. A warning that does not start in the project folder is
/// returned as it is.
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
