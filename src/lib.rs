//! dossier: keeps what a team learns and decides in its projects.
//!
//! Every module is declared here: the library modules, the command line (`cli`) and the
//! commands (`cmd`).

pub mod affects;
pub mod agents;
pub mod approval;
pub mod atomic;
pub mod blockfile;
pub mod brief_task;
pub mod cli;
pub mod cmd;
pub mod conf;
pub mod config;
pub mod create;
pub mod ctx;
pub mod entry;
pub mod error;
pub mod exchange;
pub mod filelist;
pub mod filesum;
pub mod filter;
pub mod find;
pub mod folderscan;
pub mod gitcommit;
pub mod glossary;
pub mod hash;
pub mod hooks;
pub mod ids;
pub mod imports;
pub mod journal;
pub mod lock;
pub mod mapfile;
pub mod names;
pub mod output;
pub mod paths;
pub mod picture;
pub mod remote;
pub mod secret;
pub mod select;
pub mod snapshot;
pub mod store;
pub mod tar;
pub mod teamfiles;
pub mod term;
pub mod time;
pub mod todo_bridge;
pub mod tx;
pub mod values;

// Every unit test that sets or removes an environment variable, or changes the current folder,
// holds this lock for its whole run (ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())),
// because unit tests run in parallel threads of one process.
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
