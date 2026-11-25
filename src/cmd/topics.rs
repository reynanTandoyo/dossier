//! The `topics` command (docs/design.md section 21.2, R-TOPIC-1 to 5): prints the topics of the
//! project's list, one per line, and adds a topic to it with `topics add` (a person only). The
//! list is the top-level `topics` key of dossier.toml.

use crate::cli::Global;
use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};
use crate::output::Output;

/// The arguments of `topics`: with no action it lists the topics.
#[derive(clap::Args, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<Action>,
}

/// What `topics` does besides listing the topics.
#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// Adds a topic to the project's list; needs a person
    Add {
        /// The name of the topic, such as pumps
        name: String,
    },
}

/// Runs `topics`: the topics of the settings, one per line. `topics add` is not part of this
/// change yet and is refused with Code::NotImplemented.
pub fn run(args: &Args, g: &Global) -> Result<Output> {
    let ctx = Ctx::load(g, "topics")?;
    match &args.action {
        None => Ok(Output::text(ctx.cfg.topics.join("\n"))),
        Some(Action::Add { .. }) => Err(Error::new(
            Code::NotImplemented,
            "dossier topics add is not available yet",
        )),
    }
}
