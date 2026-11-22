//! The context of a command: the dossier it works on, its settings, and who acts.

use crate::cli::Global;
use crate::config::{self, Config, Person};
use crate::error::{Code, Error, Result};
use crate::find;
use crate::names::{self, Who};
use crate::paths;
use std::path::{Path, PathBuf};

/// The name of the state folder inside the DOSSIER folder (format.md §9).
const STATE: &str = ".state";

/// The name of the settings file inside a DOSSIER folder (format.md §10).
const SETTINGS: &str = "dossier.toml";

/// The `kind` of a team dossier (format.md §26). A dossier without a kind is a project.
const TEAM: &str = "team";

/// What a command that needs the team dossier says when the project has none (R-PLACE-8).
const NO_TEAM: &str = "this project has no team dossier; link one with: dossier team use <path>";

/// The commands that change the dossier. Only they refuse a name that is not in the settings
/// (R-CFG-4: unknown names are refused for writing); a reading command runs without a name.
const WRITING: [&str; 31] = [
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
    "propose",
    "accept",
    "decline",
    "confirm",
    "refute",
    "cleared",
    "value",
    "attach",
    "detach",
    "reattach",
    "used",
    "image",
    "received",
    "delivered",
    "map",
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
    /// The team dossier of a project (cfg.team), if it exists.
    pub team: Option<PathBuf>,
    /// `g.from_document`, as given.
    pub from_document: Option<String>,
    /// With `--team` from a project: that project's DOSSIER.
    pub project: Option<PathBuf>,
}

impl Ctx {
    /// Finds the dossier from the current folder (or g.dossier), loads the settings, and
    /// resolves the name if one is given (no name is not an error here).
    ///
    /// A project whose team dossier can be reached (R-PLACE-8) gets the team's members and
    /// plain names added to its own, in memory, so that team members can write in the project.
    /// With `--team` from a project the context becomes the team dossier's (R-PLACE-5); a team
    /// dossier that cannot be reached is Code::NoDossier naming its path. A team dossier ignores
    /// `--team`. The name is resolved last, against the settings as they are then.
    pub fn load(g: &Global, command: &str) -> Result<Ctx> {
        let cwd = std::env::current_dir()?;
        // A relative --dossier is taken from the current folder, so that the root (the parent
        // of the dossier) is never empty. The path is not canonicalized: that would give
        // Windows paths in the \\?\ form.
        let explicit = g.dossier.as_ref().map(|path| cwd.join(path));
        let dossier = find::find(&cwd, explicit.as_deref())?;
        let (root, state) = folders(&dossier)?;
        let cfg = config::load(&dossier)?;
        let mut ctx = Ctx {
            root,
            dossier,
            state,
            cfg,
            who: None,
            json: g.json,
            dry_run: g.dry_run,
            command: command.to_string(),
            team: None,
            from_document: g.from_document.clone(),
            project: None,
        };
        if ctx.cfg.kind != TEAM {
            if let Some(stored) = ctx.cfg.team.clone() {
                let path = ctx.root.join(paths::shown(&stored));
                if reachable(&path) {
                    let team = config::load(&path)?;
                    add_members_and_names(&mut ctx.cfg, &team);
                    ctx.team = Some(path);
                }
            }
            if g.team {
                if ctx.team.is_none()
                    && let Some(stored) = &ctx.cfg.team
                {
                    return Err(Error::new(
                        Code::NoDossier,
                        format!(
                            "{}: the team dossier cannot be reached",
                            ctx.root.join(paths::shown(stored)).display()
                        ),
                    ));
                }
                ctx = ctx.team_ctx()?;
            }
        }
        ctx.who = resolve_who(&ctx.cfg, g.agent.as_deref(), command)?;
        Ok(ctx)
    }

