//! The command line: the global options, the commands, and the dispatch to each command.

use crate::cmd;
use crate::error::Result;
use crate::output::Output;
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
    pub allow_credential_like: bool,
    #[arg(long, global = true)]
    pub project: Option<String>,
    /// Work on the team dossier (`init` names a team path with `--with-team` instead).
    #[arg(long, global = true)]
    pub team: bool,
    /// Entries written are imported drafts from this document.
    #[arg(long, global = true)]
    pub from_document: Option<String>,
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

/// The commands of v0.1, v0.2 and v0.3: one variant per file of `src/cmd/`, named after the command.
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
    Propose(cmd::propose::Args),
    Accept(cmd::accept::Args),
    Decline(cmd::decline::Args),
    Confirm(cmd::confirm::Args),
    Refute(cmd::refute::Args),
    Cleared(cmd::cleared::Args),
    Value(cmd::value::Args),
    Values(cmd::values::Args),
    Attach(cmd::attach::Args),
    Detach(cmd::detach::Args),
    Reattach(cmd::reattach::Args),
    Files(cmd::files::Args),
    Which(cmd::which::Args),
    Used(cmd::used::Args),
    Runs(cmd::runs::Args),
    Image(cmd::image::Args),
    Received(cmd::received::Args),
    Delivered(cmd::delivered::Args),
    Exchange(cmd::exchange::Args),
    Map(cmd::map::Args),
    Hooks(cmd::hooks::Args),
    Guard(cmd::guard::Args),
    Skill(cmd::skill::Args),
    Team(cmd::team::Args),
    Confidential(cmd::confidential::Args),
    Promote(cmd::promote::Args),
    Close(cmd::close::Args),
    Howto(cmd::howto::Args),
    Tool(cmd::tool::Args),
    WhereUsed(cmd::where_used::Args),
    Term(cmd::term::Args),
    Glossary(cmd::glossary::Args),
    Reviewed(cmd::reviewed::Args),
    ReviewFolder(cmd::review_folder::Args),
    ReviewDocument(cmd::review_document::Args),
    ReviewedDocument(cmd::reviewed_document::Args),
    Import(cmd::import::Args),
    Problem(cmd::problem::Args),
    Tried(cmd::tried::Args),
    Solved(cmd::solved::Args),
    GaveUp(cmd::gave_up::Args),
    Take(cmd::take::Args),
    Return(cmd::return_cmd::Args),
    ReviewSession(cmd::review_session::Args),
    Session(cmd::session::Args),
}

/// On Linux, files and folders this program creates are writable by the group (R-XOS-8): the
/// umask is set to 002, so new files get mode 664 and new folders 775. The folders `init`
/// creates carry the setgid bit, so new files and folders also keep the dossier's group.
#[cfg(unix)]
fn group_writable() {
    unsafe extern "C" {
        fn umask(mask: u32) -> u32;
    }
    // SAFETY: umask only sets the file mode mask of this process and cannot fail. mode_t is a
    // 32-bit unsigned integer on Linux.
    unsafe {
        umask(0o002);
    }
}

/// Windows has no file modes: nothing to do.
#[cfg(not(unix))]
fn group_writable() {}

/// The value of an environment variable, when it is set and not empty.
fn variable(name: &str) -> Option<String> {
    std::env::var_os(name)
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
}

/// Runs a command once the version check of the PC (R-XOS-9) has passed. An error of the check
/// is the result and the command does not run; a warning of the check is added to the output of
/// the command, and an error of the command stays as it is.
fn gated(check: &Result<Option<String>>, run: impl FnOnce() -> Result<Output>) -> Result<Output> {
    match check {
        Err(error) => Err(error.clone()),
        Ok(None) => run(),
        Ok(Some(warning)) => run().map(|output| output.warn(warning.clone())),
    }
}

