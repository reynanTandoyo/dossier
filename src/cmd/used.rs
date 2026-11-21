//! The `used` command (docs/design.md section 9, format.md §15): records a run of a software in one
//! step (R-USE-2, R-USE-14). The usage record (USE-<n>, approved at once) keeps the software, its
//! version, the task, the model that was run with its checksum at that time, the parameters, the
//! outputs, the how-to followed and the deviations. In the same transaction the task's file list
//! (format.md §17) gets the model, with role `model`, when it is not attached yet, and each output,
//! with role `result` and made from `<model>@<checksum>`. An output that is attached already is
//! described again and gets a `reattached` history line.

use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::Kind;
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::tx::{self, Change};
use serde_json::json;
use std::path::Path;

/// The fields of a block that are given again when a detached file is attached again: the same
/// list as `attach` uses (format.md §17). A value the new attach does not give is removed.
const REFRESHED: [&str; 9] = [
    "type",
    "size",
    "files",
    "modified",
    "checksum",
    "role",
    "note",
    "made-from",
    "moved-from",
];

/// The fields that describe a path, recorded again for an output that is attached already.
const DESCRIBED: [&str; 5] = ["type", "size", "files", "modified", "checksum"];

/// The role of the simulation file that was run (format.md §17).
const MODEL_ROLE: &str = "model";

/// The role of the files a run wrote (format.md §17).
const RESULT_ROLE: &str = "result";

/// The arguments of `used`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The software that was run, such as ExampleSolver
    pub software: String,
    /// The version of the software that was run
    #[arg(long)]
    pub version: String,
    /// The task the run was for, such as T-042
    #[arg(long)]
    pub task: String,
    /// The simulation file that was run, as a path; it is attached with role model
    #[arg(long, value_name = "path")]
    pub model: Option<String>,
    /// A parameter of the run as name=value (repeatable)
    #[arg(long = "param", value_name = "name=value")]
    pub params: Vec<String>,
    /// A file or folder the run wrote, as a path (repeatable); it is attached with role result
    #[arg(long = "output", value_name = "path")]
    pub outputs: Vec<String>,
    /// The entry the run followed, such as LES-004
    #[arg(long, value_name = "ID")]
    pub howto: Option<String>,
    /// How the run differed from the how-to or the plan
    #[arg(long)]
    pub deviations: Option<String>,
    /// Any other note on the run
    #[arg(long)]
    pub note: Option<String>,
    /// The title; the default is "<software> <version> run for <task>"
    #[arg(long)]
    pub title: Option<String>,
}

/// A run as the command gives it, checked: the values the usage record and the file list take.
/// Paths are stored paths (R-XOS-2).
struct Run {
    software: String,
    version: String,
    task: String,
    title: String,
    howto: Option<String>,
    model: Option<String>,
    params: Vec<(String, String)>,
    outputs: Vec<String>,
    deviations: Option<String>,
    note: Option<String>,
}

/// Runs `used`: checks the run, then creates the usage record and attaches its files, in one
/// transaction. The first output line is `<USE-ID>: run recorded`; the data is {"id": "<USE-ID>"}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "used")?;
    let who = ctx.writer()?.clone();
    let cwd = std::env::current_dir()?;
    let run = check(&ctx, args, &cwd)?;

    tx::write(&ctx, |tx| {
        let dossier = tx.ctx().dossier.clone();
        let root = tx.ctx().root.clone();
        let time = tx.time().to_string();

        // The model as it is now: its checksum goes into the record, and its outputs are made
        // from `<model>@<checksum>`. A folder has no checksum, so its entry ends with `@`.
        let model_described = run
            .model
            .as_ref()
            .map(|path| filelist::describe(&root, path))
            .transpose()?;
        let checksum = model_described
            .as_ref()
            .map_or_else(String::new, |described| value_of(described, "checksum"));
        let made_from = run
            .model
            .as_ref()
            .map_or_else(String::new, |model| format!("{model}@{checksum}"));
        let howto = run
            .howto
            .as_ref()
            .map(|id| store::get(&dossier, id).map(|item| item.entry.id()))
            .transpose()?;

        let (mut file, old) = filelist::load(&dossier, &run.task)?;
        let before = attached_keys(&file);
        // Every output is described before an ID is given out, so that a path that does not exist
        // refuses the run with nothing numbered.
        let mut outputs = Vec::new();
        for output in &run.outputs {
            outputs.push((output, filelist::describe(&root, output)?));
        }

        let mut fields = vec![
            pair("software", &run.software),
            pair("version", &run.version),
            pair("task", &run.task),
        ];
        if let Some(id) = &howto {
            fields.push(pair("howto", id));
        }
        if let Some(model) = &run.model {
            fields.push(pair("model", model));
            fields.push(pair("model-checksum", &checksum));
        }
        let new = New {
            kind: Kind::Usage,
            title: run.title.clone(),
            fields,
            sections: sections_of(&run),
            said_by: None,
            draft: false,
        };
        let (id, _text) = create::create(tx, &new)?;

        // The model gets a block when it is not attached yet; each output gets a block, or is
        // described again when it is attached already.
        let mut new_paths: Vec<String> = Vec::new();
        let mut touched: Vec<String> = Vec::new();
        let new_model = run.model.as_ref().filter(|model| !before.contains(model));
        if let (Some(model), Some(described)) = (new_model, model_described) {
            let fields = block_fields(MODEL_ROLE, described, "");
            attach_block(&mut file, model, &fields, &time, &who);
            new_paths.push(model.clone());
            touched.push(model.clone());
        }
        for (output, described) in outputs {
            if before.contains(output) {
                refresh_block(&mut file, output, &described, &made_from, &time, &who);
            } else {
                let fields = block_fields(RESULT_ROLE, described, &made_from);
                attach_block(&mut file, output, &fields, &time, &who);
                new_paths.push(output.clone());
            }
            touched.push(output.clone());
        }

        let mut warnings: Vec<String> = new_paths
            .iter()
            .flat_map(|path| name_warnings(path))
            .collect();
        warnings.extend(case_warnings(&before, &new_paths));
        if !touched.is_empty() {
            let event = if new_paths.is_empty() {
                "reattached"
            } else {
                "attached"
            };
            tx.put_raw(
                filelist::path(&dossier, &run.task),
                old,
                file.text().into_bytes(),
                Change {
                    id: filelist::item(&run.task),
                    event: event.to_string(),
                    detail: touched.join(", "),
                },
            );
        }

        let text = format!("{id}: run recorded");
        Ok(with_warnings(
            Output::with(text, json!({"id": id})),
            warnings,
        ))
    })
}

