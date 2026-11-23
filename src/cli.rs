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

/// Runs the program with the process arguments and returns its exit code.
///
/// A command run on a server is forwarded first (`remote::forward::maybe`). A parse error
/// prints clap's message and returns 2; `--help` and `--version` return 0.
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
        Command::Propose(a) => ("propose", cmd::propose::run(a, g)),
        Command::Accept(a) => ("accept", cmd::accept::run(a, g)),
        Command::Decline(a) => ("decline", cmd::decline::run(a, g)),
        Command::Confirm(a) => ("confirm", cmd::confirm::run(a, g)),
        Command::Refute(a) => ("refute", cmd::refute::run(a, g)),
        Command::Cleared(a) => ("cleared", cmd::cleared::run(a, g)),
        Command::Value(a) => ("value", cmd::value::run(a, g)),
        Command::Values(a) => ("values", cmd::values::run(a, g)),
        Command::Attach(a) => ("attach", cmd::attach::run(a, g)),
        Command::Detach(a) => ("detach", cmd::detach::run(a, g)),
        Command::Reattach(a) => ("reattach", cmd::reattach::run(a, g)),
        Command::Files(a) => ("files", cmd::files::run(a, g)),
        Command::Which(a) => ("which", cmd::which::run(a, g)),
        Command::Used(a) => ("used", cmd::used::run(a, g)),
        Command::Runs(a) => ("runs", cmd::runs::run(a, g)),
        Command::Image(a) => ("image", cmd::image::run(a, g)),
        Command::Received(a) => ("received", cmd::received::run(a, g)),
        Command::Delivered(a) => ("delivered", cmd::delivered::run(a, g)),
        Command::Exchange(a) => ("exchange", cmd::exchange::run(a, g)),
        Command::Map(a) => ("map", cmd::map::run(a, g)),
        Command::Hooks(a) => ("hooks", cmd::hooks::run(a, g)),
        Command::Guard(a) => ("guard", cmd::guard::run(a, g)),
        Command::Skill(a) => ("skill", cmd::skill::run(a, g)),
        Command::Team(a) => ("team", cmd::team::run(a, g)),
        Command::Confidential(a) => ("confidential", cmd::confidential::run(a, g)),
        Command::Promote(a) => ("promote", cmd::promote::run(a, g)),
        Command::Close(a) => ("close", cmd::close::run(a, g)),
        Command::Howto(a) => ("howto", cmd::howto::run(a, g)),
        Command::Tool(a) => ("tool", cmd::tool::run(a, g)),
        Command::WhereUsed(a) => ("where-used", cmd::where_used::run(a, g)),
        Command::Term(a) => ("term", cmd::term::run(a, g)),
        Command::Glossary(a) => ("glossary", cmd::glossary::run(a, g)),
        Command::Reviewed(a) => ("reviewed", cmd::reviewed::run(a, g)),
        Command::ReviewFolder(a) => ("review-folder", cmd::review_folder::run(a, g)),
        Command::ReviewDocument(a) => ("review-document", cmd::review_document::run(a, g)),
        Command::ReviewedDocument(a) => ("reviewed-document", cmd::reviewed_document::run(a, g)),
        Command::Import(a) => ("import", cmd::import::run(a, g)),
    };
    crate::output::emit(name, g.json, &result)
}
