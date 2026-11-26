# Worker rules (dossier help worker)

- You write nothing to the dossier: run no `dossier` command that records, changes or approves anything.
- Say what you learned, had to assume or could not do in your todo note: `todo done <id> --note "<what>"` or `todo stuck <id> --note "<what>"`. The planner records it.
- After `todo claim`, the entries for your task are printed (or run `dossier brief --task <id>`). They bind your work; `dossier show <ID>` prints one in full.
- Never edit files under `DOSSIER/`. When your first edit of another file is refused with a list of entries, that is not an error: read them, then make the same edit again.
