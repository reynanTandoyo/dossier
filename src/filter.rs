//! Filters: the options that choose entries for `list`, `search` and the other commands.

use crate::store::Item;

/// The filter options. Without `--status`, `--drafts` and `--all`, only approved entries match.
#[derive(clap::Args, Debug, Default, Clone)]
pub struct Filter {
    #[arg(long)]
    kind: Option<String>,
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    machine: Option<String>,
    #[arg(long)]
    analysis: Option<String>,
    #[arg(long)]
    software: Option<String>,
    #[arg(long)]
    tag: Option<String>,
    #[arg(long)]
    author: Option<String>,
    #[arg(long)]
    task: Option<String>,
    #[arg(long)]
    since: Option<String>,
    #[arg(long = "where")]
    conditions: Vec<String>,
    #[arg(long)]
    drafts: bool,
    #[arg(long)]
    all: bool,
}

impl Filter {
    /// Whether `item` passes the filter.
    pub fn matches(&self, item: &Item) -> bool {
        false
    }
}
