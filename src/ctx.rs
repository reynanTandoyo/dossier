//! The context of a command: the dossier it works on, its settings, and who acts.

use crate::cli::Global;
use crate::config::{self, Config};
use crate::error::{Code, Error, Result};
use crate::find;
use crate::names::{self, Who};
use std::path::PathBuf;

/// The name of the state folder inside the DOSSIER folder (format.md §9).
const STATE: &str = ".state";

/// The commands that change entries. Only they refuse a name that is not in the settings
/// (R-CFG-4: unknown names are refused for writing); a reading command runs without a name.
const WRITING: [&str; 16] = [
    "add",
    "decide",
    "found",
    "learn",
    "assume",
    "set",
    "retitle",
    "link",
    "edit",
    "restore",
    "approve",
    "reject",
    "supersede",
    "retire",
    "remove",
    "reopen",
];

/// The context of one command.
pub struct Ctx {
    /// The parent of `dossier`: the project folder.
    pub root: PathBuf,
    /// The DOSSIER folder.
    pub dossier: PathBuf,
    /// `dossier/.state`.
    pub state: PathBuf,
    pub cfg: Config,
    pub who: Option<Who>,
    pub json: bool,
    pub dry_run: bool,
    pub command: String,
}

impl Ctx {
    /// Finds the dossier from the current folder (or g.dossier), loads the settings, and
    /// resolves the name if one is given (no name is not an error here).
    pub fn load(g: &Global, command: &str) -> Result<Ctx> {
        let cwd = std::env::current_dir()?;
        // A relative --dossier is taken from the current folder, so that the root (the parent
        // of the dossier) is never empty. The path is not canonicalized: that would give
        // Windows paths in the \\?\ form.
        let explicit = g.dossier.as_ref().map(|path| cwd.join(path));
        let dossier = find::find(&cwd, explicit.as_deref())?;
        let root = match dossier.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                return Err(Error::new(
                    Code::NoDossier,
                    format!(
                        "the DOSSIER folder {} has no folder around it",
                        dossier.display()
                    ),
                ));
            }
        };
        let state = dossier.join(STATE);
        let cfg = config::load(&dossier)?;
        // A missing name is fine here. A name that is not in the settings is an error for a
        // command that writes, and no name at all for one that only reads.
        let who = match names::resolve(&cfg, g.agent.as_deref()) {
            Ok(who) => Some(who),
            Err(error) if error.code == Code::Usage => None,
            Err(error) if error.code == Code::UnknownName && !WRITING.contains(&command) => None,
            Err(error) => return Err(error),
        };
        Ok(Ctx {
            root,
            dossier,
            state,
            cfg,
            who,
            json: g.json,
            dry_run: g.dry_run,
            command: command.to_string(),
        })
    }

    /// The name that acts. Code::Usage "this command needs --agent" when there is none.
    pub fn writer(&self) -> Result<&Who> {
        self.who.as_ref().ok_or_else(|| {
            Error::new(
                Code::Usage,
                "this command needs a name: --agent <name> or DOSSIER_AGENT",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DOSSIER folder with the default settings and the person Anna (LLM anna-claude), in a
    /// temporary project folder whose name has a space and an umlaut. Returns the temporary
    /// folder (it keeps the folder alive) and the DOSSIER folder.
    fn project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::Builder::new()
            .prefix("Projekt Äpfel ")
            .tempdir()
            .expect("temporary folder");
        let dossier = dir.path().join("DOSSIER");
        std::fs::create_dir_all(&dossier).expect("create the DOSSIER folder");
        std::fs::write(dossier.join("dossier.toml"), config::default_text("P-001"))
            .expect("write dossier.toml");
        config::add_person(&dossier, "Anna", &["anna-claude".to_string()]).expect("add Anna");
        (dir, dossier)
    }

    #[test]
    fn load_gives_the_folders_the_settings_and_the_writer() {
        let (dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier.clone()),
            agent: Some("anna-claude".to_string()),
            json: true,
            dry_run: true,
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "add").expect("the dossier loads");
        assert_eq!(ctx.root, dir.path());
        assert_eq!(ctx.dossier, dossier);
        assert_eq!(ctx.state, dossier.join(".state"));
        assert_eq!(ctx.cfg.code, "P-001");
        let expected = Who {
            name: "anna-claude".to_string(),
            person: "Anna".to_string(),
            is_llm: true,
        };
        assert_eq!(ctx.who, Some(expected.clone()));
        assert_eq!(ctx.writer().expect("a writer"), &expected);
        assert!(ctx.json);
        assert!(ctx.dry_run);
        assert_eq!(ctx.command, "add");
    }

    #[test]
    fn an_unknown_name_is_an_error() {
        let (_dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier),
            agent: Some("carl".to_string()),
            ..Default::default()
        };
        let error = Ctx::load(&g, "add").err().expect("carl is refused");
        assert_eq!(error.code, Code::UnknownName);
    }

    #[test]
    fn an_unknown_name_does_not_stop_a_reading_command() {
        let (_dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier),
            agent: Some("carl".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "show").expect("show reads without a name");
        assert!(ctx.who.is_none());
    }

    #[test]
    fn a_missing_name_is_not_an_error_but_the_writer_is_needed() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ENV_LOCK is held, so no other test reads or changes the environment
        // meanwhile. A worker's shell may set these, so both start removed.
        unsafe {
            std::env::remove_var("DOSSIER_AGENT");
            std::env::remove_var("TODO_AGENT");
        }
        let (_dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "status").expect("a missing name is not an error");
        assert!(ctx.who.is_none());
        let error = ctx.writer().expect_err("a writer is needed");
        assert_eq!(error.code, Code::Usage);
        assert_eq!(
            error.message,
            "this command needs a name: --agent <name> or DOSSIER_AGENT"
        );
    }
}
