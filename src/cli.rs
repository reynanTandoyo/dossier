//! The command line: the global options, the commands, and the dispatch to each command.

use crate::cmd;
use clap::Parser;
use std::path::PathBuf;

/// The global options of every command.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct Global {
    #[arg(long, global = true)]
    pub dossier: Option<PathBuf>,
    #[arg(long, global = true)]
    pub agent: Option<String>,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub dry_run: bool,
    #[arg(long, global = true)]
    pub allow_network_drive: bool,
    #[arg(long, global = true)]
    pub project: Option<String>,
}

/// The whole command line: the global options and one command.
#[derive(clap::Parser, Debug)]
#[command(name = "dossier", version, disable_help_subcommand = true)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,
    #[command(subcommand)]
    pub command: Command,
}

/// The commands of v0.1: one variant per file of `src/cmd/`, named after the command.
#[derive(clap::Subcommand, Debug)]
// The command line is parsed once per run, so the size of the largest variant does not matter.
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Init(cmd::init::Args),
    Where(cmd::where_cmd::Args),
    People(cmd::people::Args),
    Add(cmd::add::Args),
    Decide(cmd::decide::Args),
    Found(cmd::found::Args),
    Learn(cmd::learn::Args),
    Assume(cmd::assume::Args),
    Set(cmd::set::Args),
    Retitle(cmd::retitle::Args),
    Link(cmd::link::Args),
    Edit(cmd::edit::Args),
    Show(cmd::show::Args),
    List(cmd::list::Args),
    Search(cmd::search::Args),
    History(cmd::history::Args),
    Diff(cmd::diff::Args),
    Restore(cmd::restore::Args),
    Log(cmd::log::Args),
    ReviewTask(cmd::review_task::Args),
    Drafts(cmd::drafts::Args),
    Approve(cmd::approve::Args),
    Reject(cmd::reject::Args),
    Supersede(cmd::supersede::Args),
    Retire(cmd::retire::Args),
    Remove(cmd::remove::Args),
    Reopen(cmd::reopen::Args),
    Brief(cmd::brief::Args),
    Status(cmd::status::Args),
    Check(cmd::check::Args),
    Agents(cmd::agents::Args),
    Help(cmd::help::Args),
    Pack(cmd::pack::Args),
    Unpack(cmd::unpack::Args),
    Connect(cmd::connect::Args),
    Projects(cmd::projects::Args),
    Todo(cmd::todo::Args),
}

/// Runs the program with the process arguments and returns its exit code.
///
/// A command run on a server is forwarded first (`remote::forward::maybe`). A parse error
/// prints clap's message and returns 2; `--help` and `--version` return 0.
pub fn main_exit() -> u8 {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = crate::remote::forward::maybe(&args) {
        return code;
    }
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 2,
            };
        }
    };
    let g = &cli.global;
    let (name, result) = match &cli.command {
        Command::Init(a) => ("init", cmd::init::run(a, g)),
        Command::Where(a) => ("where", cmd::where_cmd::run(a, g)),
        Command::People(a) => ("people", cmd::people::run(a, g)),
        Command::Add(a) => ("add", cmd::add::run(a, g)),
        Command::Decide(a) => ("decide", cmd::decide::run(a, g)),
        Command::Found(a) => ("found", cmd::found::run(a, g)),
        Command::Learn(a) => ("learn", cmd::learn::run(a, g)),
        Command::Assume(a) => ("assume", cmd::assume::run(a, g)),
        Command::Set(a) => ("set", cmd::set::run(a, g)),
        Command::Retitle(a) => ("retitle", cmd::retitle::run(a, g)),
        Command::Link(a) => ("link", cmd::link::run(a, g)),
        Command::Edit(a) => ("edit", cmd::edit::run(a, g)),
        Command::Show(a) => ("show", cmd::show::run(a, g)),
        Command::List(a) => ("list", cmd::list::run(a, g)),
        Command::Search(a) => ("search", cmd::search::run(a, g)),
        Command::History(a) => ("history", cmd::history::run(a, g)),
        Command::Diff(a) => ("diff", cmd::diff::run(a, g)),
        Command::Restore(a) => ("restore", cmd::restore::run(a, g)),
        Command::Log(a) => ("log", cmd::log::run(a, g)),
        Command::ReviewTask(a) => ("review-task", cmd::review_task::run(a, g)),
        Command::Drafts(a) => ("drafts", cmd::drafts::run(a, g)),
        Command::Approve(a) => ("approve", cmd::approve::run(a, g)),
        Command::Reject(a) => ("reject", cmd::reject::run(a, g)),
        Command::Supersede(a) => ("supersede", cmd::supersede::run(a, g)),
        Command::Retire(a) => ("retire", cmd::retire::run(a, g)),
        Command::Remove(a) => ("remove", cmd::remove::run(a, g)),
        Command::Reopen(a) => ("reopen", cmd::reopen::run(a, g)),
        Command::Brief(a) => ("brief", cmd::brief::run(a, g)),
        Command::Status(a) => ("status", cmd::status::run(a, g)),
        Command::Check(a) => ("check", cmd::check::run(a, g)),
        Command::Agents(a) => ("agents", cmd::agents::run(a, g)),
        Command::Help(a) => ("help", cmd::help::run(a, g)),
        Command::Pack(a) => ("pack", cmd::pack::run(a, g)),
        Command::Unpack(a) => ("unpack", cmd::unpack::run(a, g)),
        Command::Connect(a) => ("connect", cmd::connect::run(a, g)),
        Command::Projects(a) => ("projects", cmd::projects::run(a, g)),
        Command::Todo(a) => ("todo", cmd::todo::run(a, g)),
    };
    crate::output::emit(name, g.json, &result)
}
