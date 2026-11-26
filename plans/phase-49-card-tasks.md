# Phase 49: each sidebar card shows how many of its session's tasks are done

Issue #231. A card's line 3 holds the repository, branch, pull request and the subagent count;
the owner wants a fourth line for agent activity: the subagents, then how many of the session's
tasks are done, `3/7`. Line 3 keeps only the repository, branch and pull request.

## Verified facts

- **Claude Code keeps a session's tasks in `<config dir>/tasks/<session id>/<n>.json`**, one
  file per task, beside `.lock` and `.highwatermark`, each an object with `"status"` one of
  `pending`, `in_progress` or `completed` (read on this machine: 80 folders, 166 task files of
  265 to 710 bytes). The folder's name is the session id, the transcript's file name.
- **Harbour finds the config dir once**: `registry::claude_dir` (`CLAUDE_CONFIG_DIR`, else
  `~/.claude`), and hands `Transcript` its `projects` folder. The tasks folder is that folder's
  sibling, so `CLAUDE_CONFIG_DIR` is honoured with no new wiring.
- **`Transcript::read` already counts the running subagents** every time the draw thread reads
  a session (at most every 500 ms), once the transcript is found; the task count rides on the
  same read.
- **Only `src/card.rs` knows a card's height** (`HEIGHT`, `STEP`): the scroll, the header rows
  and the drawing all derive from it. The sidebar has no click-to-select, so no mouse
  hit-testing follows the height; `j`/`k`, `jump` and the finder work by index.
- **The subagent glyph shows only on line 3** (`facts::Kind::Subagents`); the zoomed and
  side-by-side headline draws the repository and branch only.

## Decided

- The owner's (the issue): line 4 on every card, blank with nothing to show; subagents then
  tasks; the subagent glyph `md-family_tree` (U+F160E); open tasks `md-clipboard_check_outline`
  (U+F08A8) dimmed, all done `md-clipboard_check` (U+F014E) with its count in green; without
  icons `2 subagents · tasks 3/7`; no task fact on an ended card; cut from its end like line 3.
- The orchestrator's: green is the fixed `Color::Green`, no new `[colors]` key.
- The builder's: without icons, the whole `tasks 7/7` is green when all are done, since each
  fact is drawn in one colour; at most 256 task files of at most 64 KiB each are read a time.
- The orchestrator's, in review: Claude Code deletes a list's task files once every task is
  completed (65 of 80 folders on the owner's machine were in that state, none held an
  all-completed list). After two rounds of patches to inferring a cleared list, the rule is:
  a read counts only when it read and parsed every task file it listed, else the previous fact
  stays; while files are there, the largest total since the list began is kept (a list begins
  when files appear in an empty or missing folder); a folder emptied after a list shows that
  largest total all done, green, only when the last whole read had at most one task open, else
  no task fact.

## States

Reading the tasks folder, `T` = `<config dir>/tasks/<session id>/`:

| State | The user sees |
| --- | --- |
| `T` missing | no task fact |
| `T` empty, or only `.lock` and `.highwatermark`, and no task seen before | no task fact |
| every task file gone after a whole read with at most one task open (Claude Code clears them once all are completed) | every task done: the largest total of the list, `7/7`, green |
| every task file gone after a whole read with two or more open (`1/3`, the agent deleted the open ones) | no task fact |
| a read halfway through the clearing (`3/3` of a list of 7), then none left | `7/7`, the largest total of the list |
| a new task file after that | the live count again |
| `T` unreadable (permissions) | no task fact |
| a task file malformed, not an object, unreadable or over 64 KiB, or caught mid-rewrite | the previous read's fact stays; on a first read, no task fact |
| a listed file that vanishes before it is opened | the previous read's fact stays |
| a task file without `status`, or an unknown one | counted in the total, not done |
| a file appears or vanishes between two reads | the count follows at the next read, 500 ms later |
| `T` removed while the session runs | the task fact goes at the next read |
| the transcript lost (renamed or removed) | the tasks are still read, since they need only the session id |
| harbour restarted with `T` already cleared | nothing remembered: no task fact |
| more than 256 `.json` files | the first 256 listed are counted |
| `CLAUDE_CONFIG_DIR` set | `T` under it, as the transcript is |
| the session exited or stopped | no task fact, no subagents: line 4 blank |
| the registry names another session id (a `/resume` inside it) | the tasks of that id, as the transcript follows it |