/// Checks the arguments: the values are one line and not empty, the task is a task ID, the paths
/// are stored paths inside the project, and the model is not also an output (Code::BadValue).
fn check(ctx: &Ctx, args: &Args, cwd: &Path) -> Result<Run> {
    let software = single_line("the software", &args.software)?;
    let version = single_line("--version", &args.version)?;
    let task = task_id(&args.task)?;
    let title = match trimmed(args.title.as_deref()) {
        Some(title) => title.to_string(),
        None => format!("{software} {version} run for {task}"),
    };
    let howto = args
        .howto
        .as_deref()
        .map(|id| single_line("--howto", id))
        .transpose()?;
    let model = args
        .model
        .as_deref()
        .map(|typed| stored_path(ctx, cwd, typed))
        .transpose()?;
    let mut outputs: Vec<String> = Vec::new();
    for typed in &args.outputs {
        let stored = stored_path(ctx, cwd, typed)?;
        if !outputs.contains(&stored) {
            outputs.push(stored);
        }
    }
    if model.as_ref().is_some_and(|model| outputs.contains(model)) {
        return Err(bad("the model cannot also be an output"));
    }
    let params = args
        .params
        .iter()
        .map(|text| parameter_of(text))
        .collect::<Result<Vec<_>>>()?;
    Ok(Run {
        software,
        version,
        task,
        title,
        howto,
        model,
        params,
        outputs,
        deviations: trimmed(args.deviations.as_deref()).map(str::to_string),
        note: trimmed(args.note.as_deref()).map(str::to_string),
    })
}

/// The sections of the record in the order of format.md §15: Parameters (`name = value` per
/// line), Outputs (one stored path per line), Deviations and Notes. A section with nothing in it
/// is left out.
fn sections_of(run: &Run) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    if !run.params.is_empty() {
        let lines: Vec<String> = run
            .params
            .iter()
            .map(|(name, value)| format!("{name} = {value}"))
            .collect();
        sections.push(pair("Parameters", &lines.join("\n")));
    }
    if !run.outputs.is_empty() {
        sections.push(pair("Outputs", &run.outputs.join("\n")));
    }
    if let Some(text) = &run.deviations {
        sections.push(pair("Deviations", text));
    }
    if let Some(text) = &run.note {
        sections.push(pair("Notes", text));
    }
    sections
}

/// A `--param name=value`: the name and the value, each trimmed, one line and not empty.
fn parameter_of(text: &str) -> Result<(String, String)> {
    let Some((name, value)) = text.split_once('=') else {
        return Err(bad(format!("a parameter is written name=value: {text}")));
    };
    Ok((
        single_line("a parameter's name", name)?,
        single_line("a parameter's value", value)?,
    ))
}

/// Attaches `key`, which is not attached now: a new block with `fields`, or a detached block that
/// is brought back with its fields given again (as `attach` does). The history line says
/// `attached`.
fn attach_block(
    file: &mut BlockFile,
    key: &str,
    fields: &[(String, String)],
    time: &str,
    who: &Who,
) {
    if file.keys().iter().any(|known| known == key) {
        file.set_field(key, "detached", "");
        for name in REFRESHED {
            let value = fields
                .iter()
                .find(|(field, _)| field == name)
                .map_or("", |(_, value)| value.as_str());
            file.set_field(key, name, value);
        }
    } else {
        file.add_block(key, fields);
    }
    file.add_history(key, time, who, "attached", "");
}

