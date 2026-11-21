//! The `values` command (docs/design.md section 9, R-VALUE-4, R-LIFE-9): the values in force, one
//! line each, key values first and then by name. With `--on <date>` the values are those in force
//! at the end of that day, and a value that did not exist then is left out. A proposal waiting for
//! a person is shown only for today's values: a past day has no record of which proposal was
//! pending.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;
use crate::time;
use crate::values::{self, Value};
use chrono::NaiveDate;
use serde_json::json;

/// The arguments of `values`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Only the values whose name starts with this text, such as design
    pub prefix: Option<String>,
    /// The values in force at the end of this day (YYYY-MM-DD)
    #[arg(long, value_name = "date")]
    pub on: Option<String>,
}

/// One value in force, as the listing shows it.
struct Row {
    name: String,
    value: String,
    unit: Option<String>,
    source: Option<String>,
    valid_from: Option<String>,
    key: bool,
    proposed: Option<String>,
}

impl Row {
    /// "<value>[ <unit>]", as the file and the listing write a value.
    fn shown(&self) -> String {
        match &self.unit {
            Some(unit) => format!("{} {unit}", self.value),
            None => self.value.clone(),
        }
    }

    /// `<name>  <value>[ <unit>]  (<source>, <valid-from date>)`, then `  [proposed: <text>]` when a
    /// proposal waits. A missing source or date is left out; with neither, no brackets.
    fn line(&self) -> String {
        let from = self.valid_from.as_deref().map(time::date);
        let facts: Vec<&str> = [self.source.as_deref(), from]
            .into_iter()
            .flatten()
            .filter(|text| !text.is_empty())
            .collect();
        let mut line = format!("{}  {}", self.name, self.shown());
        if !facts.is_empty() {
            line.push_str(&format!("  ({})", facts.join(", ")));
        }
        if let Some(proposed) = &self.proposed {
            line.push_str(&format!("  [proposed: {proposed}]"));
        }
        line
    }

    /// The value as data: the fields of the line, with the full time the value came into force.
    fn data(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "value": self.value,
            "unit": self.unit,
            "source": self.source,
            "valid_from": self.valid_from,
            "key": self.key,
            "proposed": self.proposed,
        })
    }
}

/// Runs `values`: the values in force that start with the prefix, key values first, then a count.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "values")?;
    let day = args.on.as_deref().map(day_option).transpose()?;
    let (file, _) = values::load(&ctx.dossier, &ctx.cfg.code)?;
    let prefix = args.prefix.as_deref().unwrap_or("");
    let mut rows: Vec<Row> = values::all(&file)
        .iter()
        .filter(|value| value.name.starts_with(prefix))
        .filter_map(|value| match &day {
            Some(day) => row_on(&file, value, day),
            None => row_now(value),
        })
        .collect();
    rows.sort_by(|a, b| b.key.cmp(&a.key).then_with(|| a.name.cmp(&b.name)));

    let count = match rows.len() {
        1 => "1 value".to_string(),
        n => format!("{n} values"),
    };
    let mut lines: Vec<String> = rows.iter().map(Row::line).collect();
    lines.push(count);
    let data = serde_json::Value::Array(rows.iter().map(Row::data).collect());
    Ok(Output::with(lines.join("\n"), data))
}

/// The row of a value in force now. A block with no value yet, only a proposal, is not in force
/// and is left out.
fn row_now(value: &Value) -> Option<Row> {
    (!value.value.is_empty()).then(|| Row {
        name: value.name.clone(),
        value: value.value.clone(),
        unit: value.unit.clone(),
        source: value.source.clone(),
        valid_from: value.valid_from.clone(),
        key: value.key,
        proposed: value.proposed.clone(),
    })
}

/// The row of a value at the end of `day`: its value and unit as `values::on` gives them, and the
/// time of the `set` line in force. The source is kept only when no `set` or `changed` line is
/// dated after the day, since only those can change it and the history does not keep old sources
/// (format.md §17). The key flag is today's. No proposal: a past day has no record of one.
fn row_on(file: &BlockFile, value: &Value, day: &str) -> Option<Row> {
    let shown = values::on(file, &value.name, day)?;
    let (number, unit) =
        values::split(&shown, value.unit.is_none()).unwrap_or((shown.clone(), None));
    let history = history_events(file, &value.name);
    let valid_from = history
        .iter()
        .rfind(|(stamp, event)| event == "set" && time::date(stamp) <= day)
        .map(|(stamp, _)| stamp.clone());
    let source_kept = history.iter().all(|(stamp, event)| {
        !matches!(event.as_str(), "set" | "changed") || time::date(stamp) <= day
    });
    Some(Row {
        name: value.name.clone(),
        value: number,
        unit,
        source: value.source.clone().filter(|_| source_kept),
        valid_from,
        key: value.key,
        proposed: None,
    })
}

/// The time and the event of each history line of a value (format.md §6): the line is
/// `<time> <who>[ via <llm>]: <event>[ — <detail>]`, and the event is one word. Lines without
/// those parts are left out.
fn history_events(file: &BlockFile, name: &str) -> Vec<(String, String)> {
    file.history(name)
        .iter()
        .filter_map(|line| {
            let (stamp, rest) = line.split_once(' ')?;
            let (_who, rest) = rest.split_once(": ")?;
            let event = rest.split(' ').next()?;
            Some((stamp.to_string(), event.to_string()))
        })
        .collect()
}

/// The `--on` day as YYYY-MM-DD. Code::BadValue when it is not a date of that form.
fn day_option(text: &str) -> Result<String> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d")
        .map(|day| day.format("%Y-%m-%d").to_string())
        .map_err(|_| {
            Error::new(
                Code::BadValue,
                format!("--on needs a date such as 2026-11-21, not \"{text}\""),
            )
        })
}
