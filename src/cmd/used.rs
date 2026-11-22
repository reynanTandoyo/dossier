//! The `used` command (docs/design.md section 9, format.md §15): records a run of a software in one
//! step (R-USE-2, R-USE-14). The usage record (USE-<n>, approved at once) keeps the software, its
//! version, the task, the model that was run with its checksum at that time, the parameters, the
//! outputs, the how-to followed and the deviations. In the same transaction the task's file list
//! (format.md §17) gets the model, with role `model`, when it is not attached yet, and each output,
//! with role `result` and made from `<model>@<checksum>`. An output that is attached already is
//! described again and gets a `reattached` history line. Then, when the project has a team
//! dossier, the team's usage index gets one anonymous line for the run in a transaction of its own
//! (R-USE-3, `index_in_team`): nothing of the title, the task or the files. A deviation of a run
//! that followed a team how-to (`team:<ID>`, or a plain ID that only the team dossier has) also adds
//! a line to that how-to's Open remarks in the team dossier (R-USE-6, `remark_in_team`), after the
//! confidential check, which refuses the whole run before anything is written.

use crate::approval::{self, Mark};
use crate::blockfile::BlockFile;
use crate::cli::Global;
use crate::conf;
use crate::config::Config;
use crate::create::{self, New};
use crate::ctx::Ctx;
use crate::entry::{Entry, Kind};
use crate::error::{Code, Error, Result};
use crate::filelist;
use crate::names::Who;
use crate::output::Output;
use crate::paths;
use crate::store;
use crate::teamfiles;
use crate::time;
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

/// The file of the team dossier that holds the usage index (format.md §20).
const USAGE_FILE: &str = "usage-index.md";

/// The item of the usage index in snapshots and the journal (format.md §17).
const USAGE_ITEM: &str = "USAGE";

/// The section of a how-to where the deviations reported on it are kept (format.md §22).
const REMARKS: &str = "Open remarks";

/// The place that `--howto` takes before an ID to name an entry of the team dossier (format.md §23).
const TEAM_PLACE: &str = "team:";

