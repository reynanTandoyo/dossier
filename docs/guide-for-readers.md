# Guide for readers: drafts, approval and search

A dossier is a `DOSSIER` folder in each project, next to its tasks. It keeps what the team decided, found, assumed and learned, as one short file per item, each with an ID such as `DEC-012`. What an LLM (an AI assistant) proposes waits as a **draft** until a person approves it, and only approved entries count as knowledge.

You never write an entry. Run the commands in a terminal: PowerShell on your PC, or a shell on the server.

## What waits for me: `dossier drafts`

Each line gives the ID, the kind, the title, who proposed it and through which LLM, the task and the age, oldest first. A draft older than the project's limit (14 days by default) is marked `(old)`.

```
LES-004  lesson  Steady runs overpredict the stall margin  by anna via anna-claude  task T-042  3 days
LES-005  lesson  Mesh refinement moves the choke margin  by ben via ben-claude  task T-044  16 days  (old)
2 drafts
```

## Approving: `dossier approve <ID>`

Only a person can approve, and only in a terminal; for a team dossier entry, add `--team` after the ID. The command shows the whole draft, then asks:

```
LES-004: approve, reject or skip? (a/r/s) a
LES-004: approved
```

- `a` (or `approve`): the draft becomes approved, with your name and the time. It now counts as knowledge.
- `r` (or `reject`): the draft is rejected. It is hidden, but kept, so the same proposal can be recognised later.
- `s`, any other answer (`e` included), or just Enter: nothing changes. The draft stays a draft and comes up again next time.
- There is no answer for editing. If a draft needs a change, do not approve it as it is; ask the person who wrote it to change it, and read it again.

Before you answer `a` to a lesson, check three things:

- **Valid for**: the conditions under which the lesson holds, such as "transonic front stages, tip Mach number above 1.1". Outside them the lesson is wrong. A lesson without this section, or without a confidence, is skipped with a warning and cannot be approved.
- **Evidence**: the report, run or test it rests on. Check that it says what the lesson says.
- **Confidence**: `measured` (checked against test data), `simulated` (seen in simulation only), `experience` (seen repeatedly, not checked systematically) or `guess`. A guess is a reason to check, not a rule.

## Finding things: `dossier search` and `dossier show`

`dossier search stall margin` looks in the project and the team dossier, and finds approved entries only. Each line gives the ID, the title and the line that matched; an entry from the team dossier has `team:` before its ID. Add `--drafts` to include drafts.

```
LES-001  [changed after approval] Steady runs overpredict the stall margin  — Near the stability limit, steady runs ...
1 match
```

`dossier show LES-001` prints the entry as it is written, with its marks on the first line:

```
[due for review] [changed after approval]
# LES-001 · Steady runs overpredict the stall margin
```

The marks you will meet:

- `[draft]`: not approved yet. It is not knowledge.
- `[changed after approval]`: an approved lesson, how-to or tool was edited by hand after approval, so its text is no longer the one that was approved. Do not rely on it until someone who runs the dossier commands has checked it again.
- `[due for review]`: its review date has passed. Check that it still holds.
- `[imported]`: it was written from an old project's document after that project had finished, not when the work was done. It is less certain than an entry recorded at the time.