/// Runs the program with the process arguments and returns its exit code.
///
/// A command run on a server is forwarded first (`remote::forward::maybe`). A parse error
/// prints clap's message and returns 2; `--help` and `--version` return 0. The version check of
/// the PC (`remote::version::check`) runs after the arguments are parsed and before the command.
pub fn main_exit() -> u8 {
    group_writable();
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
    let check = crate::remote::version::check(
        variable(crate::remote::version::CLIENT_VERSION).as_deref(),
        variable(crate::remote::version::CLIENT_FORMAT).as_deref(),
    );
    let (name, result) = match &cli.command {
        Command::Init(a) => ("init", gated(&check, || cmd::init::run(a, g))),
        Command::Where(a) => ("where", gated(&check, || cmd::where_cmd::run(a, g))),
        Command::People(a) => ("people", gated(&check, || cmd::people::run(a, g))),
        Command::Add(a) => ("add", gated(&check, || cmd::add::run(a, g))),
        Command::Decide(a) => ("decide", gated(&check, || cmd::decide::run(a, g))),
        Command::Found(a) => ("found", gated(&check, || cmd::found::run(a, g))),
        Command::Learn(a) => ("learn", gated(&check, || cmd::learn::run(a, g))),
        Command::Assume(a) => ("assume", gated(&check, || cmd::assume::run(a, g))),
        Command::Set(a) => ("set", gated(&check, || cmd::set::run(a, g))),
        Command::Retitle(a) => ("retitle", gated(&check, || cmd::retitle::run(a, g))),
        Command::Link(a) => ("link", gated(&check, || cmd::link::run(a, g))),
        Command::Edit(a) => ("edit", gated(&check, || cmd::edit::run(a, g))),
        Command::Show(a) => ("show", gated(&check, || cmd::show::run(a, g))),
        Command::List(a) => ("list", gated(&check, || cmd::list::run(a, g))),
        Command::Search(a) => ("search", gated(&check, || cmd::search::run(a, g))),
        Command::History(a) => ("history", gated(&check, || cmd::history::run(a, g))),
        Command::Diff(a) => ("diff", gated(&check, || cmd::diff::run(a, g))),
        Command::Restore(a) => ("restore", gated(&check, || cmd::restore::run(a, g))),
        Command::Log(a) => ("log", gated(&check, || cmd::log::run(a, g))),
        Command::ReviewTask(a) => ("review-task", gated(&check, || cmd::review_task::run(a, g))),
        Command::Drafts(a) => ("drafts", gated(&check, || cmd::drafts::run(a, g))),
        Command::Approve(a) => ("approve", gated(&check, || cmd::approve::run(a, g))),
        Command::Reject(a) => ("reject", gated(&check, || cmd::reject::run(a, g))),
        Command::Supersede(a) => ("supersede", gated(&check, || cmd::supersede::run(a, g))),
        Command::Retire(a) => ("retire", gated(&check, || cmd::retire::run(a, g))),
        Command::Remove(a) => ("remove", gated(&check, || cmd::remove::run(a, g))),
        Command::Reopen(a) => ("reopen", gated(&check, || cmd::reopen::run(a, g))),
        Command::Brief(a) => ("brief", gated(&check, || cmd::brief::run(a, g))),
        Command::Status(a) => ("status", gated(&check, || cmd::status::run(a, g))),
        Command::Check(a) => ("check", gated(&check, || cmd::check::run(a, g))),
        Command::Agents(a) => ("agents", gated(&check, || cmd::agents::run(a, g))),
        Command::Help(a) => ("help", gated(&check, || cmd::help::run(a, g))),
        Command::Pack(a) => ("pack", gated(&check, || cmd::pack::run(a, g))),
        Command::Unpack(a) => ("unpack", gated(&check, || cmd::unpack::run(a, g))),
        Command::Connect(a) => ("connect", gated(&check, || cmd::connect::run(a, g))),
        Command::Projects(a) => ("projects", gated(&check, || cmd::projects::run(a, g))),
        Command::Todo(a) => ("todo", gated(&check, || cmd::todo::run(a, g))),
        Command::Propose(a) => ("propose", gated(&check, || cmd::propose::run(a, g))),
        Command::Accept(a) => ("accept", gated(&check, || cmd::accept::run(a, g))),
        Command::Decline(a) => ("decline", gated(&check, || cmd::decline::run(a, g))),
        Command::Confirm(a) => ("confirm", gated(&check, || cmd::confirm::run(a, g))),
        Command::Refute(a) => ("refute", gated(&check, || cmd::refute::run(a, g))),
        Command::Cleared(a) => ("cleared", gated(&check, || cmd::cleared::run(a, g))),
        Command::Value(a) => ("value", gated(&check, || cmd::value::run(a, g))),
        Command::Values(a) => ("values", gated(&check, || cmd::values::run(a, g))),
        Command::Attach(a) => ("attach", gated(&check, || cmd::attach::run(a, g))),
        Command::Detach(a) => ("detach", gated(&check, || cmd::detach::run(a, g))),
        Command::Reattach(a) => ("reattach", gated(&check, || cmd::reattach::run(a, g))),
        Command::Files(a) => ("files", gated(&check, || cmd::files::run(a, g))),
        Command::Which(a) => ("which", gated(&check, || cmd::which::run(a, g))),
        Command::Used(a) => ("used", gated(&check, || cmd::used::run(a, g))),
        Command::Runs(a) => ("runs", gated(&check, || cmd::runs::run(a, g))),
        Command::Image(a) => ("image", gated(&check, || cmd::image::run(a, g))),
        Command::Received(a) => ("received", gated(&check, || cmd::received::run(a, g))),
        Command::Delivered(a) => ("delivered", gated(&check, || cmd::delivered::run(a, g))),
        Command::Exchange(a) => ("exchange", gated(&check, || cmd::exchange::run(a, g))),
        Command::Map(a) => ("map", gated(&check, || cmd::map::run(a, g))),
        Command::Hooks(a) => ("hooks", gated(&check, || cmd::hooks::run(a, g))),
        Command::Guard(a) => ("guard", gated(&check, || cmd::guard::run(a, g))),
        Command::Skill(a) => ("skill", gated(&check, || cmd::skill::run(a, g))),
        Command::Team(a) => ("team", gated(&check, || cmd::team::run(a, g))),
        Command::Confidential(a) => (
            "confidential",
            gated(&check, || cmd::confidential::run(a, g)),
        ),
        Command::Promote(a) => ("promote", gated(&check, || cmd::promote::run(a, g))),
        Command::Close(a) => ("close", gated(&check, || cmd::close::run(a, g))),
        Command::Howto(a) => ("howto", gated(&check, || cmd::howto::run(a, g))),
        Command::Tool(a) => ("tool", gated(&check, || cmd::tool::run(a, g))),
        Command::WhereUsed(a) => ("where-used", gated(&check, || cmd::where_used::run(a, g))),
        Command::Term(a) => ("term", gated(&check, || cmd::term::run(a, g))),
        Command::Glossary(a) => ("glossary", gated(&check, || cmd::glossary::run(a, g))),
        Command::Reviewed(a) => ("reviewed", gated(&check, || cmd::reviewed::run(a, g))),
        Command::ReviewFolder(a) => (
            "review-folder",
            gated(&check, || cmd::review_folder::run(a, g)),
        ),
        Command::ReviewDocument(a) => (
            "review-document",
            gated(&check, || cmd::review_document::run(a, g)),
        ),
        Command::ReviewedDocument(a) => (
            "reviewed-document",
            gated(&check, || cmd::reviewed_document::run(a, g)),
        ),
        Command::Import(a) => ("import", gated(&check, || cmd::import::run(a, g))),
        Command::Problem(a) => ("problem", gated(&check, || cmd::problem::run(a, g))),
        Command::Tried(a) => ("tried", gated(&check, || cmd::tried::run(a, g))),
        Command::Solved(a) => ("solved", gated(&check, || cmd::solved::run(a, g))),
        Command::GaveUp(a) => ("gave-up", gated(&check, || cmd::gave_up::run(a, g))),
        Command::Take(a) => ("take", gated(&check, || cmd::take::run(a, g))),
        Command::Return(a) => ("return", gated(&check, || cmd::return_cmd::run(a, g))),
        Command::ReviewSession(a) => (
            "review-session",
            gated(&check, || cmd::review_session::run(a, g)),
        ),
        Command::Session(a) => ("session", gated(&check, || cmd::session::run(a, g))),
    };
    crate::output::emit(name, g.json, &result)
}
