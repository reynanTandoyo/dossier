<!-- dossier-format: 1 -->
# DIG-002 · Digest of T-604: Entries say what comes next: revisit-if, settle-by, typed links, lesson checks and sweep

- kind: digest
- status: draft
- created: 2026-11-27T21:46:22+01:00
- author: reyna
- via: reyna - claude
- milestone: T-604
- tasks: T-604, T-605, T-606, T-607, T-608, T-609, T-610, T-611, T-612, T-613, T-614, T-615, T-616, T-617, T-618, T-619, T-620, T-621, T-622, T-623
- history: 2026-11-27T21:46:22+01:00 reyna via reyna - claude: created

## Summary

Version 1.4.0 (tag v1.4.0) adds five things from the logbook study; the owner chose them (D59 to D63) and set six other ideas aside (D64).
Decided and binding: what was ruled out may say when to look again (revisit-if, D59); an assumption may name what would settle it (settle-by, D60); a link may be typed refines or contradicts (D61); a lesson may name the check that carries it, owed or checked (D62); sweep lists look-alike entries and changes nothing (D63).
Built: decide, propose, decline and found --dead-end take --revisit-if; decide lists the ruled-out entries on the same question; search shows the condition.
Built: assume --settle-by; brief and status show it and count the assumptions that name nothing.
Built: link --as refines|contradicts; search, brief --task and which --binding name the link type, contradictions first; status lists contradicting entries in force.
Built: learn --check and the new command checked (a person, or an LLM with --said-by); status counts lessons that owe a check.
Built: the new command sweep; the planner rules explain all five.
Found: on this project's own dossier, sweep lists 8 look-alike pairs (for example D52 and D57); none has been judged yet.
Found: twelve Haiku workers in parallel worktrees took about 20 to 25 minutes, mostly the full test runs; no task came back stuck.
Found: todo done runs a task's verify with a 300 s limit; the full suite ran past it once while workers were building, and passed on the second run.
Open: the five open assumptions now each name what would settle them; ASM-001 and ASM-005 wait for the manual server test T-412.
Did not work: a first cherry-pick failed to update main while worktrees were being cleaned up; the commit was made again from the staged change.
Checks: 2620 tests on Windows, 2624 on Linux (WSL) and CI passed.