    /// The context of this project's team dossier, with the same who, json, dry_run, command and
    /// from_document. The team's settings gain this project's members, plain names and people,
    /// so that the people of the project may write there. Code::NoDossier when the project has
    /// no reachable team dossier.
    pub fn team_ctx(&self) -> Result<Ctx> {
        let Some(team) = self.team.clone() else {
            return Err(Error::new(Code::NoDossier, NO_TEAM));
        };
        let mut cfg = config::load(&team)?;
        add_everything(&mut cfg, &self.cfg);
        let (root, state) = folders(&team)?;
        Ok(Ctx {
            root,
            dossier: team,
            state,
            cfg,
            who: self.who.clone(),
            json: self.json,
            dry_run: self.dry_run,
            command: self.command.clone(),
            team: None,
            from_document: self.from_document.clone(),
            project: Some(self.dossier.clone()),
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

/// The parent folder of a DOSSIER folder (the project or team folder) and its state folder.
fn folders(dossier: &Path) -> Result<(PathBuf, PathBuf)> {
    match dossier.parent() {
        Some(root) => Ok((root.to_path_buf(), dossier.join(STATE))),
        None => Err(Error::new(
            Code::NoDossier,
            format!(
                "the DOSSIER folder {} has no folder around it",
                dossier.display()
            ),
        )),
    }
}

/// Whether `path` is a DOSSIER folder that can be reached: a folder with dossier.toml in it.
fn reachable(path: &Path) -> bool {
    path.is_dir() && path.join(SETTINGS).is_file()
}

/// Who acts, resolved against the settings as they are now. A missing name is fine here. A name
/// that is not in the settings is an error for a command that writes, and no name at all for one
/// that only reads.
fn resolve_who(cfg: &Config, flag: Option<&str>, command: &str) -> Result<Option<Who>> {
    match names::resolve(cfg, flag) {
        Ok(who) => Ok(Some(who)),
        Err(error) if error.code == Code::Usage => Ok(None),
        Err(error) if error.code == Code::UnknownName && !WRITING.contains(&command) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Adds the members and plain names of `from` that `into` does not have yet, ignoring case
/// (format.md §21). In memory only.
fn add_members_and_names(into: &mut Config, from: &Config) {
    add_missing(&mut into.members, &from.members);
    add_missing(&mut into.names, &from.names);
}

/// `add_members_and_names`, and also the people of `from` with their LLMs (format.md §21).
fn add_everything(into: &mut Config, from: &Config) {
    add_members_and_names(into, from);
    for person in &from.people {
        match into
            .people
            .iter_mut()
            .find(|known| same(&known.name, &person.name))
        {
            Some(known) => add_missing(&mut known.llms, &person.llms),
            None => into.people.push(Person {
                name: person.name.clone(),
                llms: person.llms.clone(),
            }),
        }
    }
}

/// Appends each of `new` that `list` does not hold yet, ignoring case.
fn add_missing(list: &mut Vec<String>, new: &[String]) {
    for value in new {
        if !list.iter().any(|known| same(known, value)) {
            list.push(value.clone());
        }
    }
}

/// Names are compared ignoring case.
fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
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

    /// The text of each value, as a list of settings.
    fn owned(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    /// A team dossier at `<dir>/<folder>/DOSSIER` with the given members and plain names. Returns
    /// its DOSSIER folder.
    fn team(dir: &Path, folder: &str, members: &[&str], names: &[&str]) -> PathBuf {
        let dossier = dir.join(folder).join("DOSSIER");
        std::fs::create_dir_all(&dossier).expect("create the team DOSSIER folder");
        let text = format!("kind = \"team\"\n{}", config::default_text("TEAM"));
        std::fs::write(dossier.join("dossier.toml"), text).expect("write the team's settings");
        config::set_list(&dossier, "members", &owned(members)).expect("set the members");
        config::set_list(&dossier, "names", &owned(names)).expect("set the names");
        dossier
    }

    /// Makes the project at `dossier` name the team dossier at `stored`.
    fn link(dossier: &Path, stored: &str) {
        let value = format!("\"{}\"", stored.replace('\\', "\\\\").replace('"', "\\\""));
        config::set_value(dossier, "team", &value).expect("set the team path");
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
    fn load_keeps_the_document_and_leaves_team_and_project_empty() {
        let (_dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier),
            from_document: Some("docs/Bericht Ä.pdf".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "show").expect("the dossier loads");
        assert_eq!(ctx.from_document.as_deref(), Some("docs/Bericht Ä.pdf"));
        assert!(ctx.team.is_none());
        assert!(ctx.project.is_none());
        assert_eq!(ctx.cfg.kind, "project");
        let error = ctx.team_ctx().err().expect("a project without a team");
        assert_eq!(error.code, Code::NoDossier);
        assert_eq!(error.message, NO_TEAM);
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
        // meanwhile. A worker's shell may set these, and the operating-system user is a name
        // too, so all four start removed.
        unsafe {
            std::env::remove_var("DOSSIER_AGENT");
            std::env::remove_var("TODO_AGENT");
            std::env::remove_var("USERNAME");
            std::env::remove_var("USER");
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

    #[test]
    fn a_project_with_a_team_path_gets_the_team_members_and_names() {
        let (dir, dossier) = project();
        config::set_list(&dossier, "members", &owned(&["Mia"])).expect("set the members");
        team(dir.path(), "TEAM", &["tony", "mia"], &["Dr. Weber"]);
        link(&dossier, "TEAM/DOSSIER");
        let g = Global {
            dossier: Some(dossier.clone()),
            agent: Some("tony".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "add").expect("the project loads");
        assert_eq!(ctx.dossier, dossier);
        assert_eq!(ctx.team, Some(dir.path().join("TEAM").join("DOSSIER")));
        // "mia" is the same member as "Mia": it is not added twice.
        assert_eq!(ctx.cfg.members, owned(&["Mia", "tony"]));
        assert_eq!(ctx.cfg.names, owned(&["Dr. Weber"]));
        assert!(ctx.project.is_none());
        // A team member writes in the project, as itself and as an LLM of its own.
        let tony = Who {
            name: "tony".to_string(),
            person: "tony".to_string(),
            is_llm: false,
        };
        assert_eq!(ctx.writer().expect("tony writes"), &tony);
        let g = Global {
            dossier: Some(dossier),
            agent: Some("tony - claude".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "add").expect("the project loads");
        assert_eq!(
            ctx.who,
            Some(Who {
                name: "tony - claude".to_string(),
                person: "tony".to_string(),
                is_llm: true,
            })
        );
    }

    #[test]
    fn a_team_path_that_cannot_be_reached_leaves_the_project_working() {
        let (dir, dossier) = project();
        // A folder that does not exist, then a folder without dossier.toml.
        std::fs::create_dir_all(dir.path().join("EMPTY").join("DOSSIER"))
            .expect("create an empty folder");
        for stored in ["NOWHERE/DOSSIER", "EMPTY/DOSSIER"] {
            link(&dossier, stored);
            let g = Global {
                dossier: Some(dossier.clone()),
                agent: Some("anna-claude".to_string()),
                ..Default::default()
            };
            let ctx = Ctx::load(&g, "add").expect("the project still works");
            assert!(ctx.team.is_none());
            assert_eq!(
                ctx.who.as_ref().map(|who| who.person.as_str()),
                Some("Anna")
            );
            let error = ctx.team_ctx().err().expect("no team dossier");
            assert_eq!(error.code, Code::NoDossier);
            assert_eq!(error.message, NO_TEAM);

            // --team names the path that cannot be reached.
            let g = Global { team: true, ..g };
            let error = Ctx::load(&g, "add").err().expect("--team needs the team");
            assert_eq!(error.code, Code::NoDossier);
            assert_eq!(
                error.message,
                format!(
                    "{}: the team dossier cannot be reached",
                    dir.path().join(paths::shown(stored)).display()
                )
            );
        }
    }

    #[test]
    fn team_flag_without_a_team_path_says_the_project_has_no_team() {
        let (_dir, dossier) = project();
        let g = Global {
            dossier: Some(dossier),
            team: true,
            agent: Some("anna-claude".to_string()),
            ..Default::default()
        };
        let error = Ctx::load(&g, "add")
            .err()
            .expect("a project without a team");
        assert_eq!(error.code, Code::NoDossier);
        assert_eq!(error.message, NO_TEAM);
    }

    #[test]
    fn team_flag_switches_to_the_team_dossier_and_sets_the_project() {
        let (dir, dossier) = project();
        config::set_list(&dossier, "members", &owned(&["Mia"])).expect("set the members");
        let team_dossier = team(dir.path(), "TEAM", &["tony"], &["Dr. Weber"]);
        link(&dossier, "TEAM/DOSSIER");
        let g = Global {
            dossier: Some(dossier.clone()),
            team: true,
            agent: Some("tony".to_string()),
            json: true,
            dry_run: true,
            from_document: Some("docs/Bericht Ä.pdf".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "decide").expect("the team dossier loads");
        assert_eq!(ctx.dossier, team_dossier);
        assert_eq!(ctx.root, dir.path().join("TEAM"));
        assert_eq!(ctx.state, team_dossier.join(".state"));
        assert_eq!(ctx.cfg.kind, "team");
        assert_eq!(ctx.cfg.members, owned(&["tony", "Mia"]));
        assert_eq!(ctx.cfg.names, owned(&["Dr. Weber"]));
        assert_eq!(ctx.project, Some(dossier));
        assert!(ctx.team.is_none());
        assert_eq!(
            ctx.who,
            Some(Who {
                name: "tony".to_string(),
                person: "tony".to_string(),
                is_llm: false,
            })
        );
        assert!(ctx.json);
        assert!(ctx.dry_run);
        assert_eq!(ctx.command, "decide");
        assert_eq!(ctx.from_document.as_deref(), Some("docs/Bericht Ä.pdf"));
    }

    #[test]
    fn team_flag_lets_a_person_of_the_project_write_in_the_team() {
        let (dir, dossier) = project();
        team(dir.path(), "TEAM", &["tony"], &[]);
        link(&dossier, "TEAM/DOSSIER");
        let g = Global {
            dossier: Some(dossier),
            team: true,
            agent: Some("anna-claude".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "decide").expect("the team dossier loads");
        assert_eq!(
            ctx.who,
            Some(Who {
                name: "anna-claude".to_string(),
                person: "Anna".to_string(),
                is_llm: true,
            })
        );
    }

    #[test]
    fn team_flag_on_a_team_dossier_changes_nothing() {
        let (dir, _dossier) = project();
        let team_dossier = team(dir.path(), "TEAM", &["tony"], &["Dr. Weber"]);
        let g = Global {
            dossier: Some(team_dossier.clone()),
            team: true,
            agent: Some("tony".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "decide").expect("the team dossier loads");
        assert_eq!(ctx.dossier, team_dossier);
        assert_eq!(ctx.cfg.kind, "team");
        assert!(ctx.project.is_none());
        assert!(ctx.team.is_none());
        assert_eq!(ctx.who.as_ref().map(|who| who.name.as_str()), Some("tony"));
    }

    #[test]
    fn team_ctx_is_the_team_with_the_same_who_and_the_project_set() {
        let (dir, dossier) = project();
        let team_dossier = team(dir.path(), "TEAM", &["tony"], &[]);
        link(&dossier, "TEAM/DOSSIER");
        let g = Global {
            dossier: Some(dossier.clone()),
            agent: Some("anna-claude".to_string()),
            json: true,
            dry_run: true,
            from_document: Some("docs/Bericht Ä.pdf".to_string()),
            ..Default::default()
        };
        let ctx = Ctx::load(&g, "promote").expect("the project loads");
        let team_ctx = ctx.team_ctx().expect("the team dossier");
        assert_eq!(team_ctx.dossier, team_dossier);
        assert_eq!(team_ctx.root, dir.path().join("TEAM"));
        assert_eq!(team_ctx.project, Some(dossier));
        assert!(team_ctx.team.is_none());
        assert_eq!(team_ctx.who, ctx.who);
        assert_eq!(
            team_ctx.who,
            Some(Who {
                name: "anna-claude".to_string(),
                person: "Anna".to_string(),
                is_llm: true,
            })
        );
        assert!(team_ctx.json);
        assert!(team_ctx.dry_run);
        assert_eq!(team_ctx.command, "promote");
        assert_eq!(
            team_ctx.from_document.as_deref(),
            Some("docs/Bericht Ä.pdf")
        );
        assert!(
            team_ctx
                .cfg
                .people
                .iter()
                .any(|person| person.name == "Anna")
        );
    }
}