Drawing line 4, `S` = the subagent glyph, `O` the open-tasks glyph, `D` the done glyph:

| Facts | Width | `nerd_font` | Line 4 |
| --- | --- | --- | --- |
| 2 subagents, 3 of 7 done | 40 | on | `    S 2 O 3/7`, all dimmed |
| 2 subagents, 7 of 7 done | 40 | on | `    S 2 D 7/7`, `S 2` dimmed, `D 7/7` green |
| 2 subagents, 3 of 7 done | 40 | off | `    2 subagents · tasks 3/7`, all dimmed |
| 1 subagent, 7 of 7 done | 40 | off | `    1 subagent · tasks 7/7`, `tasks 7/7` green |
| no subagents, 3 of 7 | 40 | either | the task fact alone |
| 2 subagents, no tasks | 40 | either | the subagents alone |
| neither | any | either | blank; the card keeps its 4 rows |
| 2 subagents, 3 of 7 | 20 (narrowest) | off | `    2 subagents · t…` |
| 2 subagents, 3 of 7 | 20 | on | `    S 2 O 3/7`, whole |
| a CJK title and branch | 20 | either | lines 1 to 3 as today; line 4 holds only digits and ASCII |
| exited or stopped, with facts | any | either | blank |

## Tasks

1. `Transcript` counts the session's tasks: `Facts::tasks` (`tests/transcript/tasks.rs` red first).
2. Cards grow to 4 lines; line 4 holds the subagents (moved from line 3, the new glyph) and the
   tasks (`tests/card/` red first); scroll and group tests follow the height.
3. The demo shows a card with open tasks and one with all done; screenshots, the `AGENTS.md`
   entries, the known limit of the draw thread's reads, the fragment.

## Outcome

`Transcript::read` counts the session's tasks beside its subagents, from the projects folder's
sibling `tasks/<session id>/`, so `CLAUDE_CONFIG_DIR` needed no new wiring: `Facts::tasks` is
`Some(Tasks { done, total })` once a `*.json` task file is there, done the `completed` ones,
total every JSON object, at most 256 files of at most 64 KiB, before the transcript is looked
for, since they need only the session id. A read counts only when every listed file was read and
parsed, else the previous fact stays. Claude Code deletes a list's files once every task is
completed, so an emptied folder shows the list's largest total all done when the last whole read
had at most one task open, else no task fact (review rounds 1 and 2). Every card
has 4 lines; line 4 holds the subagents (moved from line 3) and the tasks through the same
`facts::fitted` as line 3, so it is cut from its end the same way, blank on an ended card. The
subagent glyph is now `md-family_tree`; the tasks use `md-clipboard_check_outline` while open,
dimmed, and `md-clipboard_check` with the count in green once all are done (the whole
`tasks 7/7` green without icons). The scroll, the headers and every test of the cards' rows
follow the height through `HEIGHT`; the sidebar has no click-to-select, so no hit-testing
changed. The demo writes 3 of 7 tasks for the working card and 4 of 4 for the idle one, its task
files before its transcript so a reader that waits for the transcript sees them; the demo shots
grew from 26 to 32 rows so every card still shows, and the `icons` shot gained a task list. The
known limit of the draw thread's reads names the task files. Fragment `kind: feat`.

The review loop ended at the 3-round limit with no must left. Round 3's findings, all edge
states of the cleared-list and partial-read rule, went to #233.

## Hurdles

- `review-loop` round 1 found the all-done state never shows, since Claude Code deletes a finished list's files; the last total seen is now kept as done → code
- `design` two review rounds on the cleared-list inference → redesigned as whole reads plus the largest total → code
- `design` taller cards no longer fit the demo's 26-row shots and two e2e demo screens; every demo shot and those tests grew to fit → code
- `review-loop` the cleared-list inference drew findings in all three rounds; round 3's went to #233 → backlog
