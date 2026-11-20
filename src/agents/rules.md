# Working with the dossier

- Use one unique name per agent: pass `--agent <name>` or set `DOSSIER_AGENT`.
- Start a session with `dossier brief`.
- Before a task, run `dossier brief --task <id>`.
- Before work that no task asks for, run `dossier status`.
- Work only on a task you have claimed in `todo`.
- Decisions in force bind you: if your task contradicts one, stop and ask your person; never work around it.
- When your person decides something, record it at once: `dossier decide "<decision>" --reason "<why>" --said-by <person>`.
- If `dossier decide` lists related decisions, run it again with `--replaces <ID>` when yours replaces one, or `--besides` when both stay in force.
- Record what you observe with `dossier found "<observation>"`, and what was tried and failed with `dossier found "<what was tried>" --dead-end`.
- Record what you had to assume with `dossier assume "<assumption>" --reason "<why>"`.
- After `todo done <id>`, run `dossier review-task <id>`, then propose lessons with `dossier learn` (it needs `--valid-for`, `--evidence` and `--confidence`), or propose none if nothing was learned.
- Search before you propose something new: `dossier search <words>`.
- Write every entry in English, also when the session is in another language.
- Never edit files under `DOSSIER/` directly; use the commands.
- Never approve, reject, retire or remove an entry: a person does that.
- Exit code `1`: a rule refused the change; read the message and do not repeat the command unchanged.
- Exit code `3`: busy; run the same command again.