/// Describes an output that is attached already again: its description is recorded, and its
/// made-from entry is set when the run has a model (a run without one keeps the old entry). The
/// history line says `reattached`.
fn refresh_block(
    file: &mut BlockFile,
    key: &str,
    described: &[(String, String)],
    made_from: &str,
    time: &str,
    who: &Who,
) {
    for name in DESCRIBED {
        file.set_field(key, name, &value_of(described, name));
    }
    if !made_from.is_empty() {
        file.set_field(key, "made-from", made_from);
    }
    file.add_history(key, time, who, "reattached", "");
}

/// The fields of a new block in the order of format.md §17: role, type, the other description
/// fields, made-from. Empty values are left out when the block is written.
fn block_fields(
    role: &str,
    described: Vec<(String, String)>,
    made_from: &str,
) -> Vec<(String, String)> {
    let (kind, rest): (Vec<_>, Vec<_>) =
        described.into_iter().partition(|(name, _)| name == "type");
    let mut fields = vec![pair("role", role)];
    fields.extend(kind);
    fields.extend(rest);
    fields.push(pair("made-from", made_from));
    fields
}

/// The keys of the blocks that are not detached: the files that belong to the task now.
fn attached_keys(file: &BlockFile) -> Vec<String> {
    file.keys()
        .into_iter()
        .filter(|key| !is_detached(file, key))
        .collect()
}

/// Whether the block of `key` has a detached mark.
fn is_detached(file: &BlockFile, key: &str) -> bool {
    file.field(key, "detached").is_some_and(|at| !at.is_empty())
}

/// The value of a named field of a described path, or the empty text.
fn value_of(described: &[(String, String)], name: &str) -> String {
    described
        .iter()
        .find(|(field, _)| field == name)
        .map_or_else(String::new, |(_, value)| value.clone())
}

/// The task as the list names it: upper-cased. A task is letters, digits, `-` and `_`, since its
/// name is a file name (Code::BadValue otherwise).
fn task_id(task: &str) -> Result<String> {
    let task = task.trim();
    if task.is_empty()
        || !task
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(bad(format!("a task is written like T-042: {task}")));
    }
    Ok(task.to_uppercase())
}

/// A typed path as stored (R-XOS-2). The project folder itself cannot be a file of a task, and a
/// path that begins or ends with a space or has a line break cannot be a block key.
fn stored_path(ctx: &Ctx, cwd: &Path, typed: &str) -> Result<String> {
    let stored = paths::stored(&ctx.root, cwd, typed)?;
    if stored.is_empty() {
        return Err(bad("the project folder itself is not a file of a task"));
    }
    if stored.trim() != stored || stored.contains(['\r', '\n']) {
        return Err(bad(format!(
            "a path cannot begin or end with a space or have a line break: {typed}"
        )));
    }
    Ok(stored)
}

/// The text trimmed, which must not be empty and must be one line (Code::BadValue).
fn single_line(what: &str, text: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() || text.contains(['\r', '\n']) {
        return Err(bad(format!("{what} must be one line and not empty")));
    }
    Ok(text.to_string())
}

/// The text with its ends trimmed, or None when it is not given or blank.
fn trimmed(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// Warnings for the names of a stored path that Windows cannot have (R-XOS-5): one warning for
/// each such name of the path.
fn name_warnings(stored: &str) -> Vec<String> {
    stored
        .split('/')
        .filter(|name| !paths::windows_safe(name))
        .map(|name| {
            format!(
                "{} has the name {name}, which Windows cannot have; the file cannot be copied to a PC",
                paths::shown(stored)
            )
        })
        .collect()
}

/// Warnings for paths that differ only in upper and lower case (R-XOS-5): each new path is
/// compared with the paths already in the list and with the new paths before it.
fn case_warnings(known: &[String], new: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    for (index, path) in new.iter().enumerate() {
        let earlier = known.iter().chain(&new[..index]);
        if let Some(other) = earlier
            .clone()
            .find(|other| *other != path && other.to_lowercase() == path.to_lowercase())
        {
            warnings.push(format!(
                "{} and {} differ only in upper and lower case; both cannot be copied to a PC",
                paths::shown(path),
                paths::shown(other)
            ));
        }
    }
    warnings
}

/// The output with each warning added, in order.
fn with_warnings(output: Output, warnings: Vec<String>) -> Output {
    warnings
        .into_iter()
        .fold(output, |output, warning| output.warn(warning))
}

/// A field or section as a pair of owned strings.
fn pair(key: &str, value: &str) -> (String, String) {
    (key.to_string(), value.to_string())
}

/// A failure for a value the person gave, Code::BadValue.
fn bad(message: impl Into<String>) -> Error {
    Error::new(Code::BadValue, message)
}
