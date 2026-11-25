# dossier — requirements

Draft 12, 2026-11-25 (owner's decisions: D28 to D53). Four points are still open (§12); everything else is decided (§3) until the
owner says otherwise.

Each requirement has a priority, which says how important it is. When it is built is decided
separately, by the six phases and the patch release 0.3.1 in §11.

| Priority | Meaning |
|---|---|
| **MUST** | the tool is not complete without it |
| **SHOULD** | makes the tool clearly better |
| **COULD** | a valuable extra |
| Idea (§13) | recorded, not planned |

---

## 1. What `dossier` is

`dossier` is a command-line program that keeps what a small engineering team learns and decides
while doing its projects, so that neither the people nor their LLMs have to rely on memory.

The team does turbomachinery design and simulation for customers. Each customer project has a
folder on a Linux machine. The tasks of a project are managed by [`todo`](https://github.com/reynanTandoyo/todo)
in that folder's `TODO.md`. `dossier` sits next to it and manages the knowledge:

- **Decisions**: what was chosen, why, and what was rejected.
- **Findings**: what was observed in this project.
- **Lessons**: what holds beyond this project, with the conditions under which it holds.
- **How-tos**: how a certain software is used for a certain task.
- **Scripts**: the Java macros and Python scripts the team has, what each does and where it is.
- **Files of a task**: which simulation file, geometry, description and results belong to it.
- **Assumptions**: what was assumed because it wasn't known, and whether it was confirmed.
- **Key values**: the project's important numbers with their units, in one place.
- **Exchange with the customer**: what was received and what was delivered, when, in which
  version.
- **Usage records**: which software and version was used for which task in which project.

There are two places. Each project folder has a **project dossier** (`DOSSIER/`) that may contain
customer data. One **team dossier**, shared by all projects, holds everything reusable and
contains no customer data.

`dossier` does three jobs: it writes entries (directly, or as drafts that a person approves), it
finds them again, and it prints the entries that matter for the session or task at hand, so that
an LLM starts with the right context.

## 2. Who uses it, and the situations it must handle

- **Team members**: engineers (three at the start). Each works on a Windows PC and reaches the
  project folders over ssh or a network share. Each has their own LLM.
- **LLM agents**: Claude Code and others, working for one team member in one session. They read
  entries, write decisions the person made in the session, and propose drafts.
- **Project lead**: the team member who plans a project and closes it.
- **Later readers**: a team member or an LLM who opens a project after months, or starts a
  similar one.

| # | Situation | Requirements |
|---|---|---|
| S1 | A person and their LLM decide something in a session; a colleague's LLM knows it at its next session start | R-WRITE-2, R-CTX-1 |
| S2 | A task is finished; what was learned is proposed, and a person approves or rejects it within a minute | R-DRAFT-1 … R-DRAFT-5 |
| S3 | A lesson from customer A's project is reused in customer B's project without revealing customer A | R-CONF-1 … R-CONF-4, R-TEAM-3 |
| S4 | A person starts a task and is shown how the team did this kind of analysis with this software before | R-CTX-2, R-USE-4 |
| S5 | A lesson is applied outside the conditions it was found under | R-ENTRY-6, R-CHECK-1 |
| S6 | A lesson turns out to be wrong or outdated | R-LIFE-4, R-LIFE-5 |
| S7 | Two people or LLMs write entries at the same moment | R-LOCK-1, R-ID-1 |
| S8 | An LLM writes something wrong into the dossier | R-DRAFT-4, R-LIFE-2, R-HIST-1 |
| S9 | Two people copy the same large simulation file to their PCs and both copy it back | R-FILE-1 … R-FILE-4 |
| S10 | A person had the insight while working in the simulation program, with no LLM session | R-WRITE-7 |
| S11 | A person opens a project after two years and wants to know why it was done this way | R-READ-6, R-READ-7 |
| S12 | A person on a Windows PC runs a command for a project folder on the Linux machine | R-REMOTE-1 … R-REMOTE-5 |
| S13 | Someone edits an entry file by hand in an editor | R-EDIT-3, R-LIFE-6 |
| S15 | An assumption turns out to be wrong after results were built on it | R-WRITE-15, R-EXCH-4 |
| S16 | The customer asks which data a report from last spring was based on | R-EXCH-2, R-LIFE-9 |
| S17 | An LLM uses a mass flow from an old document instead of the current one | R-VALUE-1 … R-VALUE-4 |
| S18 | The same failed approach is tried again in the next project | R-ENTRY-15, R-CTX-16 |
| S19 | The project is a year old and its decisions no longer fit into an LLM's context | R-CTX-14, R-CTX-15 |
| S20 | The server's disk fails | R-BACKUP-1 … R-BACKUP-3 |
| S21 | The team has years of finished projects whose knowledge is only in reports and heads | R-IMPORT-1 … R-IMPORT-8 |
| S22 | The evidence for a lesson is a plot | R-ENTRY-16 … R-ENTRY-18 |
| S23 | A session is held in German, and a colleague later searches in English | R-ENTRY-19, R-TERM-4 |
| S24 | A how-to for the licence server is written with the password in it | R-CONF-7 |
| S25 | An LLM claims a task and starts without reading what the dossier holds for it | R-AGENT-5 |
| S26 | An LLM's first solver run blows up; it tests three hypotheses before the third one fixes it, and the next LLM must not test the first two again | R-INV-1 … R-INV-5, R-INV-10 |
| S27 | An engineer looks for a first converging solution, tries three changes at the simulation program, and the order in which they were made is what worked | R-INV-3, R-INV-4, R-INV-8 |
| S14 | Three LLMs work for three people in one project; each must know the state of the whole project and stay inside what was decided | R-CTX-9 … R-CTX-13 |

---

## 3. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | `dossier` is its own program. It uses `todo` (0.8 or newer) by running it with `--json`; `todo` knows nothing about `dossier`. | `todo` edits one file under a lock; `dossier` manages many files and search. `todo` stays usable alone. |
| D2 | One entry is one Markdown file, readable and correctable in any editor. No database. | Engineering knowledge must stay readable for years, without the tool. |
| D3 | Two places: a project dossier in each project folder, and one team dossier. Reusable knowledge is written to the team dossier; the project dossier keeps what is specific to the customer. | Knowledge locked in one project folder is lost to the next project. Customer data must not spread. |
| D4 | Seven kinds of entry: decision, assumption, finding, lesson, how-to, script, usage record. | They differ in who approves them, what they must contain and how long they stay true. |
| D5 | An entry is either **approved** or a **draft**. What a person states is approved at once; what an LLM concludes on its own is a draft until a person approves it. Only approved entries are shown as knowledge. | A wrong engineering rule costs more than a missing one. |
| D6 | Approving needs a person at a terminal. An LLM can't approve. | Otherwise D5 means nothing. |
| D7 | `dossier` never calls a model, just like `todo`: the model calls the tool. Drafts are written by the LLM that is already in the session, using material and rules that `dossier` prints. | No network code, no keys, no cost outside the session, and it works with every LLM. |
| D8 | The team dossier holds no customer data. Projects appear there only under a project code. Each project lists its confidential terms, and writing to the team dossier is refused while a term appears in the text. | The team works for several customers under confidentiality agreements. |
| D9 | One copy of each dossier, on the Linux machine, changed only by commands that run on that machine. PCs reach it through ssh (remote mode). | The lock works only between programs on the same machine, and one copy needs no syncing. |
| D10 | Entry IDs are a kind prefix and a number (`LES-031`), given out under the lock, never reused. From outside, an ID is written with its place: `team:LES-031`, `P-017:FND-004`. | Short enough to say and cite. One copy with a lock makes numbers safe. |
| D11 | Search is full-text plus fields. No embeddings. | A few thousand entries don't need them, and they would add a model to every write. |
| D12 | Every change is recorded in a journal in the dossier's state folder. git is optional. | The project folders are not git repositories today, and the history must exist anyway. |
| D13 | Every write needs a name (`--agent` or `DOSSIER_AGENT`, falling back to `TODO_AGENT`). The team settings say which names are people and which LLM belongs to whom. | Each entry must show whether a person or an LLM wrote it, and for whom. |
| D14 | Agents learn the tool from one rules text (`dossier help agents`, installed into `AGENTS.md`). Claude Code hooks are an extra. No MCP server. | Same reasons as in `todo`: every agent can run a command, and hooks can only run commands. |
| D15 | Large simulation files are never stored or copied by the dossier itself. It records who took which file and checks before a file is returned. | The files are gigabytes; the risk is overwriting a colleague's work, not losing history. |
| D18 | The dossier points to files; it doesn't hold them. Each task has a list of its files with a role for each (model, geometry, result, …), kept as paths relative to the project folder. | A reader must find the simulation file, the geometry and the results of a task without searching the disk, and the files stay where the programs expect them. |
| D20 | An entry is never overwritten to change what it says. It is closed with an end date and replaced, and every entry carries the dates between which it was in force. | "What did we know and decide on the day the report went out" must have an answer. (Idea taken from the Graphiti project.) |
| D21 | A decision can be proposed by anyone, person or LLM. Only the project lead, or a person the lead names, accepts it. | Three people and three LLMs must be able to suggest a direction without it becoming binding by accident. (Idea taken from decision-record tools such as MADR.) |
| D22 | When a `todo` milestone is closed, its decisions and findings are summed up in a digest. Afterwards `brief` shows the digest instead of the single entries. | The context an LLM gets must stay small as a project grows. |
| D23 | The project's key numbers live in one sheet of named values with units. Everything else refers to a value by name. | One number written in five places is soon five numbers, and an LLM can't tell which one is current. |
| D24 | The dossier depends on the server's backup. `dossier` offers a command that packs a dossier into one archive, and nothing more. | One copy and no git (D9, D12) means the backup is the only protection against losing the disk. |
| D25 | Every entry is written in English, whatever language the session was held in. | Almost all customers are not German-speaking, and word search only works when everyone uses the same words. |
| D26 | Small pictures (plots, contour images) that an entry shows are copied into the dossier. Everything else is still only pointed to (D18). | A lesson whose plot was deleted with the results folder has lost its evidence. Pictures are small; simulation files are not. |
| D27 | Finished projects are brought in the same way new knowledge is: `dossier` prints the material, the LLM in the session drafts, a person approves. Imported entries are always drafts first and stay marked as imported. | The dossier must not start empty, and what is reconstructed from an old folder is less certain than what was recorded at the time. |
| D19 | A script (a Java macro, a Python script) is an entry of its own kind that points to the script file and says what it does, for which software, and what it needs. The file itself lives in the team's script folder or repository. | Scripts are reused more than anything else, and a script nobody can find or trust is rewritten. Code belongs where it can be versioned and run. |
| D16 | Rust 1.89 or newer, edition 2024. One program for Linux x64 (the server, where the work is done) and one for Windows 10/11 x64 (remote mode, and local dossiers). Same code rules as `todo`. | One toolchain for all the team's tools. |
| D17 | UTF-8 only. Each line keeps its own line ending; bytes the tool doesn't change stay identical. | Files are edited by hand on Windows and Linux. |
| D28 | One list of people who decide, `deciders` in the project settings. No separate project lead. With an empty list everyone listed in the project decides, as in v0.1. (Owner, 2026-11-21.) | Simple to understand and to set up; one list covers accepting proposals and closing a project. |
| D29 | Literally every change is saved with who and when: entries, values, file lists, exchange records, usage records, settings changes (`init`, `people add`), the agent rules block and the register on the PC. (Owner, 2026-11-21.) | Nothing in the dossier may appear without a trace of who made it and when. |
| D30 | Knowledge (text) can be promoted to the team dossier; files (simulation files, CAD parts, spreadsheets, scripts) are never copied, only pointed to from their project. Small pictures are the exception of D26. (Owner, 2026-11-21.) | The team dossier stays small and free of customer files. |
| D31 | Scripts, commands on the server, installed software and git repositories are one entry kind, a *tool* (`TOOL-001`), with a `type` (`script`, `command`, `software`, `repo`) and a `location`. It replaces the separate software list, script entries and `script_root` of D19 and R-USE-1, R-USE-8 to R-USE-13, which are rewritten when phase 3 starts. The team standardises its tools in the meantime. (Owner, 2026-11-21.) | The team has no single place or form for its tools yet; one flexible kind fits all of them. |
| D32 | Anyone in the project may add a confidential term, also an LLM when its person says so in the session; removing a term needs a person at a terminal. There are no automatic terms; writing to the team dossier is refused until the project has at least one term or a person stated that there are none. (Owner, 2026-11-21.) | The tool cannot guess a customer's names; the refusal makes sure nobody forgets the list. |
| D33 | The Claude Code hooks (`brief` at session start, a guard against direct edits of `DOSSIER/`) and the skill come in phase 2; the session-end review (R-DRAFT-9) stays in phase 6 (numbered 5 before D45). Export (R-OUT-7) writes Markdown only. Review dates (R-LIFE-7) have no default: they are set per entry. (Owner, 2026-11-21.) | The hooks make the tool part of every session from the start. |
| D34 | The team is a list of usernames (the operating-system login of each member), plus a separate list of plain names: people who are not users (an external colleague, a customer contact) and can be named as author, source or `--said-by`, but never write themselves. An LLM is named `<username> - <anything>`, for example `tony - claude opus 5.5 high`; LLM names are not registered. Without `--agent` or `DOSSIER_AGENT` the writer is the operating-system username. `hooks install` sets `DOSSIER_AGENT` to `<username> - claude` in the user's personal Claude Code settings (`.claude/settings.local.json`), so that Claude's writes are always marked as an LLM's. Names registered with `people add --llm` keep working. Replaces the registration of LLM names of D13. (Owner, 2026-11-21.) | Simple to set up: nobody keeps a list of LLM names, and a person's own commands need no name at all. |
| D35 | Software names stay free text; R-USE-1's refusal of unknown names is dropped for now and may come back when the team has standardised its tools. (Owner, 2026-11-21.) | The team has no standard list yet. |
| D36 | Project folders and the team dossier are git repositories. `dossier` commits its own changes in the team dossier (R-TEAM-6) and never pushes; in a project folder `DOSSIER/` is committed with the project's work, like `TODO.md`. (Owner, 2026-11-21.) | One history for the work, the tasks and the knowledge. |
| D37 | A tool (D31) has a name, required, and a location, optional: a path, a command or a URL when one is known. Nothing is checked for a tool without a location. (Owner, 2026-11-21.) | Many tools have no single place yet. |
| D38 | `dossier` keeps the knowledge, `todo` keeps the work, side by side. `dossier` never creates, changes, claims or closes a task and never writes `TODO.md`; it reads tasks only through `todo --json`, and `dossier todo` only passes a command through to `todo` on the server. `dossier`'s register on the PC (`connect`, `projects`) holds the server connection for remote mode and is separate from `todo`'s project register (`todo project`, `todo overview`). Every later feature keeps to this. (Owner, 2026-11-21.) | Two tools that each do one thing work well together; overlapping features would drift apart. |
| D39 | A rule for agents that a `todo` command can trigger is carried out by a Claude Code hook and not left to the agent's memory: after a task is claimed, finished or given back, a hook prints what `dossier` has for that moment. The hook only reads the `todo` command that was run; it changes no task (D38). (Owner, 2026-11-22.) | A rule in a text is followed only some of the time; a hook runs every time. (Idea taken from the ECC project.) |
| D40 | Text that looks like a credential (a private key, an access token, a password) is refused on every write, in a project dossier and in the team dossier. (Owner, 2026-11-22.) | Dossiers are committed to git (D36), where a credential stays in the history. How-tos about licence servers and ssh are where one would be written down. (Idea taken from the ECC project.) |
| D41 | Trial and error is recorded in an eighth kind of entry, the *investigation* (`INV-001`): one problem, the attempts in the order they were made, each with its hypothesis and its result, and what solved it. It serves people and LLMs alike. (Owner, 2026-11-22.) | Single dead ends lose the order and the link to what finally worked, and the order is often the recipe: a change that failed first may work after another one. |
| D42 | An attempt is a line in an investigation, not a `todo` task. Only an attempt that is real work (long compute, another person, a plan) is a task; a person or the planner creates it with `todo`, and the investigation points to it. `dossier` never creates it (D38). (Owner, 2026-11-22.) | Most attempts take minutes inside a task that is already claimed; a task for each would be too heavy to be written, and workers would be planning. |
| D43 | A command from a PC makes its own ssh connection; nothing is kept open in the background. Reading through the network share with a file browser or an editor stays possible at any time (R-XOS-1). R-REMOTE-7 is not built. (Owner, 2026-11-23.) | The ssh program of Windows cannot share a connection, and a helper process on every PC would be more to install and to go wrong than the wait it saves. |
| D44 | The session review reads a session file in `dossier`'s own format, which has a version number like the entry format. A transcript of Claude Code is first converted to it; only that converter knows Claude Code's format. (Owner, 2026-11-23.) | Claude Code's transcript format is internal and changes between releases. With an own format the review works for every LLM, and a change at Claude Code touches one converter. |
| D45 | The order of the releases after 0.3: refusing credentials (R-CONF-7) is released alone as 0.3.1; investigations (D41, D42) become phase 4 (v0.4); working from the PC moves to phase 5 (v0.5) and growing projects to phase 6 (v0.6). (Owner, 2026-11-23.) | The credential check protects every dossier already in use, so it does not wait for a larger release. |
| D46 | `take` from a PC never overwrites a local file that already exists: it keeps the local copy and says that deleting it and running the same `take` copies the server's version. A copy goes to `<name>.dossier-part` first and is renamed only when complete, so a failed copy leaves no half file. (Owner, 2026-11-23.) | A repeated `take` by mistake must not wipe local edits, and a retry after a failed copy still works. |
| D47 | 0.5.0 is released without the manual test on the real server (TEST-10). It is run before `take` and `return` are used on real work. (Owner, 2026-11-23.) | The team does not use the large-file feature yet; the automatic tests, and an end-to-end run with real ssh and scp against a test server in WSL, pass. |
| D48 | Finding by word stems and rare words: wherever entries are chosen as related to a text, and in search, words are compared by their English word stem, and a word that few entries hold counts more than a common one (R-FIND-1 to R-FIND-4). This is still full-text search: no model is called and nothing is stored next to the entries, as D7 and D11 require. (Owner, 2026-11-25.) | In the project's own dossier, `search connection reuse` found nothing although two entries are about it, and `brief --task` listed entries that shared only the word "and" with the task. |
| D49 | Word stems come from the library rust-stemmers (the Snowball English stemmer, MIT/BSD licence); dossier carries no stemmer of its own. (Owner, 2026-11-25.) | A tested stemmer handles English endings correctly; an own one would be smaller but make more mistakes, and finding is what version 1.0 is for. |
| D50 | Each project has one short list of topics, and entries name their topics from it. Only a person adds a topic (`topics add`); an LLM that finds no fitting topic asks its person. (Owner, 2026-11-25.) | A short list stays useful for finding; LLMs adding topics would fill it with near-duplicates. |
| D51 | An LLM is shown the entries in force that bind a file before it changes the file: its first edit of the file in a session is refused once, naming those entries, and its next edit goes through (R-CODE-8). (Owner, 2026-11-25.) | Shown after the edit, a rule comes too late, and the LLM can at best repair its change. A test with Claude Code 2.1.331 on 2026-11-25 showed that the reason of a refused edit reaches the model before the file changes, while text added to an allowed edit only arrives with the edit's result. |
| D53 | Wherever an LLM is given entries (search, brief --task, which --binding and the refusal of guard), each entry's related entries (`link`) are shown under it: one level deep, titles only, at most three, only entries with status approved. (Owner, 2026-11-25.) | A link is the most precise relation in the dossier because someone made it on purpose; one level, titles and three at most keep the output short, and an old link must not feed a replaced rule. |

---

## 4. An entry in brief

```markdown
<!-- dossier-format: 1 -->
# LES-031 · Steady mixing-plane runs overpredict stall margin

- kind: lesson
- status: approved
- created: 2026-11-19T14:03:00+01:00
- author: anna
- via: anna-claude
- approved: 2026-11-19T16:20:00+01:00 by anna
- machine: axial compressor
- analysis: steady RANS, speedline
- software: ExampleSolver 2406
- confidence: simulated
- source: P-017:T-042
- related: team:HOW-007

## Statement

Near the stability limit, steady runs with a mixing-plane interface converged at mass flows
where the transient run already showed rotating stall.

## Valid for

Transonic front stages, tip Mach number above 1.1. Not checked for subsonic rear stages.

## Evidence

Comparison of steady and transient speedlines in P-017, report section 4.2.
```

The exact format is fixed in [`docs/format.md`](format.md); where the two differ, format.md counts.

---

## 5. Functional requirements

### 5.1 Places — PLACE

- **R-PLACE-1 MUST** A project dossier is a folder `DOSSIER/` in the project folder, next to
  `TODO.md`. `dossier` finds it by walking up from the current folder; `--dossier <path>` names it
  directly.
- **R-PLACE-2 MUST** `dossier init --code <project code> --team <path>` creates the project
  dossier: the folders, the settings file with the project code and the path of the team dossier,
  and an empty list of confidential terms. It refuses to run where a dossier already exists.
- **R-PLACE-3 MUST** `dossier init --team-dossier` creates a team dossier in the current folder:
  the folders, the team settings (people, LLM names, tag lists) and the software list.
- **R-PLACE-4 MUST** The project code (for example `P-017`) is unique in the team. `init`
  registers it in the team dossier and refuses a code that is already registered for another
  folder.
- **R-PLACE-5 MUST** Every command that takes `--team` works on the team dossier of the current
  project; inside a team dossier it is the default.
- **R-PLACE-6 SHOULD** `dossier init --with-todo` also runs `todo init` when the folder has no
  `TODO.md`.
- **R-PLACE-7 SHOULD** `dossier where` prints the project dossier, the team dossier, both state
  folders, the project code and, in remote mode, the server and path.
- **R-PLACE-8 MUST** A project dossier whose team dossier can't be reached still works for
  everything inside the project; commands that need the team dossier fail with an error that
  names the path.

### 5.2 Entries — ENTRY

- **R-ENTRY-1 MUST** One entry is one file `<ID>-<short-title>.md` in the folder of its kind:
  `decisions/`, `findings/`, `lessons/`, `howtos/`, `usage/`. The short title is made from the
  title: lower case, ASCII, words joined by `-`, at most 50 characters.
- **R-ENTRY-2 MUST** A file has a format line, a heading `# <ID> · <title>`, a block of
  `- key: value` lines, and then sections with `##` headings. Unknown keys and unknown sections
  are kept as written.
- **R-ENTRY-3 MUST** Fields of every entry: `kind`, `status`, `created`, `author` (the person),
  `via` (the LLM, when one wrote it), `approved` (time and person, when approved), `source` (task
  IDs, and for team entries the project code), `related`, `supersedes`, `superseded-by`,
  `review` (a date after which the entry should be checked again).
- **R-ENTRY-4 MUST** Tag fields of every entry: `machine` (for example axial compressor, radial
  turbine), `analysis` (for example steady RANS, modal analysis, throughflow), `software` (name
  and, where it matters, version), and free `tags`. Each holds a comma-separated list. The team
  settings may list the allowed values for `machine` and `analysis`; a value outside a list is
  refused with the list shown.
- **R-ENTRY-5 MUST** A **decision** has the sections *Decision*, *Reason* and *Rejected*
  (what else was considered, and why not). *Decision* and *Reason* are required.
- **R-ENTRY-6 MUST** A **lesson** has the sections *Statement*, *Valid for* and *Evidence*, and
  the field `confidence`: `measured` (checked against test data), `simulated` (seen in
  simulation only), `experience` (seen repeatedly, not checked systematically) or `guess`. All
  four are required; a lesson without *Valid for* can't be approved.
- **R-ENTRY-7 MUST** A **finding** has the sections *Observation* and *Evidence*. It states what
  was seen in this project and makes no claim beyond it.
- **R-ENTRY-8 MUST** A **how-to** has the sections *Purpose*, *Steps*, *Settings*, *Traps* and
  *Tested with* (software versions and the project codes where it worked). `software`, *Purpose*
  and *Steps* are required.
- **R-ENTRY-9 MUST** A **usage record** has the fields `software`, `version`, `task` (the
  `todo` task), `analysis`, `howto` (the how-to that was followed, if any) and the section
  *Deviations* (what was done differently, and whether it worked).
- **R-ENTRY-13 MUST** An **assumption** has the field `state` (`open`, `confirmed` or `wrong`)
  and the sections *Assumption*, *Reason* (why it had to be assumed), *Affects* (the tasks,
  files and values that rest on it) and *Confirmation* (who confirmed or refuted it, when, and
  how). *Assumption* and *Reason* are required.
- **R-ENTRY-14 MUST** Every entry has the fields `valid-from` and `valid-until` (D20), written by
  the tool: `valid-from` is the time of approval, `valid-until` the time it was superseded,
  retired or, for an assumption, found wrong.
- **R-ENTRY-15 MUST** A finding may carry `outcome: dead end`: something that was tried and did
  not work. Its *Observation* says what was tried, what happened and, if known, why.
- **R-ENTRY-16 MUST** An entry may show pictures. `--image <file> --caption "…"` (repeatable)
  on every command that writes an entry, and `image add <ID> <file> --caption "…"`, copy the
  picture into `DOSSIER/images/<ID>/` and add a standard Markdown image line with the caption to
  the entry, so that it is displayed by any Markdown viewer. Allowed: PNG, JPEG and SVG up to
  2 MB each (`image_limit`), at most 10 per entry. A caption is required and says what the
  picture shows and what to look at.
- **R-ENTRY-17 MUST** `show` lists an entry's pictures with path and caption, and `brief --task`
  prints the path and caption of the pictures of the entries it shows in full, so that an LLM
  that can read pictures knows which to open. `image remove <ID> <file>` removes one.
- **R-ENTRY-18 MUST** A picture can't be checked for confidential terms (R-CONF-2). `promote`
  and writes with `--team` therefore list each picture with its caption and need a person at a
  terminal to confirm each one before it is copied to the team dossier.
- **R-ENTRY-19 MUST** Entries are written in English (D25): titles, sections, captions, notes
  and tag values. The agent rules tell an LLM to write in English also when the session is in
  another language, and to keep the original wording in quotes only where the exact words
  matter. Names of files, software and people are left as they are. The tool's own messages are
  English.
- **R-ENTRY-20 SHOULD** `check` warns about entries that look as if they were written in German
  (several of the most common German words in the text), naming the entry. It is a warning, not
  an error.
- **R-ENTRY-12 MUST** A **script** entry has the fields `software` (the program it runs in or
  with), `language` (for example Java macro, Python), `path` (where the script file is, R-USE-9)
  and `checksum`, and the sections *Purpose*, *Inputs* (files, parameters, the state the model
  must be in), *Outputs* (what it writes or changes), *How to run*, *Traps* and *Tested with*.
  `software`, `path`, *Purpose* and *How to run* are required.
- **R-ENTRY-10 MUST** Evidence and sources may name files and report pages by path. `dossier`
  never copies those files; `check` reports paths that no longer exist (R-CHECK-1).
- **R-ENTRY-11 SHOULD** Numbers in entries are written with their unit. `check` warns about
  lessons and how-tos whose *Settings* or *Statement* contain a bare number on a `key: value`
  line.

### 5.3 Identity — ID

- **R-ID-1 MUST** IDs are `DEC-`, `ASM-`, `FND-`, `LES-`, `HOW-`, `SCR-` or `USE-` and a number of at least three
  digits; since D31, D41 and D22 also `TOOL-`, `INV-` and `DIG-`. Each dossier counts each kind separately. The next number is the highest ever given
  plus one, chosen under the lock, also counting entries that were deleted.
- **R-ID-2 MUST** Inside its own dossier an entry is named by its ID alone. Elsewhere it is named
  with its place: `team:LES-031` or `<project code>:FND-004`. Commands accept both forms.
- **R-ID-3 MUST** An ID never changes and is never reused. Changing an entry's title renames the
  file but not the ID.
- **R-ID-4 MUST** A `todo` task is named as `T-042` inside its project and as `P-017:T-042`
  elsewhere.

### 5.4 Writing entries — WRITE

Every write command needs a name (D13) and prints one line such as `DEC-012: added`.

- **R-WRITE-1 MUST** `add <kind> --title "…"` creates an entry. Each section and field has an
  option (`--reason "…"`, `--valid-for "…"`, `--machine "…"`); `--from <file>` reads a prepared
  Markdown file; `--stdin` reads one from standard input, so that text with quotes, backticks or
  `$` survives PowerShell and ssh.
- **R-WRITE-2 MUST** `decide "<decision>" --reason "…" [--rejected "…"] [--task T-042]` is the
  short form for a decision. It is approved at once when a person runs it. When an LLM runs it,
  it must pass `--said-by <person>` (the person who made the decision in the session); the entry
  is then approved, with `author` that person and `via` the LLM.
- **R-WRITE-3 MUST** `found "<observation>" [--evidence "…"] [--task T-042]` is the short form
  for a finding. Findings are facts of the project and are approved at once, whoever writes
  them; `via` shows when an LLM did.
- **R-WRITE-4 MUST** `learn "<statement>" --valid-for "…" --evidence "…" --confidence <level>`
  is the short form for a lesson. Run by a person it is approved; run by an LLM it is a draft.
- **R-WRITE-5 MUST** `howto --software <name> --title "…"` creates a how-to, usually with
  `--from` or `--stdin`. Run by a person it is approved; run by an LLM it is a draft.
- **R-WRITE-6 MUST** New entries go to the project dossier unless `--team` is given. Lessons and
  how-tos written with `--team` pass the check of R-CONF-2.
- **R-WRITE-7 MUST** `found --later "<text>"` saves a quick note with nothing else required: a
  finding with status `draft` and the tag `inbox`. It exists so that a thought costs ten seconds;
  it is sorted out later (R-DRAFT-3).
- **R-WRITE-12 MUST** `propose "<decision>" --reason "…" [--rejected "…"] [--task T-042]` writes
  a decision with status `proposed` (D21). Anyone may propose, also an LLM without `--said-by`.
  `accept <ID>` and `decline <ID> --reason "…"` are run by a person listed in `deciders` in the
  project settings (D28), at a terminal. An accepted proposal is a decision in
  force from that moment; a declined one is kept, hidden like a rejected draft.
- **R-WRITE-13 MUST** `decide` (R-WRITE-2) is accepted at once only when the person who decided
  is listed in `deciders` (or the list is empty); from anyone else it becomes a proposal, and the
  command says so.
- **R-WRITE-14 MUST** `assume "<assumption>" --reason "…" [--affects <task, file or value>…]`
  writes an assumption with state `open`. Assumptions are facts of the project and are approved
  at once, whoever writes them.
- **R-WRITE-15 MUST** `confirm <ID> --by "<who and how>"` sets an assumption to `confirmed`.
  `refute <ID> --by "<who and how>"` sets it to `wrong`, and prints everything listed under
  *Affects* together with the files made from those files (R-FILE-10) and the entries whose
  `source` is one of those tasks; each of them is marked `rests on a wrong assumption` in
  `show`, `files`, `status` and `brief` until a person clears the mark with
  `cleared <ID or path> --note "…"`. Both commands are run by a person.
- **R-WRITE-16 MUST** `found --dead-end "<what was tried and what happened>"` writes a finding
  with `outcome: dead end` (R-ENTRY-15).
- **R-WRITE-8 MUST** `set <ID> --<field> "…"` changes a field or section; `retitle <ID> "…"`
  changes the title. Changing an approved lesson or how-to by an LLM turns it back into a draft
  and keeps the approved text available (R-HIST-2).
- **R-WRITE-9 MUST** `link <ID> <other ID>` adds each entry to the other's `related` field.
- **R-WRITE-10 SHOULD** `--dry-run` on every write command prints what would be written and
  changes nothing.
- **R-WRITE-11 SHOULD** When a new lesson or how-to is added, `dossier` prints the titles of up
  to five existing entries with the most similar title and tags, so that the writer can update
  one of them instead (`--update <ID>` does that in the same command).

### 5.5 Drafts and approval — DRAFT

- **R-DRAFT-1 MUST** `dossier review-task <task>` prints the material for drafting: the task as
  `todo show` gives it (title, fields, notes), the entries already linked to the task, the
  commits linked to the task when the folder is a git repository, and the drafting rules of
  R-DRAFT-2. It changes nothing.
- **R-DRAFT-2 MUST** The drafting rules, printed for the LLM: propose a lesson only for something
  that would change how a later project is done; propose none when nothing was learned; state the
  conditions; give the evidence; say what was not checked; one lesson per draft; never include a
  confidential term in a team draft; check for an existing entry first; record what was tried
  and did not work as a dead end, because that is what gets repeated; record what had to be
  assumed.
- **R-DRAFT-3 MUST** `dossier drafts` lists the drafts: ID, kind, title, who proposed it, from
  which task, and age. `--mine` shows those proposed for the calling person.
- **R-DRAFT-4 MUST** `approve <ID>…` approves drafts. It runs only on a terminal, shows each
  draft in full, and asks for confirmation one by one (approve, edit first, reject, skip). It
  refuses when standard input is not a terminal, and it refuses a name that the team settings
  list as an LLM.
- **R-DRAFT-5 MUST** `reject <ID> [--reason "…"]` sets a draft to `rejected`. Rejected entries
  are hidden everywhere except `list --status rejected`, and are kept so that the same proposal
  can be recognised (R-WRITE-11 includes them, marked as rejected).
- **R-DRAFT-6 MUST** Drafts are never printed by `brief` (R-CTX) or found by `search` unless
  `--drafts` is given; when they are shown they are marked `DRAFT — not approved`.
- **R-DRAFT-7 SHOULD** `dossier review-session <session file>` prints the same kind of material
  for a whole session: the parts where a decision was stated or a result was discussed, cut to
  a size limit, followed by the drafting rules. The session file is in `dossier`'s own format
  (D44, format.md): a version line and the turns of the person and the LLM as plain text; a
  file with a newer version than the program knows is refused with a message that says to
  update. `review-session --claude <transcript>` first converts a transcript of Claude Code;
  `session convert --claude <transcript>` only converts and prints the session file. A
  transcript the converter cannot read gives an empty session and one line that says so,
  never an error in the middle. These commands read a file of the machine the session ran on,
  so they run there and need no dossier, also in a folder that is connected to a server.
- **R-DRAFT-8 SHOULD** Drafts older than 14 days (`draft_days`) are listed first in `summary`
  with their age.
- **R-DRAFT-9 COULD** `hooks install` adds a Claude Code hook that, when a session ends, runs a
  configured agent command (D7: the command is the team's own, `dossier` only starts it) on the
  output of `review-session`, so that drafts are waiting the next morning.

### 5.6 Life of an entry — LIFE

- **R-LIFE-1 MUST** Statuses: `draft`, `approved`, `rejected`, `superseded`, `retired`; and for
  decisions also `proposed` and `declined`, which behave like `draft` and `rejected` except for
  who may accept them (R-WRITE-12).

  | From | approve | reject | supersede | retire | reopen |
  |---|---|---|---|---|---|
  | draft | approved | rejected | — | — | — |
  | approved | — | — | superseded | retired | draft |
  | rejected | — | — | — | — | draft |
  | superseded | — | — | — | — | — |
  | retired | — | — | — | — | approved (by a person) |

- **R-LIFE-2 MUST** Every entry shows who wrote it, through which LLM, who approved it and when.
  These fields are written by the tool and can't be set by options.
- **R-LIFE-3 MUST** Entries are never deleted by a normal command. `remove <ID> --yes` exists for
  a person at a terminal (for example for customer data entered by mistake); it deletes the file
  and records the removal, without the text, in the journal.
- **R-LIFE-4 MUST** `supersede <old ID> --by <new ID>` marks the old entry as replaced and links
  both. Wherever the old entry would be shown, the new one is shown instead, with a note.
- **R-LIFE-5 MUST** `retire <ID> --reason "…"` marks an entry as no longer valid without a
  replacement. Retired entries are shown only with `--all`, with the reason.
- **R-LIFE-6 MUST** The tool records a checksum of each approved lesson and how-to at approval.
  When the file is later changed by hand, `check`, `show` and `brief` mark the entry as
  `changed after approval` until a person approves it again.
- **R-LIFE-9 MUST** `--on <date>` on `list`, `search`, `show`, `story`, `status`, `brief`,
  `values` and `files` prints what was in force on that date: entries whose `valid-from` is on or
  before it and whose `valid-until` is empty or after it, values as they were, and file lists as
  they were.
- **R-LIFE-7 SHOULD** An entry with a `review` date in the past is marked `due for review` in
  `show`, `summary` and `check`. `reviewed <ID> [--next <date>]`, run by a person, confirms it
  still holds.
- **R-LIFE-8 SHOULD** A how-to whose newest *Tested with* version is older than the version in
  the newest usage record of the same software is marked `not tested with <version>`.

### 5.7 Team dossier and promotion — TEAM

- **R-TEAM-1 MUST** The team dossier has the same folders and rules as a project dossier, plus
  the team settings, the software list (R-USE-1), the list of project codes (R-PLACE-4) and the
  usage index (R-USE-3). It has no decisions of single projects and no findings.
- **R-TEAM-2 MUST** `promote <ID> [--as lesson|howto]` copies a project entry into the team
  dossier as a new draft with its own team ID, `source` set to the project code and entry, and
  both entries linked. A finding is promoted as a lesson and then needs *Valid for* and
  `confidence` before it can be approved.
- **R-TEAM-3 MUST** `promote` and every write with `--team` run the check of R-CONF-2 and refuse
  when it fails, naming the terms found and the lines.
- **R-TEAM-4 MUST** `dossier close` (run by a person listed in `deciders` at the end of a project) lists
  every approved finding, lesson and how-to in the project that has not been promoted, and asks
  for each: promote, keep in the project only, or decide later. It records the answers, so a
  second run shows only what is left.
- **R-TEAM-5 SHOULD** `dossier team summary` shows the team dossier: entries per kind and status,
  drafts waiting, entries due for review, and the newest ten entries.
- **R-TEAM-7 MUST** Only members write to the team dossier. `team member add <person> [--llm
  <name>]…` and `team member remove <person>` (a person at a terminal) keep the list of members
  in the team settings, each change with who and when (D29). (Owner, 2026-11-21.)
- **R-TEAM-6 SHOULD** When the team dossier is a git repository, every command that changes it
  also commits the change with a subject naming the command and the entry. `dossier` never
  pushes.

### 5.8 Software and usage — USE

- **R-USE-1 MUST** The team dossier holds the software list: for each software its name, other
  spellings that mean the same, whether it is commercial or the team's own, and where it is
  installed or kept. `software add`, `software list`, `software show <name>`. A `software` field
  with a name that isn't on the list is refused with the closest names shown.
- **R-USE-2 MUST** `used <software> --version <v> --task <task> [--howto <ID>]
  [--deviations "…"]` writes a usage record in the project dossier.
- **R-USE-3 MUST** Every usage record also adds one line to the team's usage index: date,
  project code, software, version, analysis, machine, the how-to followed, and the person. No
  title, no task text, no file names.
- **R-USE-4 MUST** `where-used <software> [--analysis "…"] [--machine "…"]` reads the usage index
  and prints the matching lines, newest first, with the how-tos they followed.
- **R-USE-5 MUST** `list --kind howto --software <name>` lists the approved how-tos for a software with their
  purpose and the versions they were tested with.
- **R-USE-8 MUST** `script add <file> --software <name> --title "…"` creates a script entry
  (R-ENTRY-12) for a script file, usually with `--from` or `--stdin` for the sections. Run by a
  person it is approved; run by an LLM it is a draft. A script written for one customer stays in
  the project dossier; a reusable one goes to the team dossier with `--team` or `promote`, after
  the check of R-CONF-2, which also reads the script file itself.
- **R-USE-9 MUST** The team settings name the team's script folder (`script_root`, a folder or
  the working folder of a git repository on the server). A team script's `path` is relative to
  it; a project script's `path` is relative to the project folder. `dossier` never copies or
  changes a script file.
  *Phase 3 (D37):* there is no `script_root`; a tool keeps its `location` as given (relative
  to the project folder or absolute), and `dossier` still never copies or changes a script file.
- **R-USE-10 MUST** `list --kind script [--software <name>] [--language "…"] [words…]` lists approved scripts with
  their purpose, language and tested versions; `show <ID>` also prints the full path and whether
  the file still matches its recorded checksum.
- **R-USE-11 MUST** A script file that changed since its entry was approved is marked
  `file changed since approval` in `show`, `scripts`, `brief` and `check`. `script update <ID>`
  records the new checksum; run by an LLM it turns the entry into a draft.
- **R-USE-12 MUST** `used` accepts `--script <ID>…`, and the usage index (R-USE-3) records the
  scripts that were run. `where-used --script <ID>` lists the projects that ran a script.
- **R-USE-14 MUST** (Part of `used`, R-USE-2; there is no separate `ran` command.)
  `used <software> --version <v> --task <task> --model <file>
  [--script <ID>…] [--param <name>=<value>…] [--output <path>…] [--howto <ID>] [--note "…"]`
  records one simulation run in a single step: it writes a usage record with the model's
  checksum, the scripts, the parameters and the outputs, and attaches each output to the task
  with role `result`, made from the model (R-FILE-10). A parameter may name a key value
  (R-VALUE-1) instead of giving a number. (Idea taken from run trackers such as Sumatra and
  AiiDA.)
- **R-USE-15 MUST** `runs [--task <task>] [--model <file>] [--software <name>]` lists the
  recorded runs, newest first: date, person, model, parameters that differ from the previous run
  of the same model, and outputs. `runs --compare <ID> <ID>` prints what differs between two
  runs.
- **R-USE-13 SHOULD** A how-to may name the scripts it uses in a `scripts` field; `howtos` and
  `brief --task` then show them together.
- **R-USE-6 SHOULD** A usage record with *Deviations* that names a how-to adds a line to that
  how-to's *Open remarks* section in the team dossier (project code, date, the deviation), after
  the check of R-CONF-2, so that the how-to's owner sees what to update.
- **R-USE-7 SHOULD** When a task is finished with `todo done` and has no usage record, the agent
  rules (R-AGENT-1) tell the agent to add one if software was used; `dossier summary` lists
  finished tasks without a usage record that carry an `analysis` field.

### 5.9 Reading and searching — READ

- **R-READ-1 MUST** `show <ID>` prints the entry as it is in the file, with marks for draft,
  superseded, retired, changed after approval and due for review. With `--json` it returns
  every field, every section, unknown keys, and the links in both directions.
- **R-READ-2 MUST** `list` prints one line per entry: ID, kind, status mark, tags and title.
  Filters: `--kind`, `--status`, `--machine`, `--analysis`, `--software`, `--tag`, `--author`,
  `--task`, `--since <date>`, `--where key=value`. Without `--status` it shows approved entries.
- **R-READ-3 MUST** `search <words…>` finds entries in the project dossier and the team dossier
  together, each result marked with its place. It lists first the entries that hold every word:
  matches in the title first, then in tags, then in text, and within each newest first. A word is
  held when the entry holds it, a glossary spelling of it (R-TERM-4) or a word with the same stem
  (R-FIND-1); case is ignored. When more than one word is given, it then lists up to 10 entries that
  hold only some of the words, best first (R-FIND-2), under the line `some of the words:`. The same
  filters as `list`.
- **R-READ-4 MUST** Search results print the ID, the title and the line that matched; `--full`
  prints whole entries. With `--json` each result has `match`: `all` or `some`.
- **R-READ-5 MUST** Every read command supports `--json` and `--team` / `--project` to limit the
  place.
- **R-READ-6 MUST** `dossier history <task or ID>` prints, in time order, everything recorded
  about a task or entry: entries created from it, changes, approvals, supersessions.
- **R-READ-7 SHOULD** `dossier story` prints the project's decisions in the order they were made,
  each with its reason and the task it belongs to: the answer to "why was it done this way".
  `--milestone <task>` limits it to the tasks under one `todo` milestone.
- **R-READ-8 SHOULD** *(Merged into `status`, R-CTX-9: `status` also shows what follows.)*
  Formerly `summary`, at most 40 lines: entries per kind, drafts waiting (oldest
  first), entries due for review or changed after approval, the five newest decisions, and files
  taken and not returned (R-FILE-5).
- **R-READ-10 MUST** `status --since last` (there is no separate `news` command) prints what is new for the calling person since they last ran it: one
  line per entry that was approved, superseded or retired in the meantime, in the project dossier
  and the team dossier, grouped by place, with ID, kind, title and who approved it. At most 20
  lines (`news_lines`); a last line counts the rest. Drafts and changes to fields are not listed.
  Running `news` moves the person's marker to now; `--keep` leaves it, `--since <date>` ignores
  it. A person's LLMs share the person's marker and never move it.
- **R-READ-11 MUST** `brief` (R-CTX-1) ends with one line when there is news, for example
  `4 new entries since 2026-11-12: dossier status --since last`, and nothing when there is none.
- **R-READ-9 COULD** `search --similar <ID>` lists entries that share the most tags and title
  words with an entry.

### 5.9a Finding related entries — FIND

Several commands choose entries that are related to a text: `decide` (R-CTX-13), `learn`, `problem`,
`brief --task` (R-CTX-2) and `search --similar` (R-READ-9). They all choose the same way (D48).

- **R-FIND-1 MUST** Words are compared by their English word stem (D49), so `reuse`, `reusing` and
  `reused` count as one word, and common English words such as `and`, `the` and `with` never count.
- **R-FIND-2 MUST** A word or tag that few entries hold counts more: its weight is
  1 + floor(log2(entries / entries that hold it)). A match in the title counts 5 times the weight,
  a match in a tag 3 times, a match in the text once.
- **R-FIND-3 MUST** An entry is listed only when it shares a tag with the query or its score reaches
  a third of the sum of the weights of the query's words, so a list may be empty.
- **R-FIND-4 MUST** When the query holds a spelling of a glossary term (R-TERM-4), the words of the
  term's other spellings join the query.
- **R-FIND-5 MUST** Under each entry that search lists, each entry line of brief --task and each
  entry of which --binding (and so of guard's refusal), its related entries with status approved
  are shown, in the order of its related field, at most three, as one line each: two spaces,
  linked:, the ID, · and the title. A linked entry is not followed further (D53).

### 5.10 Context for a session or task — CTX

- **R-CTX-1 MUST** `brief` prints what an agent should know at the start of a session in this
  project, at most 60 lines (`brief_lines`): the project code and its tags; the project's
  approved decisions in full, newest first; the titles and IDs of the project's findings; the
  titles and IDs of the team lessons and how-tos that match the project's tags; and the number of
  drafts waiting. When the limit is reached, older decisions are shortened to their titles and a
  last line says how many entries were left out and how to see them.
- **R-CTX-2 MUST** `brief --task <task>` prints what matters for one task: the entries linked to
  the task; the decisions of the task's milestones; the team lessons and how-tos that match the
  task's `machine`, `analysis` and `software` fields and the words of its title, best matches
  first, lessons in full and how-tos by title; and the last three usage records of the same
  software and analysis from the usage index. The words of the title are matched as
  R-FIND-1 to R-FIND-4 say.
- **R-CTX-3 MUST** The project settings hold the project's default tags (`machine`, `analysis`,
  `software`). A task may narrow them with fields of the same names in `TODO.md` (`todo` keeps
  unknown fields as written).
- **R-CTX-4 MUST** `brief` never prints drafts, rejected, superseded or retired entries, and
  marks entries that are changed after approval or due for review.
- **R-CTX-5 MUST** `brief` works without `todo` being installed; `brief --task` needs it and says
  so otherwise.
- **R-CTX-9 MUST** `status` prints the state of the whole project for a person or an LLM who
  asks "where are we", at most 60 lines (`status_lines`), in this order:
  1. the project code, its tags, and the goal of each open `todo` milestone with its progress
     (tasks done of total);
  2. who is working on what now: claimed tasks with owner (person or LLM) and age, and the files
     that are taken;
  3. what is waiting: blocked tasks with their reason, and open questions;
  4. the decisions in force, newest first, as titles with IDs;
  5. the newest five findings, as titles with IDs;
  6. what is unsettled: drafts waiting for approval, entries due for review, results marked
     `outdated` (R-FILE-11);
  7. the next five ready tasks.
  The task parts come from `todo --json`; without `todo` they are left out with one line saying
  so. `status --json` returns the same as data.
- **R-CTX-10 MUST** `status --since <date>` (or `--since last`, the caller's `news` marker)
  prints only what changed: tasks finished, decisions made, findings recorded, files returned,
  each as one line with who did it.
- **R-CTX-11 MUST** `status --milestone <task>` limits everything to one `todo` milestone.
- **R-CTX-12 MUST** The agent rules (R-AGENT-1) say: decisions in force bind every LLM in the
  project. An LLM whose task or instruction contradicts a decision stops and asks its person; it
  never works around the decision, and only a person can replace one (`supersede`).
- **R-CTX-13 MUST** `decide` prints the decisions in force that share tags or title words with
  the new one before it writes, and with `--replaces <ID>` supersedes the old one in the same
  step. An LLM must pass `--replaces` or `--besides` (stating that the new decision stands next
  to the listed ones); without either the command refuses and shows the list. Shared title words
  are found as R-FIND-1 to R-FIND-4 say.
- **R-CTX-14 MUST** `digest <milestone>` prints the material for summing up a closed `todo`
  milestone: its goal, its tasks, and its decisions, assumptions, findings and dead ends, with
  rules for the summary (at most 15 lines: what was decided and still binds, what was found,
  what was assumed and is still open, what did not work). The LLM in the session writes it with
  `digest <milestone> --write --stdin`; it is a draft until a person approves it. A digest is
  an entry of its own kind (`DIG-001`, in `digests/`, with the field `milestone`), so it is
  approved, shown and superseded like a lesson.
- **R-CTX-15 MUST** Once a milestone has an approved digest, `brief` and `status` print the
  digest in place of that milestone's single entries, with one line saying how to see them.
  Decisions marked `standing: yes` (they bind the whole project, not one milestone) and open
  assumptions are always printed singly.
- **R-CTX-16 MUST** `brief` and `status` also print: the key values (R-VALUE-4), the open
  assumptions, the proposals waiting for the project lead, and anything marked
  `rests on a wrong assumption`. `brief --task` adds the dead ends that match the task's tags and
  title words.
- **R-CTX-6 SHOULD** `brief --budget <lines>` changes the limit for one call, and `--json`
  returns the same selection as data, each item with the reason it was chosen (linked, milestone,
  tag match, word match).
- **R-CTX-7 SHOULD** `hooks install` adds a Claude Code session-start hook that runs `brief`.
- **R-CTX-8 COULD** `todo brief` (the work order of `todo` 0.8) can include the output of
  `dossier brief --task` when a setting in `todo.toml` names the command. This needs a change in
  `todo` and is decided there. For Claude Code, R-AGENT-5 gives the same result without a
  change in `todo`. *Not planned in `dossier`: it stays an idea for `todo`.*

### 5.11 Files of a task — FILE

The first part (R-FILE-8 to R-FILE-16) is the list of files that belong to a task. The second
part (R-FILE-1 to R-FILE-7) protects large files that are copied to a PC and back.

- **R-FILE-8 MUST** `attach <task> <path>… --role <role> [--note "…"]` adds files or folders to
  a task's file list. Each line holds the path (relative to the project folder), the role, an
  optional note, who added it and when, and for a file its size, modification time and checksum
  (R-FILE-6). A folder is recorded as a folder, with the number of files in it and no checksum.
- **R-FILE-9 MUST** Roles: `description` (the task's specification or the customer's request),
  `geometry` (for example `.prt`), `mesh`, `model` (the simulation file, for example `.sim`),
  `setup` (boundary conditions, parameter files), `script`, `result` (data that came out, for
  example `.csv`, a results folder), `report`, `reference` (papers, standards, test data) and
  `other`. The team settings may add roles.
- **R-FILE-10 MUST** `attach` accepts `--from <path>…`: the files this file was made from (a
  result from a model, a model from a geometry). `dossier` records the checksum those files had
  at that moment, so it can later tell whether a result still belongs to the model next to it.
- **R-FILE-11 MUST** `files --task <task>` prints the task's list grouped by role: path, note,
  date, and a mark when the file is missing, changed since it was attached, taken by someone
  (R-FILE-1), or made from a file that has changed since (`outdated`). `files --role <role>`
  and `files <words…>` search all lists of the project.
- **R-FILE-12 MUST** The list of each task is one readable Markdown file,
  `DOSSIER/files/<task>.md`, changed only through the tool.
- **R-FILE-13 MUST** `detach <task> <path>` removes a line; `reattach <task> <path>` records the
  file's current size, time and checksum after an intended change; `attach --moved <old> <new>`
  changes a path and keeps its history.
- **R-FILE-14 MUST** `brief --task` (R-CTX-2) includes the task's file list, at most 15 lines,
  descriptions and models first.
- **R-FILE-15 MUST** `which <path>` answers the opposite question: which tasks a file belongs
  to, in which role, what it was made from and what was made from it.
- **R-FILE-16 SHOULD** Entries may point to attached files in their *Evidence* section by path;
  `show` marks such a path when the file is missing or changed.

- **R-FILE-1 MUST** `take <file>… --task <task>` records for each file who took it, for which
  task, when, and the file's size, modification time and checksum. It refuses a file that someone
  else has taken, naming the person, the task and the time; `--force` takes it anyway and records
  that.
- **R-FILE-2 MUST** `return <file>… --check` tells whether each file on the server is still the
  one that was taken (same size, time and checksum). Exit code `0` when all are unchanged, `1`
  with the changed files named otherwise.
- **R-FILE-3 MUST** `return <file>… --done` records the new size, time and checksum and ends the
  take. `return <file> --abandon` ends a take without a new version.
- **R-FILE-4 MUST** `dossier` on the server never copies a file. In remote mode the client does
  the copy (R-REMOTE-4).
- **R-FILE-5 MUST** `files` lists the files that are taken: file, person, task, since when.
  `files --log <file>` lists every recorded version of a file with the task it came from.
- **R-FILE-6 SHOULD** Checksums of files over 1 GB (`checksum_limit`) are taken from the first
  and last 16 MB plus the size; the record says which method was used.
- **R-FILE-7 SHOULD** A take older than 14 days is marked in `files` and `summary`.

### 5.11a Key values — VALUE

- **R-VALUE-1 MUST** `value set <name> "<number> <unit>" [--source "…"] [--note "…"]` sets a
  named value of the project, for example `design.massflow` = `12.4 kg/s`. Names are lower case
  with dots. A value without a unit is refused unless `--dimensionless` is given. A value may
  also be a range or a short text (`"0.92 … 0.95"`, `"air, ideal gas"`).
- **R-VALUE-2 MUST** Values are approved at once when a person sets them or when an LLM passes
  `--said-by <person>` or a `--source` that is an attached file (R-FILE-8). Otherwise the change
  waits as a draft, and the old value stays in force.
- **R-VALUE-3 MUST** Changing a value keeps the old one with its dates (D20). The command prints
  the runs that used the old value (R-USE-14) and the assumptions and entries that name it, so
  that the person sees what may need redoing.
- **R-VALUE-4 MUST** `values [<prefix>]` prints the values in force: name, value, unit, source,
  date. `value show <name>` prints one with its history. All values live in one readable file,
  `DOSSIER/values.md`. `brief` and `status` print at most 15 of them, those marked `--key`
  first.
- **R-VALUE-5 MUST** Values belong to the project and never go to the team dossier.
- **R-VALUE-6 SHOULD** Entries, parameters of `ran` and task fields may refer to a value as
  `{design.massflow}`; `show` and `brief` print the name with the value in force.

### 5.11b Exchange with the customer — EXCH

- **R-EXCH-1 MUST** `received <path>… --from "<who>" [--date <date>] [--note "…"]` records what
  came from the customer: the files with size and checksum, from whom, the date, and a note.
  The files are attached to the project with role `reference` or `description`.
- **R-EXCH-2 MUST** `delivered <path>… --to "<who>" [--date <date>] [--based-on <task or
  file>…] [--note "…"]` records what was sent to the customer, with the checksum of each file
  and of everything named under `--based-on`, so that the exact results behind a report version
  can be named later.
- **R-EXCH-3 MUST** `exchange` lists both in time order; `exchange --received` or `--delivered`
  limits it. The records live in one readable file, `DOSSIER/exchange.md`, and never go to the
  team dossier.
- **R-EXCH-4 MUST** `delivered` warns, and asks a person to confirm, when something named under
  `--based-on` is marked `outdated` or `rests on a wrong assumption`, or when an assumption that
  affects it is still `open`.
- **R-EXCH-5 SHOULD** `status` shows the last thing received and the last thing delivered.

### 5.11c Glossary — TERM

- **R-TERM-1 MUST** The team dossier holds a glossary: `term add "<term>" "<meaning>" [--symbol
  "…"] [--unit "…"] [--also "<other spellings>"]`, `term <word>` prints matching terms,
  `glossary` prints all. A project may add its own terms with the same commands; they stay in
  the project.
- **R-TERM-2 MUST** `brief --task` prints the glossary terms that appear in the task's title,
  fields or notes, at most 10 lines.
- **R-TERM-4 MUST** A term's other spellings (`--also`) include its German word where the team
  uses one. `search` also looks for a term when one of its other spellings is searched for, so
  that a German word finds the English entries.
- **R-TERM-3 SHOULD** `check` lists abbreviations of two to six capital letters that appear in
  at least three entries and are in no glossary.

### 5.11d Backup — BACKUP

- **R-BACKUP-1 MUST** `pack --to <file> [--team]` writes the whole dossier, the state folder
  included, into one archive, under the lock. It never includes the files that entries and file
  lists point to.
- **R-BACKUP-2 MUST** `unpack <file> --into <folder>` restores an archive into an empty folder
  and runs `check`.
- **R-BACKUP-3 MUST** The README says plainly that the dossier is one copy on the server, that
  the team must include the project folders and the team dossier in the server's backup, and
  how to schedule `pack` as an extra.

### 5.11f Bringing in finished projects — IMPORT

- **R-IMPORT-1 MUST** `init --past --code <project code>` creates a dossier for a finished
  project in its folder, marked as past. If the folder can't be written to, `--in <folder>`
  puts the dossier elsewhere and records the project folder it describes.
- **R-IMPORT-2 MUST** `review-folder [<path>]` prints the material for drafting from an old
  project folder: the folder tree to a given depth with file types, sizes and dates; the
  documents it found (reports, notes, presentations, spreadsheets, scripts) newest first; what
  has already been reviewed; and the import rules of R-IMPORT-4. It reads no file contents; the
  LLM opens the documents itself.
- **R-IMPORT-3 MUST** `review-document <file>` records that a document is being reviewed and
  prints the import rules. Entries drafted afterwards with `--from-document <file>` carry the
  document, and the page or section, as their source. `reviewed-document <file> [--nothing]`
  marks it as done, with `--nothing` when it held nothing worth keeping.
- **R-IMPORT-4 MUST** The import rules, printed for the LLM: draft only what a later project
  would use; prefer the report's conclusions and the reasons behind choices over descriptions
  of what was done; record dead ends; give the page; set `confidence` to what the document
  supports, and `guess` when it only implies something; write in English also when the document
  isn't; never copy customer names or product names into the text of a lesson.
- **R-IMPORT-5 MUST** Every imported entry is a draft, whoever wrote it, and carries
  `imported: yes` for good. `show`, `search` and `brief` mark approved imported entries as
  `imported`, so a reader knows it was reconstructed afterwards.
- **R-IMPORT-6 MUST** `drafts --imported` lists imported drafts apart from the others, grouped
  by document. `approve` handles them one by one as usual (R-DRAFT-4).
- **R-IMPORT-7 MUST** `import status` shows, per past project: documents found, reviewed,
  marked as holding nothing, drafts waiting, entries approved, entries promoted.
- **R-IMPORT-8 MUST** Promotion from a past project follows R-TEAM-2 and R-CONF-2 unchanged: the
  confidential terms of the old project must be entered first (R-CONF-5).
- **R-IMPORT-9 SHOULD** `review-folder --software` lists the script files and macro files found
  (by file type), as candidates for `script add`.

### 5.11e Code projects — CODE

A dossier can also sit in a software project. It holds why the code is the way it is; where
things are in the code is left to code navigation tools.

- **R-CODE-1 SHOULD** `map set <path> "<what this part is responsible for>" [--start "<where to
  begin reading>"]` describes one package or folder of the code; `map` prints all descriptions
  as a tree. The map lives in one readable file, `DOSSIER/map.md`, and is meant for a few dozen
  lines, not one per file. `brief` prints it first, at most 25 lines.
- **R-CODE-2 SHOULD** An entry may point into code with a `code` field: a path, optionally with
  a class or function name (`solver/io.py::MeshReader.read`). `show` prints the pointers;
  `which <path>` (R-FILE-15) also lists the entries that point to a file.
- **R-CODE-3 SHOULD** `check` reports `code` pointers whose file no longer exists or no longer
  contains the name as text, and `map` paths that no longer exist. It doesn't parse the
  language.
- **R-CODE-4 SHOULD** `brief --task` prints the entries whose `code` pointers lie in the files
  the task names in its `files` field (`todo` 0.8 file scopes).
- **R-CODE-5 SHOULD** `init --code-project` creates a dossier without confidential terms, a team
  dossier or the customer exchange, and writes agent rules without the parts about runs, large
  files and customers.
- **R-CODE-6 MUST** A code pointer to a folder covers every file in it, in `which` and in
  `brief --task`.
- **R-CODE-7 MUST** `decide`, `found` and `learn` take `--code <pointers>`.
- **R-CODE-8 MUST** `which <path> --binding` prints only the approved entries in force whose
  pointers cover the path; with `--session <id>` it leaves out the entries that session was shown
  before. The PreToolUse hook that `hooks install` adds (`guard`) refuses an LLM's first edit of a
  file in a session when such entries exist and names them in the refusal; the next edit of the
  file goes through (D51).

### 5.11h Investigations — INV

An investigation records trial and error (D41): a problem, the attempts to solve it in order,
and what solved it.

- **R-INV-1 MUST** An **investigation** is an entry of its own kind with IDs `INV-` (as
  R-ID-1), kept in `investigations/` in the project dossier. It has the field `state` (`open`,
  `solved` or `given up`) and the sections *Problem* (what goes wrong and how it shows;
  required), *Attempts* and *Solution*. Like a finding it is a fact of the project: it is
  approved at once, whoever writes it, and `via` shows when an LLM did. Investigations stay in
  the project and never go to the team dossier as they are (R-INV-8).
- **R-INV-2 MUST** `problem "<what goes wrong>" [--observed "…"] [--task <task>]` opens an
  investigation with state `open` and prints its ID. Before it writes, it prints the
  investigations, open or closed, with the most similar title and tags (as R-WRITE-11), so
  that an existing one is continued or its solution is used.
- **R-INV-3 MUST** `tried <ID> "<hypothesis, or what was tried>" --failed | --worked | --unclear
  [--did "<what was changed>"] [--result "<what happened>"] [--run <usage record>]
  [--task <task>]` adds one attempt at the end of *Attempts*. The text and the verdict are
  required and nothing else, so that an attempt costs ten seconds. Each attempt has a number,
  the person, the LLM if one wrote it, and the time (R-HIST-4). Attempts keep the order in
  which they were recorded and are never renumbered. `--worked` doesn't close the
  investigation: an attempt may solve one part of the problem. `--run` names the usage record
  of the run that tested it (R-USE-14), so the parameters are not typed twice.
- **R-INV-4 MUST** `solved <ID> "<what solved it>" [--attempt <number>…]` sets the state to
  `solved` and writes *Solution*, naming the attempts that led to it when several were needed
  together or in a certain order. `gave-up <ID> --reason "…"` sets `given up`. `tried` on a
  closed investigation opens it again and says so.
- **R-INV-5 MUST** Reading:
  - `show <ID>` prints the problem, every attempt in order with its verdict, and the solution;
    `list --kind investigation [--state <state>]` lists them.
  - `brief` and `status` list the open investigations under what is waiting: ID, title, number
    of attempts and the last attempt.
  - `brief --task` prints the open investigations of the task with one line per attempt, and
    the closed ones that match the task's tags and title words as one line each (the problem,
    what solved it, how many attempts failed), next to the dead ends of R-CTX-16; at most 15
    lines.
  - `search` also finds the text of attempts and names the attempt in the result.
- **R-INV-6 MUST** Tasks (D42). `--task` on `problem` names the task in which the problem came
  up; `--task` on `tried` names a task that carried out that attempt. `dossier` reads these
  tasks through `todo --json` and never creates or changes one. `history <task>` (R-READ-6)
  lists the investigations and attempts that name the task.
- **R-INV-7 MUST** A failed attempt counts as a dead end wherever dead ends are shown or
  searched (R-CTX-16, S18). `found --dead-end` (R-WRITE-16) stays for a single thing that did
  not work and belongs to no problem.
- **R-INV-8 MUST** `review-task` (R-DRAFT-1) prints the investigations of the task as material,
  and the drafting rules (R-DRAFT-2) add: a solved investigation with failed attempts is a
  candidate for a how-to or a lesson. `promote <ID> --as howto|lesson` writes that draft in the
  team dossier as in R-TEAM-2, after the check of R-CONF-2.
- **R-INV-9 SHOULD** The hook of R-AGENT-5 also covers investigations: after `todo stuck <id>`
  it suggests `problem … --task <id>`, and after `todo done <id>` for a task that an open
  investigation or one of its attempts names, it reminds the agent to record the result with
  `tried`.
- **R-INV-10 MUST** The agent rules (R-AGENT-1) say: when something fails and you start looking
  for the cause, open an investigation with `problem`, or continue the one that exists; read
  its attempts before you try anything; record one `tried` line for each hypothesis, also the
  failed ones, and not one for each compile or run; close it with `solved`.

### 5.11i Topics — TOPIC

Tags say which machine, analysis and software an entry is about; a topic says which part of the work
it belongs to, from one short list per project (D50).

- **R-TOPIC-1 MUST** The project settings hold a list `topics`. `topics` prints each topic with the
  number of approved entries that name it, and `(no topic)` with the number of approved decisions,
  findings and lessons that name none. `topics add <name>` adds a topic; an LLM is refused (D50).
- **R-TOPIC-2 MUST** An entry may carry a field `topic` (one or more, comma-separated), given with
  `--topic` on `decide`, `propose`, `found`, `learn`, `assume`, `problem` and `howto`, or with
  `--field` on `add` and `set`. When the list is not empty, a value that is not on it is refused,
  naming the list.
- **R-TOPIC-3 MUST** When the list is not empty, a decision, assumption, finding, lesson, how-to or
  investigation written by an LLM without a topic is refused; entries written from a document
  (R-IMPORT-3) are exempt.
- **R-TOPIC-4 MUST** `list` and `search` take `--topic <name>`; a topic counts as a tag for
  R-FIND-2.
- **R-TOPIC-5 MUST** `brief` prints one line with the topics and the number of decisions in force of
  each, and each decision line names its topics.

### 5.12 Remote mode — REMOTE

- **R-REMOTE-1 MUST** `dossier connect <user@server>:<project folder>` in a folder on a PC writes a
  pointer file there. From then on every `dossier` command in that folder or below is run on the
  server in the project folder, through ssh, and its output and exit code are passed through
  unchanged.
- **R-REMOTE-2 MUST** Arguments are passed so that quotes, backticks, `$`, non-ASCII characters
  and line breaks arrive unchanged; text given with `--stdin` is sent as standard input.
- **R-REMOTE-3 MUST** Commands that need a terminal (`approve`, `remove`, `close`) ask ssh for
  one. When the PC side has no terminal, they refuse as they would on the server.
- **R-REMOTE-4 MUST** In remote mode `take` first records the take on the server and then copies
  the files into the PC folder; `return` runs `--check`, copies the files back only when it
  passed, and then runs `--done`. A failed copy leaves the take as it was and says what to do.
- **R-REMOTE-5 MUST** `dossier todo <arguments…>` runs `todo` on the server in the project folder
  in the same way, so that the task list is reached from the PC without a second wrapper.
- **R-REMOTE-6 MUST** The name is taken from the PC's `DOSSIER_AGENT` (or `--agent`) and sent
  with each command; the server's own setting is not used.
- **R-REMOTE-9 MUST** `connect` also records the project in a per-user register on the PC
  (`%APPDATA%\dossier\projects.toml`; on Linux `$XDG_CONFIG_HOME/dossier/projects.toml`, normally
  `~/.config/dossier/`): project code, server, user, project folder on the server, and the local
  folder if there is one. `connect --code <code> <user@server>:<folder>` records a project
  without a local folder.
- **R-REMOTE-10 MUST** `projects` lists the register: code, server, folder, local folder, and
  whether the server answers. `--project <code>` on any command runs it for that project from
  any folder on the PC. `projects remove <code>` forgets one. So a person or an LLM never has to
  know or type a server name or path.
- **R-REMOTE-11 MUST** ssh is run so that it never asks a question: key sign-in only, no password
  prompt, a connection time limit of 10 seconds (`ssh_timeout_seconds`). When sign-in fails or
  the server doesn't answer, the command ends with exit code `5` and a message that says to sign
  in once by hand with `ssh <user@server>`. A session without a terminal must never hang.
- **R-REMOTE-12 MUST** `brief` and `status` in remote mode print one first line naming the
  project code and the server folder, so an LLM knows where the project's files are.
- **R-REMOTE-7 SHOULD** One ssh connection is kept open and reused for several commands, so that
  a command takes under half a second on the office network after the first.
  *Not built (D43): each command makes its own connection.*
- **R-REMOTE-8 SHOULD** `dossier connect --check` tests the connection, that `dossier` and `todo`
  exist on the server with matching versions, and that the folder has a dossier.

### 5.13 Confidentiality — CONF

- **R-CONF-1 MUST** The project settings hold the project's confidential terms: the customer's
  name, product and machine names, people's names, anything that identifies the customer.
  `confidential add "<term>"`, `confidential list`, `confidential remove "<term>"`. Anyone in
  the project may add a term, also an LLM when its person tells it to; `remove` needs a person at
  a terminal (D32). Matching ignores case and treats spaces, hyphens and underscores as the same.
- **R-CONF-2 MUST** Before anything is written to the team dossier from a project, the whole
  text (title, fields, sections, file paths) is checked against the project's terms. When a term
  is found the write is refused with the terms and lines.
- **R-CONF-3 MUST** Team entries name projects only by project code. The list of project codes
  in the team dossier holds the code and the server folder, never the customer's name.
- **R-CONF-4 MUST** Nothing from a project dossier is ever printed by a command run in another
  project. Search and `brief` in project B see project B and the team dossier only.
- **R-CONF-5 MUST** A project with an empty list of confidential terms can't promote or write to
  the team dossier; the error says to add the terms first, or to state with
  `confidential none --yes` (a person, at a terminal) that there are none.
- **R-CONF-6 SHOULD** `confidential scan` checks the whole team dossier against the current
  project's terms and lists every match, for terms that were added late.
- **R-CONF-7 MUST** Every command that writes text into a dossier (an entry, a value, a note on
  a file or an exchange record, a glossary term, a digest) refuses when the text looks like it
  holds a credential (D40): a private-key block (`-----BEGIN … PRIVATE KEY-----`), an access
  token of a well-known shape, or a name such as `password`, `passwd`, `secret`, `token` or
  `api key` followed by `=` or `:` and a value of one word (a value of several words, such
  as `password: ask the administrator`, is not refused). The error names the line and the kind of
  credential and never prints the value; nothing is written and nothing reaches the journal.
  The rule holds in the project dossier and the team dossier, for people and LLMs.
  `--allow-credential-like`, for a person at a terminal, writes the text anyway when the match
  is a false alarm (for example `password: required`), and the journal records
  that it was used. `check` reports the same matches in files that were changed by hand. The
  agent rules (R-AGENT-1) say: never write a password, key or token into an entry; write where
  it is kept. The check catches known shapes only; it is a safety net, not a guarantee.

### 5.14 Locking and writing — LOCK, IO, EDIT

- **R-LOCK-1 MUST** Every command that changes a dossier takes that dossier's write lock (an
  operating-system file lock in its state folder), reads, changes and writes, then releases. The
  operating system frees the lock when the process dies. A command that writes to both dossiers
  takes the project lock first, then the team lock.
- **R-LOCK-2 MUST** A command waits up to 30 seconds (`lock_wait_seconds`) for a lock, then exits
  with code `3` and says who holds it.
- **R-LOCK-3 MUST** When the dossier lies on a network file system (Windows: a network drive or
  UNC path; Linux: a mount of type cifs, smb or nfs), write commands refuse with an error that
  explains remote mode. `--allow-network-drive` overrides this for one command.
- **R-IO-1 MUST** A file is replaced in one step: written to a temporary file in the same
  folder, flushed, then renamed. A crash at any moment leaves either the old or the new file.
- **R-IO-2 MUST** Before writing, the tool checks that the file is still what it read; if it
  changed, the command exits with code `3` and nothing is written.
- **R-EDIT-1 MUST** The tool changes only the lines it has to and keeps every other byte of an
  entry file, line endings included.
- **R-EDIT-2 MUST** `edit <ID>` opens the entry in `VISUAL`, `EDITOR` or the system's default
  editor under the lock, checks the result (R-CHECK-1) when the editor closes, and records the
  change. An LLM's edit of an approved lesson or how-to follows R-WRITE-8.
- **R-EDIT-3 MUST** A file changed by hand without the tool is still read. If it can't be
  parsed, commands that only read skip it with a warning naming the file and line; commands that
  would change that file refuse.

### 5.14a Windows and Linux together — XOS

The dossier lives on Linux and is read, and through remote mode used, from Windows. Nothing may
depend on which of the two a command was typed on.

- **R-XOS-1 MUST** Only one program ever writes to a dossier: the one on the machine where the
  dossier lives. The Windows program in a connected folder forwards and writes nothing itself
  (R-REMOTE-1); run directly on a network share it refuses to write (R-LOCK-3). Reading through
  the share with an editor or file browser is always safe.
- **R-XOS-2 MUST** Every path stored in a dossier is relative to the project folder (or, for
  team scripts, to `script_root`) and written with `/`. A path typed with `\`, a Windows drive
  path or a share path that lies inside the project is converted; one that lies outside is
  refused. Output shows the separators of the system the command was typed on.
- **R-XOS-3 MUST** In remote mode a path typed on the PC relative to the connected local folder
  means the same relative path in the project folder on the server.
- **R-XOS-4 MUST** File names the tool creates use lower-case ASCII letters, digits and `-`
  after the ID, never a name Windows reserves (`con`, `nul`, `aux`, `prn`, `com1` … `lpt9`),
  never a trailing dot or space, and stay under 120 characters. Two entries never get names that
  differ only in upper and lower case.
- **R-XOS-5 MUST** `attach`, `take` and `check` warn when two files in a list differ only in
  upper and lower case, or when a file name can't exist on Windows (reserved name, one of
  `< > : " | ? *`, trailing dot or space), because such files can't be copied to a PC.
- **R-XOS-6 MUST** A file written by the tool has `LF` line endings in new lines and keeps the
  endings of lines it didn't change (D17). An entry saved from a Windows editor with `CRLF`
  endings or a byte-order mark is read and changed without converting the rest of the file.
- **R-XOS-7 MUST** Times are written with their UTC offset. Checksums, IDs and the order of
  search results are the same on both systems for the same files.
- **R-XOS-8 MUST** On Linux, files and folders the tool creates are readable and writable by the
  group of the dossier folder, and new folders pass the group on, so that an entry written by
  one team member can be changed by another. `init` sets this up and says which group it used;
  `check` reports files a team member could not write.
- **R-XOS-9 MUST** In remote mode the PC program and the server program compare versions at the
  first command of a session. A different format version is refused with a message saying which
  side to update; a different program version with the same format is a warning.
- **R-XOS-10 MUST** Text passed from the PC arrives byte for byte on the server (R-REMOTE-2),
  whatever the PC's console code page is; output from the server is shown as UTF-8.

### 5.15 History — HIST

- **R-HIST-1 MUST** Every change adds one line to `journal.jsonl` in the state folder: time,
  name, person, command, entry ID, and for field changes the old and new value.
- **R-HIST-2 MUST** Before an entry file is changed or removed, the old file is saved as a
  snapshot in the state folder. `history <ID>` lists the snapshots; `diff <ID> [<n>]` shows what
  changed; `restore <ID> <n>` (a person, at a terminal) brings a version back.
- **R-HIST-3 SHOULD** `log [--since <date>] [--author <name>]` prints the journal as readable
  lines.
- **R-HIST-4 MUST** Everything that is recorded carries who and when: the person, the LLM if one
  wrote it, and the local time to the second with its UTC offset. This holds for every entry,
  value, file list line, exchange record, usage record, take and return, and for every later
  change of them. Nothing is written without both.
- **R-HIST-5 MUST** Each entry file carries its own history as `- history:` lines, one per event
  (created, approved, rejected, changed, superseded, retired, reopened, promoted), each with
  time, person and LLM. So who decided or approved what, and when, can be read in the file
  itself, without the tool and without the journal.
- **R-HIST-6 MUST** `log` answers "who did what when" for the whole dossier:
  `log [--kind <kind>] [--person <name>] [--event <event>] [--since <date>] [--until <date>]`
  prints one line per event, newest first: time, person, LLM, event, ID and title. `show` prints
  an entry's history lines; `list --wide` adds the time and person of creation and approval.

### 5.16 Checking — CHECK

- **R-CHECK-1 MUST** `check` reads every entry and reports, one line each as
  `<file>:<line>: <ID> <KIND> <message>`: files that can't be parsed; missing required fields
  and sections; unknown statuses and tag values; IDs that don't match the file name or appear
  twice; links to entries, tasks, how-tos or files that don't exist; approved entries changed
  after approval; lessons without *Valid for*; entries due for review; in a team dossier, any
  project named other than by code. Exit code `0` when nothing was found, `1` otherwise.
- **R-CHECK-2 SHOULD** `check --fix` repairs what is safe: file names that don't match the
  title, missing format lines, the order of the tool's own fields.

### 5.17 Agents — AGENT

- **R-AGENT-1 MUST** `help agents` prints the rules for LLMs, at most 40 lines: run `brief` at
  the start and `brief --task` before a task; run `status` before proposing or starting work that
  no task asks for; work only on a task you have claimed; follow the decisions in force
  (R-CTX-12); never edit entry files directly; record a decision
  with `decide --said-by` when the person makes one, at once; record observations with `found`;
  after `todo done`, run `review-task` and propose lessons as drafts, or none; record a simulation
  run with `ran` and other software use with `used`; record what you had to assume with
  `assume`; take numbers from `values` and never from memory; suggest a decision with `propose`
  when your person is not the one who decides; attach the files a task reads and produces with `attach`; look for an
  existing script with `scripts` before writing one; never approve; never write a confidential term to the team dossier; use
  `search` before proposing something new.
- **R-AGENT-2 MUST** `agents install` writes these rules as a managed block into `AGENTS.md`
  (below `todo`'s block, if there is one) and keeps every other line; `agents check` tells
  whether the block is current.
- **R-AGENT-3 SHOULD** `hooks install` adds Claude Code hooks: `brief` at session start, and a
  guard that refuses direct edits of files under `DOSSIER/` and names the command to use.
- **R-AGENT-4 SHOULD** `skill install` writes a Claude Code skill with the same rules and a few
  worked examples.
- **R-AGENT-5 SHOULD** `hooks install` also adds a Claude Code hook that runs after every shell
  command of the agent (D39). It reads the hook's payload, as `guard` does, and acts only when
  the command was a `todo` command that succeeded, typed directly or as `dossier todo …`
  (R-REMOTE-5); the task ID is taken from the command or from its output:
  - after a task was claimed (`todo next --claim`, `todo claim <id>`): the output of
    `brief --task <id>` (R-CTX-2);
  - after `todo done <id>`: the output of `review-task <id>` (R-DRAFT-1) and, when the task has
    no usage record, one line that reminds the agent to add one if software was used (R-USE-7);
  - after `todo stuck <id>`: one line that tells the agent to record what was missing or did not
    work, with `found --dead-end` or `assume`, or to open an investigation with
    `problem … --task <id>` (R-INV-9), so that the planner finds it in `brief --task`.

  After any other command it prints nothing. It never blocks or fails the agent's command,
  never changes a task (D38) or an entry, and stays silent when the folder has no dossier or
  `todo` is missing. Its text reaches the agent's context; how Claude Code passes it on is
  checked against Claude Code's documentation when the task is split. `hooks uninstall` removes
  it with the others.

### 5.18 Settings — CFG

- **R-CFG-1 MUST** Project settings in `DOSSIER/dossier.toml`: project code, the
  deciders (R-WRITE-12, D28), path of the team
  dossier, default tags, confidential terms, and overrides of the values below.
- **R-CFG-2 MUST** Team settings in the team dossier's `dossier.toml`: the people (usernames,
  and a separate list of plain names, D34); the allowed values for `machine` and `analysis` (optional); and the
  defaults:

  | Setting | Default | Meaning |
  |---|---|---|
  | `brief_lines` | `60` | longest `brief` |
  | `news_lines` | `20` | longest `news` |
  | `status_lines` | `60` | longest `status` |
  | `draft_days` | `14` | when a draft counts as old |
  | `take_days` | `14` | when a taken file counts as old |
  | `review_months` | none | default `review` date for new lessons and how-tos |
  | `lock_wait_seconds` | `30` | how long a write waits for the lock |
  | `checksum_limit` | `1 GB` | above this, the short checksum is used |

- **R-CFG-3 MUST** An unknown setting or wrong value is an error naming the file, line and
  setting.
- **R-CFG-4 MUST** A name that isn't in the team settings is refused for writing, with the list
  of known names. `people add <person> [--llm <name>…]` adds one. Since D34 an LLM is named
  `<username> - <anything>` and needs no registration; its username must be a member.

### 5.19 Output — OUT

- **R-OUT-1 MUST** `--json` on every command prints exactly one JSON object:
  `{"schema":"dossier.<command>.v1","ok":true,"data":…,"warnings":[…]}` or the same with
  `"ok":false` and `"error":{"code":"…","message":"…"}`.
- **R-OUT-2 MUST** Exit codes as in `todo`: `0` done; `1` refused by a rule or problems found;
  `2` wrong usage; `3` busy, try again; `4` file or settings problem; `5` a needed program or
  connection failed (`todo` missing, ssh failed).
- **R-OUT-3 MUST** Every failure is an error with a code and a message that says what to do
  next. No panics outside tests.
- **R-OUT-4 MUST** Output is written so that a closed pipe ends the program quietly. Output is
  UTF-8, also on a Windows console.
- **R-OUT-5 SHOULD** `help <command>` shows two examples in PowerShell and one in a Linux shell.
- **R-OUT-6 COULD** `schema` prints a machine-readable description of the commands, JSON shapes
  and exit codes.
- **R-OUT-7 COULD** `export [--milestone <task>]` writes one Markdown document with the
  project's decisions, findings and usage records in order, as the basis for a project report
  or a handover.

---

## 6. Commands at a glance

| Command | What it does | Priority |
|---|---|---|
| `init`, `init --team-dossier` | create a project or team dossier | MUST |
| `add`, `decide`, `found`, `learn`, `howto`, `note` | write entries | MUST |
| `propose`, `accept`, `decline` | proposed decisions | MUST |
| `assume`, `confirm`, `refute`, `cleared` | assumptions and what rests on them | MUST |
| `value set/show`, `values` | the project's key numbers | MUST |
| `received`, `delivered`, `exchange` | what came from and went to the customer | MUST |
| `ran`, `runs` | simulation runs | MUST |
| `digest` | summary of a closed milestone | MUST |
| `problem`, `tried`, `solved`, `gave-up` | investigations: a problem, the attempts in order, what solved it | MUST |
| `term add`, `term`, `glossary` | glossary | MUST |
| `pack`, `unpack` | backup archive | MUST |
| `image add`, `image remove` | pictures in entries | MUST |
| `init --past`, `review-folder`, `review-document`, `reviewed-document`, `import status` | bringing in finished projects | MUST |
| `set`, `retitle`, `link`, `edit` | change entries | MUST |
| `review-task`, `drafts`, `approve`, `reject` | drafts and approval | MUST |
| `supersede`, `retire`, `remove` | end of an entry's life | MUST |
| `promote`, `close` | project to team | MUST |
| `software add/list/show`, `used`, `where-used`, `howtos` | software and usage | MUST |
| `show`, `list`, `search`, `history`, `news` | read; what is new since last time | MUST |
| `brief`, `brief --task` | context for a session or task | MUST |
| `status` | the state of the whole project: progress, who does what, decisions, findings | MUST |
| `script add`, `script update`, `scripts` | scripts (macros, Python) and where they are | MUST |
| `attach`, `detach`, `reattach`, `files`, `which` | the files that belong to a task | MUST |
| `take`, `return` | large files copied to a PC and back | MUST |
| `connect`, `todo` | remote mode | MUST |
| `confidential add/list/none` | confidential terms | MUST |
| `check`, `diff`, `restore` | checking and recovery | MUST |
| `people add`, `help`, `help agents`, `agents install`, `--version` | setup and help | MUST |
| `summary`, `story`, `where`, `log`, `reviewed`, `team summary` | overviews | SHOULD |
| `map set`, `map`, `init --code-project` | code projects | SHOULD |
| `review-session`, `confidential scan`, `check --fix`, `connect --check` | extras | SHOULD |
| `hooks install`, `skill install` | Claude Code extras | SHOULD |
| `export`, `schema`, `search --similar` | extras | COULD |

Options used by many commands: `--agent`, `--json`, `--team`, `--project`, `--dossier`,
`--dry-run`.

---

## 7. Folders

A project folder:

```
<project folder>/
  TODO.md                      (todo)
  AGENTS.md                    rules for agents: todo's block and dossier's block
  DOSSIER/
    dossier.toml               project code, team dossier path, default tags, confidential terms
    decisions/DEC-001-….md
    assumptions/ASM-001-….md
    findings/FND-001-….md
    investigations/INV-001-….md   a problem, the attempts in order, what solved it
    values.md                  the key values with their history
    exchange.md                received from and delivered to the customer
    glossary.md                terms of this project
    digests/DIG-001-….md       summary of each closed milestone
    images/FND-004/….png       pictures shown by entries
    lessons/LES-001-….md       lessons kept in the project only
    howtos/HOW-001-….md        how-tos kept in the project only
    usage/USE-001-….md
    scripts/SCR-001-….md       entries for scripts kept in the project only
    files/T-001.md             the files of each task, by role
    taken.md                   the files that are taken, and the log of their versions
    .state/
      write.lock
      ids.json                 highest number given out, per kind
      journal.jsonl
      approved.json            checksums of approved lessons and how-tos
      credential-ok.json       lines a person allowed although they look like a credential
      history/                 snapshots
```

The team dossier:

```
<team dossier>/
  dossier.toml                 people and their LLMs, allowed tag values, defaults
  software.toml                the software list
  projects.toml                project codes and server folders
  glossary.md                  the team's terms, symbols and abbreviations
  lessons/LES-001-….md
  howtos/<software>/HOW-001-….md
  scripts/<software>/SCR-001-….md   entries; the script files are under `script_root`
  usage-index.md               one line per use
  .state/                      as above
```

On a PC, in a connected folder: `.dossier-connect`, holding the server and the project folder.

---

## 8. Non-functional requirements — NFR

- **NFR-1 MUST** One program for Linux x64, running on the team's server without administrator
  rights and without anything else to install, and one `dossier.exe` for Windows 10/11 x64 with
  the C runtime linked in.
- **NFR-2 MUST** The program has no network code. Remote mode uses the `ssh` and `scp` programs
  of the operating system.
- **NFR-3 MUST** Speed on the server with 2,000 entries in the team dossier and 300 in the
  project: `brief` and `search` under 300 ms, a write under 200 ms. No index is kept; if these
  limits can't be met without one, the index is a cache that can be deleted at any time.
- **NFR-4 MUST** Paths with spaces and non-ASCII characters work, in project folders, file names
  given to `take`, and titles.
- **NFR-5 MUST** No `unwrap`, `expect` or `panic!` outside tests; no output with `println!`.
- **NFR-6 MUST** Dependencies: the crates `todo` already approves (its NFR-9). Anything else
  needs a reason in its commit message.
- **NFR-7 MUST** A dossier written by a newer version of the tool is refused by an older one
  with an error that says to update, never half-read.
- **NFR-8 SHOULD** Each program stays under 6 MB.

---

## 9. Testing — TEST

- **TEST-1 MUST** Fixtures are invented and marked as such in their first line. No real project
  data, customer name or team member's name appears in this repository.
- **TEST-2 MUST** Tests run in temporary folders; `todo` is replaced by a stand-in where a test
  needs it, and one test run uses the real `todo` when it is on `PATH`.
- **TEST-3 MUST** Format tests: every example in `format.md` is parsed; a file with Windows line
  endings, mixed endings, a missing final line ending and unknown keys is changed by one command
  and every other byte stays the same.
- **TEST-4 MUST** Concurrency tests: 20 processes add entries at once and all get different IDs;
  a held lock makes a write exit with `3`; a process killed while holding the lock doesn't block
  the next one.
- **TEST-5 MUST** Approval tests: an LLM name can't approve; `approve` without a terminal
  refuses; an LLM's `learn` is a draft and a person's is approved; drafts never appear in `brief`
  or plain `search`; a hand-changed approved entry is marked.
- **TEST-6 MUST** Confidentiality tests: a term in the title, a field, a section and a file path
  each stop `promote`; matching ignores case, spaces, hyphens and underscores; a project with no
  terms can't promote; a command in project B never prints text from project A.
- **TEST-7 MUST** File tests: a second `take` is refused; `return --check` notices a changed
  size, time and content; the short checksum is used above the limit and recorded.
- **TEST-8 MUST** Remote mode tests against a stand-in for `ssh`: arguments with quotes,
  backticks, `$`, umlauts and line breaks arrive unchanged; exit codes pass through; a failed
  copy leaves the take in place.
- **TEST-9 MUST** `brief` tests: the limit holds with 500 decisions; the selection for a task is
  the same on every run; each reason in `--json` is correct.
- **TEST-11 MUST** Windows and Linux tests, run in CI on both systems with the same fixtures:
  the same commands give byte-identical dossier files, the same checksums and the same search
  order; a path typed with `\` and one typed with `/` store the same line; an entry with `CRLF`
  endings and a byte-order mark is changed with every other byte kept; reserved and
  case-colliding names are refused or warned about; on Linux, a file made by one user of the
  group can be changed by another; a version mismatch between PC and server is refused.
- **TEST-10 SHOULD** One manual test plan on the real server: lock behaviour when a Windows PC
  reaches the folder through the network share (O-1), `connect --check` and `approve` through ssh
  from PowerShell, and `take` and `return` of a large file from a PC, by one person and by two
  (the refused and the forced take), including an interrupted copy. (ssh connection reuse is not
  built, D43.) Run before `take` and `return` are used on real work (D47).
- **TEST-12 MUST** Finding tests (`tests/retrieval.rs`): a test dossier of invented entries and a
  fixed list of searches, each with the entry that must be among the first three results.

---

## 10. Build and documentation — REL

- **REL-1 MUST** CI on every push: format check, Clippy with warnings as errors, and the tests on
  Linux and Windows.
- **REL-2 MUST** A README that explains in this order: what the tool is for, what it needs
  (`todo` 0.8 or newer, with a link to its repository, and `ssh` for remote mode), a first project
  in ten commands, how it works with `todo`, remote mode, and confidentiality.
- **REL-3 MUST** `docs/format.md` describes the entry format exactly, with examples of every
  kind.
- **REL-4 SHOULD** A one-page guide for team members who only read and approve, with no command
  beyond `drafts`, `approve`, `search` and `show`.

---

## 11. Releases

The tool is built in six phases, with one patch release (0.3.1) between phases 3 and 4. The
order after phase 3 was set by D45. Each phase ends with something that is used on real work
before the next one starts; what is learned there may change the later phases. The non-functional
requirements (§8), the tests that belong to a phase's requirements (§9) and REL-1 to REL-3 apply
from phase 1 on.

### Phase 1 — Core (v0.1)

**Goal:** one project remembers what was decided, assumed, found and learned, and every LLM
starts its session with it.

**At the end you can:** create a dossier in a project folder; record decisions, assumptions,
findings, dead ends and lessons; approve an LLM's drafts; search; get `brief` and `status`; undo
any change. It runs on the server (or on a PC for a project that lives there, such as the
owner's own software). From a PC, `dossier` and `dossier todo` are typed as if the project were
local: a folder is connected once, the project is noted in a per-user register, and commands are
forwarded through ssh.

| Area | Requirements |
|---|---|
| Places | R-PLACE-1, 2, 7, 8 (a project dossier without a team dossier) |
| Entries: decision, assumption, finding, lesson | R-ENTRY-1 to 7, 10, 13, 14, 15, 19 |
| Identity | R-ID-1 to 4 |
| Writing | R-WRITE-1 to 4, 7 to 11, 14, 16 |
| Drafts and approval | R-DRAFT-1 to 6, 8 |
| Life of an entry | R-LIFE-1 to 6 |
| Reading | R-READ-1 to 6, 8 |
| Context | R-CTX-1 to 5, 9 to 13 |
| Locking, writing, editing | R-LOCK-1 to 3, R-IO-1, 2, R-EDIT-1 to 3 |
| History, checking | R-HIST-1 to 3, R-CHECK-1 |
| Agents | R-AGENT-1, 2 |
| Settings | R-CFG-1, 3; the people and their LLMs are listed in the project settings until phase 3 |
| Output | R-OUT-1 to 5 |
| Backup | R-BACKUP-1 to 3 |
| Running commands from the PC (no file copying yet) | R-REMOTE-1 to 3, 5, 6, 9 to 12 |
| Windows and Linux together | R-XOS-1 to 4, 6 to 10 (R-XOS-5 comes with the file lists in phase 2) |

In this phase every listed person may decide (R-WRITE-2); proposals come in phase 2.

**It worked if:** after three weeks on one real project the dossier holds entries from at least
two people, and an LLM in a new session has followed a decision it was not told about.

### Phase 2 — Project facts (v0.2)

**Goal:** the project's numbers, files and runs are recorded, so that a result can be traced
to what it was made from.

**At the end you can:** keep the key values in one sheet; list the files of each task by role
and see which results are outdated; record a simulation run in one command; show plots in
entries; propose a decision for the project lead; refute an assumption and see what rests on
it; log what came from and went to the customer; ask what was in force on a given date; and use
a dossier in a software project with a map of the code.

| Area | Requirements |
|---|---|
| Key values | R-VALUE-1 to 6 |
| Files of a task | R-FILE-8 to 16 |
| Software use and runs (software names kept in the project for now) | R-USE-2, 14, 15 |
| Pictures | R-ENTRY-16, 17 |
| Proposals and deciders | R-WRITE-12, 13 |
| Assumptions confirmed and refuted | R-WRITE-15 |
| In force on a date | R-LIFE-9 |
| Exchange with the customer | R-EXCH-1 to 5 |
| Code projects | R-CODE-1 to 5 |
| Context | R-CTX-16 |
| Who and when for every change (D29) | R-HIST-4 for `init`, `people add`, `agents install`, the register |
| Short checksums for large files | R-FILE-6 |
| Claude Code hooks and skill (D33) | R-AGENT-3, R-AGENT-4, R-CTX-7 |

**It worked if:** for one delivered report the dossier names the model, the runs, the values
and the assumptions behind every result in it.

### Phase 3 — Team (v0.3)

**Goal:** what one project learned reaches the next project, without customer data, and the
finished projects of the past are brought in.

**At the end you can:** create the team dossier; promote entries after the confidentiality
check; keep the software list, how-tos and scripts; ask where a software was used; keep a
glossary; see what is new; close a project; and draft entries from old project folders and
reports.

| Area | Requirements |
|---|---|
| Places | R-PLACE-3 to 6 |
| Team dossier and promotion | R-TEAM-1 to 6 |
| Confidentiality | R-CONF-1 to 6, R-ENTRY-18 |
| Entries: how-to, usage record, script | R-ENTRY-8, 9, 12, 20 |
| Writing to the team | R-WRITE-5, 6 |
| Software, usage index, scripts | R-USE-1, 3 to 13 |
| Glossary | R-TERM-1 to 4 |
| What is new | R-READ-10, 11 |
| Settings | R-CFG-2, 4 |
| Review dates | R-LIFE-7, 8 |
| Finished projects | R-IMPORT-1 to 9 |

**It worked if:** a task in a new project is shown a lesson and a how-to that came from another
project, and one finished project has been brought in with at least ten approved entries.

### Patch release 0.3.1 — Credentials are refused

**Goal:** no write to a project dossier or the team dossier stores a private key, an access
token or a password, and `check` finds one that was added by hand.

| Area | Requirements |
|---|---|
| Credentials | R-CONF-7 (D40) |

It comes before phase 4 because it protects every dossier that is already in use.

### Phase 4 — Investigations (v0.4)

**Goal:** a person or an LLM records a problem, each attempt to solve it with its hypothesis and
result, and what solved it; the next person or LLM on a similar task sees what was tried and
what worked.

| Area | Requirements |
|---|---|
| Investigations (D41, D42) | R-INV-1 to 10 |

The exact form of an attempt in the file is fixed in `docs/format.md` §28. Pointing to
investigations after `todo stuck` and `todo done` needs the hook of R-AGENT-5 and comes with it in
phase 6.

### Phase 5 — Working from the PC (v0.5)

**Goal:** team members and their LLMs use the dossier from their Windows PCs as if it were
local, and large files are safe when copied out and back.

**At the end you can:** connect a folder on a PC to a project on the server; run every
`dossier` and `todo` command from there; take a simulation file, work on it locally and return
it without overwriting a colleague's version.

| Area | Requirements |
|---|---|
| Remote mode: copying files, connection check | R-REMOTE-4, 8 (R-REMOTE-7 is not built, D43) |
| Large files | R-FILE-1 to 7 |

**It worked if:** all three team members work for two weeks without typing `ssh`, and one
attempted double copy-back was caught.

### Phase 6 — Growing projects and comfort (v0.6)

**Goal:** the dossier stays small enough for an LLM's context in a long project, and the
routine steps happen by themselves.

| Area | Requirements |
|---|---|
| Milestone digests | R-CTX-14, 15 |
| Context extras | R-CTX-6 (R-CTX-8 is not built, see there) |
| Session review | R-DRAFT-7, 9 |
| Reading extras | R-READ-7, 9 |
| Repairs, export, schema | R-CHECK-2, R-OUT-6, 7 |
| Unit warnings | R-ENTRY-11 |
| Team overview, guide for readers | R-TEAM-5, REL-4 |
| Hooks after `todo` commands (D39) | R-AGENT-5 |

**It worked if:** `brief` in a project with more than 200 entries still fits its limit and
still contains what the LLM needed.

### Phase 7 — Finding (v1.0)

**Goal:** an LLM is shown the entries that matter for its work, also when they are worded
differently (D48 to D51).

| Area | Requirements |
|---|---|
| Ranking by stems and rare words | R-FIND-1 to 4 |
| Search | R-READ-3, 4 |
| Topics | R-TOPIC-1 to 5 |
| Code pointers and the check before an edit | R-CODE-6 to 8 |
| Finding tests | TEST-12 |

**It worked if:** in the project's own dossier, `search connection reuse` lists the two entries
about connection reuse among its first three results, and every search of TEST-12 finds its entry.

### Added on 2026-11-22

Three pieces of work were added after phase 3 was finished: R-AGENT-5 (hooks after `todo`
commands), R-CONF-7 (refusing credentials) and investigations (D41, D42, R-INV-1 to 10). D45
places them: R-CONF-7 is the patch release 0.3.1, investigations are phase 4, and R-AGENT-5
belongs to phase 6 but depends on nothing in phases 4 and 5, so it may be built earlier.

### Merged commands

By the owner's decision of 2026-11-20, five overlapping commands were merged so that people and
LLMs have fewer to choose from. Requirement IDs are unchanged.

| Removed | Use instead | Requirement |
|---|---|---|
| `summary` | `status` | R-READ-8, R-CTX-9 |
| `note` | `found --later` | R-WRITE-7 |
| `ran` | `used` with `--model` | R-USE-2, R-USE-14 |
| `howtos`, `scripts` | `list --kind howto` / `list --kind script`, with `--software` | R-USE-5, R-USE-10 |
| `news` | `status --since last` | R-READ-10, R-READ-11, R-CTX-10 |

Where another requirement still names a removed command, the replacement is meant.

---

## 12. Open points

- **O-1 Locks through the network share.** D9 and R-LOCK-3 assume that a lock taken through
  Samba and one taken on the Linux machine may not see each other. This was not tested on the
  team's server. If they do see each other reliably, R-LOCK-3 can become a warning.
- **O-2 Working without a connection.** *Decided 2026-11-19: the team works in its office only,
  always connected to the server. No offline mode.*
- **O-3 The name.** `dossier` must be checked against the commands installed on the server.
- **O-4 How much review the team dossier needs.** *Decided 2026-11-19: the strict version.
  Everything promoted is a draft until a person approves it (R-TEAM-2), and `news` (R-READ-10)
  shows the others what was added.*
- **O-6 Code projects with several copies.** Numbered IDs are safe only with one copy of a
  dossier (D9, D10). A code project where several people each have their own git copy would
  create the same ID twice. Today the owner develops alone, so this doesn't arise. It must be
  solved (for example by IDs that carry the author) before colleagues commit from their own
  copies.
- **O-7 Reaching the server in phases 1 to 3.** *Decided 2026-11-20: running commands from the
  PC is part of phase 1, with a per-user register of projects (R-REMOTE-9 to 12). The team's
  network signs in to the server by key without a password. Copying large files stays in
  phase 5 (numbered 4 before D45).*
- **O-5 Tags on tasks.** R-CTX-3 uses `machine`, `analysis` and `software` fields on `todo`
  tasks. Whether `todo` 0.8's planner writes them, or the project's default tags are enough, is
  to be seen in the first real project.

---

## 13. Ideas, not planned

- **I-1** Connection to OpenProject: a `todo` milestone carries a work package number, and
  decisions and finished tasks are reported to that work package. One-way, after the team has
  used OpenProject for a while.
- **I-2** Search by meaning (embeddings stored next to the entries) when full-text search starts
  to miss entries. Version 1.0 improves full-text finding first (D48); embeddings stay an idea.
- **I-3** An MCP server for clients that can't run commands.
- **I-4** A read-only HTML view of a dossier for reading on the office network.
- **I-5** An overview across all projects for the team lead (codes, open drafts, last activity).
- **I-6** Reading settings out of simulation files to fill a usage record automatically.
- **I-7** Reminders by mail or chat for drafts that wait too long.
- **I-8** Translating entries, for teams that write in two languages.
- **I-9** Measuring whether LLMs follow the agent rules. A test that is skipped by default sets
  up a temporary project, runs a headless agent session on a few prepared scenarios, each with
  three prompts (one that asks for the dossier, one that doesn't mention it, one that pushes
  against it), and checks the journal and the agent's commands against the rules. It prints a
  rate per rule. A rule with a low rate gets a hook (D39) or better wording. One such hook
  would hold a session open at its end until a task that was finished in it has its
  `review-task` and usage record. To be built after R-AGENT-5, so that it measures what the
  hooks leave open. (Idea taken from ECC's `skill-comply`.)
- **I-10** Recording when a lesson was applied again and whether it held: an option on `used`
  such as `--lesson <ID> --held` or `--failed`, next to `--howto`, that adds a line (project
  code, date, held or not) to the lesson in the team dossier, as R-USE-6 does for how-tos. A
  reader then sees whether a lesson held in five projects or was seen once, a single
  contradiction has a place without retiring the lesson, and *Valid for* can grow. To be
  decided when lessons are really reused across projects.

---

## 14. Out of scope

- Managing tasks, deadlines or people: that is `todo` and, above it, the team's project
  management software.
- Storing, versioning or backing up simulation files.
- Calling a model, or any network access by the program itself.
- A server process, a database, or a web application.
- Access rights inside the team: everyone who can reach a folder can read its dossier.
- Deciding what is true: `dossier` records who said what on which evidence; engineers judge it.
- macOS builds, an installer and code signing.
