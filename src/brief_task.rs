//! The lines of `brief --task`: what a session needs to know about one task.

use crate::ctx::Ctx;
use crate::error::{Code, Error, Result};

/// The lines of `brief --task`: the task's title and goal lines from todo, the entries whose
/// `task` field names it, the decisions whose task is one of its milestones, then
/// select::best over the remaining approved entries with the task's machine, analysis and
/// software fields as tags and its title as words.
pub fn lines(ctx: &Ctx, task: &str, limit: usize) -> Result<Vec<String>> {
    Err(Error::new(Code::NotImplemented, "brief_task::lines"))
}
