//! dossier: keeps what a team learns and decides in its projects.
//!
//! Every library module is declared here. The command line (`cli`, `cmd`) is added by the
//! next skeleton task.

// Temporary: every function is a stub until its task fills it in. The release task removes
// this line.
#![allow(dead_code, unused_variables)]

pub mod agents;
pub mod approval;
pub mod atomic;
pub mod brief_task;
pub mod cli;
pub mod cmd;
pub mod config;
pub mod create;
pub mod ctx;
pub mod entry;
pub mod error;
pub mod filter;
pub mod find;
pub mod hash;
pub mod ids;
pub mod journal;
pub mod lock;
pub mod names;
pub mod output;
pub mod paths;
pub mod remote;
pub mod select;
pub mod snapshot;
pub mod store;
pub mod tar;
pub mod term;
pub mod time;
pub mod todo_bridge;
pub mod tx;

// Every unit test that sets or removes an environment variable, or changes the current folder,
// holds this lock for its whole run (ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())),
// because unit tests run in parallel threads of one process.
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