/// The how-to a run followed, resolved before anything is written.
struct Followed {
    /// The `howto` field of the record: the entry's ID, or `team:<ID>` for a team how-to.
    field: String,
    /// The ID of the team's how-to, when the run followed one (R-USE-6).
    team: Option<String>,
}

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
/// transaction. Then the team's usage index gets the run's line (`index_in_team`), and a deviation
/// of a team how-to goes to its Open remarks (`remark_in_team`). The first output line is
/// `<USE-ID>: run recorded`; the data is {"id": "<USE-ID>"}.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "used")?;
    let who = ctx.writer()?.clone();
    let cwd = std::env::current_dir()?;
    let run = check(&ctx, args, &cwd)?;
    let followed = followed_of(&ctx, run.howto.as_deref())?;
    // The deviation of a team how-to is checked before anything is written (R-USE-6, R-CONF-2).
    let remark = remark_of(&ctx, &run, followed.as_ref())?;

    // The record of the run as it was written, kept for the team's usage index.
    let mut recorded: Option<(Entry, String)> = None;
    let mut output = tx::write(&ctx, |tx| {
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
        if let Some(followed) = &followed {
            fields.push(pair("howto", &followed.field));
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
        let (id, file_text) = create::create(tx, &new)?;
        recorded = Some((Entry::parse(&file_text)?, time.clone()));

        // The model gets a block when it is not attached yet; each output gets a block, or is
        // described again when it is attached already.
        let mut new_paths: Vec<String> = Vec::new();
        let mut touched: Vec<String> = Vec::new();
        // A model that is attached already and changed since is described again: the run used
        // this version, so the list says it is current.
        if let (Some(model), Some(described)) = (run.model.as_ref(), model_described) {
            if !before.contains(model) {
                let fields = block_fields(MODEL_ROLE, described, "");
                attach_block(&mut file, model, &fields, &time, &who);
                new_paths.push(model.clone());
                touched.push(model.clone());
            } else if value_of(&described, "checksum")
                != file.field(model, "checksum").unwrap_or_default()
            {
                refresh_block(&mut file, model, &described, "", &time, &who);
                touched.push(model.clone());
            }
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
    })?;
    if let Some((entry, time)) = &recorded {
        for warning in index_in_team(&ctx, &who, entry, time) {
            output = output.warn(warning);
        }
    }
    if let Some((id, line)) = &remark {
        for warning in remark_in_team(&ctx, id, line) {
            output = output.warn(warning);
        }
    }
    Ok(output)
}

/// Adds the run's line to the team's usage index (R-USE-3, format.md §20) in a transaction of its
/// own on the team dossier. Nothing when the project has no team dossier. A team that cannot be
/// reached, a write it refuses and a line that the index has already are warnings: the run stays
/// recorded. Returns the warnings to add to the output of the command.
fn index_in_team(ctx: &Ctx, who: &Who, entry: &Entry, time: &str) -> Vec<String> {
    let Some(stored) = ctx.cfg.team.as_deref() else {
        return Vec::new();
    };
    let outcome = match &ctx.team {
        Some(_) => ctx
            .team_ctx()
            .and_then(|team| write_index(&team, ctx, who, entry, time)),
        None => Err(Error::new(
            Code::NoDossier,
            format!(
                "{}: the team dossier cannot be reached",
                ctx.root.join(paths::shown(stored)).display()
            ),
        )),
    };
    match outcome {
        Ok(output) => output.warnings,
        Err(error) => vec![format!(
            "the run is recorded, but not in the team's usage index: {}",
            error.message
        )],
    }
}

/// Writes the run's block into the team's usage index, in one transaction on the team dossier: the
/// block is keyed `<project code>:<USE-ID>` and has the history line `used` (format.md §20). A key
/// the index has already is refused, since a second block with it would make the index unreadable.
fn write_index(team: &Ctx, project: &Ctx, who: &Who, entry: &Entry, time: &str) -> Result<Output> {
    let key = format!("{}:{}", project.cfg.code, entry.id());
    let fields = index_fields(&project.cfg, who, entry, time);
    tx::write(team, |tx| {
        let (mut file, old) = teamfiles::usage(&team.dossier, &team.cfg.code)?;
        if file.keys().contains(&key) {
            return Err(Error::new(
                Code::Exists,
                format!("the team's usage index has {key} already"),
            ));
        }
        file.add_block(&key, &fields);
        file.add_history(&key, tx.time(), who, "used", "");
        tx.put_raw(
            team.dossier.join(USAGE_FILE),
            old,
            file.text().into_bytes(),
            Change {
                id: USAGE_ITEM.to_string(),
                event: "used".to_string(),
                detail: key.clone(),
            },
        );
        Ok(Output::text(format!(
            "{key}: added to the team's usage index"
        )))
    })
}

/// The fields of the team's line for a run (format.md §20): the date of the run, the project code,
/// the software and its version, the analysis and the machine (the record's own, else the project's
/// tags), the how-to followed and the person who ran it. No title, task or file name goes into the
/// index (R-USE-3).
fn index_fields(cfg: &Config, who: &Who, entry: &Entry, time: &str) -> Vec<(String, String)> {
    vec![
        pair("date", time::date(time)),
        pair("code", &cfg.code),
        pair("software", &field_of(entry, "software")),
        pair("version", &field_of(entry, "version")),
        pair(
            "analysis",
            &field_or_tags(entry, "analysis", &cfg.tags.analysis),
        ),
        pair(
            "machine",
            &field_or_tags(entry, "machine", &cfg.tags.machine),
        ),
        pair("howto", &field_of(entry, "howto")),
        pair("person", &who.person),
    ]
}

/// The value of a field of the record, or the empty text.
fn field_of(entry: &Entry, key: &str) -> String {
    entry.field(key).unwrap_or_default()
}

/// The value of a field of the record when it has one, else the project's tags of that kind,
/// joined with commas.
fn field_or_tags(entry: &Entry, key: &str, tags: &[String]) -> String {
    match entry.field(key) {
        Some(value) if !value.is_empty() => value,
        _ => tags.join(", "),
    }
}

/// The how-to that `--howto` names, resolved before anything is written. `team:<ID>` (any case)
/// names a how-to of the team dossier. A plain ID names the project's entry when the project has
/// one; otherwise it names the team's how-to with that ID, when the team has one. Otherwise the
/// project's entry must exist (Code::NotFound, as before).
fn followed_of(ctx: &Ctx, typed: Option<&str>) -> Result<Option<Followed>> {
    let Some(typed) = typed else {
        return Ok(None);
    };
    if let Some(id) = team_place(typed) {
        return team_howto(ctx, id).map(Some);
    }
    match store::get(&ctx.dossier, typed) {
        Ok(item) => Ok(Some(Followed {
            field: item.entry.id(),
            team: None,
        })),
        Err(error) if error.code == Code::NotFound => match team_howto(ctx, typed) {
            Ok(followed) => Ok(Some(followed)),
            Err(_) => Err(error),
        },
        Err(error) => Err(error),
    }
}

/// The team's how-to with `id`, which must be a how-to of the team dossier. Its record field is
/// `team:<ID>`. Code::NoDossier when the team dossier cannot be reached.
fn team_howto(ctx: &Ctx, id: &str) -> Result<Followed> {
    let team = team_dossier(ctx)?;
    let item = store::get(team, id)?;
    let id = item.entry.id();
    if item.entry.kind() != Some(Kind::Howto) {
        return Err(bad(format!("{id} in the team dossier is not a how-to")));
    }
    Ok(Followed {
        field: format!("{TEAM_PLACE}{id}"),
        team: Some(id),
    })
}

/// The team dossier of the project, for a how-to that is named with `team:` or that only the team
/// has. Code::NoDossier: the path when the team dossier it names cannot be reached, else how to link
/// one. A team dossier that is not needed for a plain ID is not an error (see `followed_of`).
fn team_dossier(ctx: &Ctx) -> Result<&Path> {
    if let Some(team) = &ctx.team {
        return Ok(team.as_path());
    }
    Err(match &ctx.cfg.team {
        Some(stored) => Error::new(
            Code::NoDossier,
            format!(
                "{}: the team dossier cannot be reached",
                ctx.root.join(paths::shown(stored)).display()
            ),
        ),
        None => Error::new(
            Code::NoDossier,
            "this project has no team dossier; link one with: dossier team use <path>",
        ),
    })
}

/// The ID after the place `team:` (any case), or None when the ID has no such place.
fn team_place(typed: &str) -> Option<&str> {
    let (place, id) = typed.split_at_checked(TEAM_PLACE.len())?;
    place.eq_ignore_ascii_case(TEAM_PLACE).then_some(id)
}

/// The deviation of a run as one line, with the ID of the team's how-to it followed (R-USE-6). Only a
/// run with a deviation that followed a team how-to has one. The line is checked against the
/// project's confidential terms first (conf::gate and conf::check, R-CONF-2): a refusal refuses the
/// whole run, and nothing is written.
fn remark_of(
    ctx: &Ctx,
    run: &Run,
    followed: Option<&Followed>,
) -> Result<Option<(String, String)>> {
    let (Some(id), Some(deviation)) = (
        followed.and_then(|followed| followed.team.as_deref()),
        run.deviations.as_deref(),
    ) else {
        return Ok(None);
    };
    let line = one_line(deviation);
    let terms = conf::gate(&ctx.dossier, &ctx.cfg.code)?;
    conf::check(&terms, &line)?;
    Ok(Some((id.to_string(), line)))
}

/// Adds the deviation `line` to the Open remarks of the team's how-to `id`. A team that cannot be
/// written, a writer who is not a team member and a write that is refused are warnings: the run
/// stays recorded. Returns the warnings to add to the output of the command.
fn remark_in_team(ctx: &Ctx, id: &str, line: &str) -> Vec<String> {
    match write_remark(ctx, id, line) {
        Ok(output) => output.warnings,
        Err(error) => vec![format!(
            "the run is recorded, but its deviation is not in the how-to's Open remarks: {}",
            error.message
        )],
    }
}

/// Writes `line` as the last line of the Open remarks of the team's how-to `id`, in a transaction on
/// the team dossier. A missing section is made last (format.md §22). The history line says `remark`
/// with the project code as its detail. When the how-to was unchanged since its approval, its
/// approval record is renewed with the new text, so that the remark is not a hand change.
fn write_remark(ctx: &Ctx, id: &str, line: &str) -> Result<Output> {
    let team = ctx.team_ctx()?;
    let who = team.writer()?.clone();
    let code = ctx.cfg.code.clone();
    let mut renewed: Option<(String, String)> = None;
    let mut output = tx::write(&team, |tx| {
        let item = store::get(&team.dossier, id)?;
        let how_to = item.entry.id();
        let mut entry = item.entry.clone();
        let remark = format!("- {code} {}: {line}", time::date(tx.time()));
        let body = match entry.section(REMARKS) {
            Some(body) if !body.is_empty() => format!("{body}\n{remark}"),
            _ => remark,
        };
        entry.set_section(REMARKS, &body);
        entry.add_history(tx.time(), &who, "remark", &code);
        let text = entry.text();
        if matches!(
            approval::mark(&team.state, &how_to, &item.raw),
            Mark::Unchanged
        ) {
            renewed = Some((how_to.clone(), text.clone()));
        }
        tx.put(
            item.path.clone(),
            Some(&item),
            text,
            Change {
                id: how_to.clone(),
                event: "remark".to_string(),
                detail: code.clone(),
            },
        );
        Ok(Output::text(format!("{how_to}: remark added")))
    })?;
    if let Some((how_to, text)) = renewed
        && !team.dry_run
        && let Err(error) = approval::record(&team.state, &how_to, text.as_bytes())
    {
        output = output.warn(format!(
            "the approval record of {how_to} is not renewed: {}",
            error.message
        ));
    }
    Ok(output)
}

/// The text as one line: its lines trimmed and joined with a space, blank lines left out.
fn one_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
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
