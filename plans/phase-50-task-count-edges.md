# Phase 50: the task count stays right when a task list is cleared

Issue #233, the round 3 findings of #232 (phase 49). Line 4 of a card counts a session's tasks
from Claude Code's `<config dir>/tasks/<session id>/*.json`. Phase 49 inferred a cleared list
from totals (the largest total, "at most one open" before the clearing), and every review round
found another hole in that inference. This phase replaces it with tracking by task id, which
removes the class of finding. It opens with the sweep of phases 46 to 49.

## Verified facts

- **Claude Code names each task file by its numeric id**, `<n>.json`, beside `.lock` and
  `.highwatermark` (phase 49, read on the owner's machine). The demo writes `1.json` to `7.json`.
- **Claude Code deletes a list's task files once every task is completed** (phase 49, review
  round 1: 65 of 80 folders on the owner's machine were empty, none held an all-completed list).
- **`DirEntry::file_type` does not follow a symlink**, so a symlink named `1.json` is not a
  regular file; `File::set_modified` lets a test age a file.

## Decided

The orchestrator's, from #233 (not to be reopened):

- Only regular files named `<digits>.json` count; other names and directories are skipped
  silently. They are sorted by numeric id and the lowest 256 are read.
- A read is partial, and the previous fact stays, when a directory entry yields an error, or a
  listed task file fails to open, read or parse while its modification time is under 2 s old.
  One that has failed for 2 s or more counts as an open task.
- Each list keeps each id's last seen status. An id that vanishes after it was seen `completed`
  counts as done; one that vanishes while open, with other task files still there, was deleted
  and leaves the list; when the folder empties, every id that vanished in that read counts as
  done. Shown: done = present completed + vanished as done, total = present + vanished as done.
- A missing folder drops the fact and the list; task files after an emptied folder start a new
  list; the all-done fact stays until a new list starts; nothing is remembered across a restart
  (now in `docs/known-limits.md`). Line 4's drawing does not change.

The builder's:

- "Under 2 s old" is the distance between now and the modification time either way, so a broken
  file dated in the future also counts as open once that distance is 2 s or more, never freezing.
- A file whose modification time cannot be read (it vanished between the listing and the open)
  counts as under 2 s old: the read is partial, and the next read sees it gone.
- An id that vanished as done and comes back is not handled: Claude Code never reuses an id
  (`.highwatermark`).

## States

`T` = `<config dir>/tasks/<session id>/`. Every row reads `T` on the draw thread's read, at most
every 500 ms. "Test" names the test of `tests/transcript/tasks.rs` that pins it.

| State | The user sees | Test |
| --- | --- | --- |
| `T` missing | no task fact | `no_tasks_folder_is_no_task_fact` |
| `T` empty, or only `.lock` and `.highwatermark`, no list before | no task fact | `a_tasks_folder_without_a_task_file_is_no_task_fact` |
| tasks `1/3` (one completed, one in progress, one pending) | `1/3` | `the_tasks_done_and_their_total_are_counted` |
| a task without `status`, or an unknown one | counted, not done | `a_task_without_a_status_is_open` |
| a task completes, another is deleted while others remain | `1/1` at the next read | `a_task_that_completes_or_vanishes_is_followed_at_the_next_read` |
| every file gone after `1/2` (#233: the last one open) | `2/2`, green | `a_task_list_whose_files_are_all_cleared_shows_every_task_done` |
| two tasks complete between two reads, then the folder empties (#233) | `3/3`, green, not blank | `a_list_emptied_after_two_tasks_completed_between_reads_shows_every_task_done` |
| 7 tasks, 2 deleted while open, 5 completed, then cleared (#233) | `5/5`, not `7/7` | `open_tasks_deleted_before_the_list_is_cleared_leave_its_total` |
| a read in the middle of clearing 7 completed tasks (3 left), then none | `7/7`, then `7/7`, never `3/3` | `a_list_read_halfway_through_its_clearing_shows_every_task_done` |
| a task file after an emptied folder | a new list, the live count | `a_new_task_after_a_cleared_list_shows_the_live_count` |
| `T` removed while the session runs | no task fact at the next read | `a_tasks_folder_removed_while_the_session_runs_drops_the_task_fact` |
| the transcript lost | the tasks still read | `the_tasks_are_still_read_while_the_transcript_is_lost` |
| a stray `notes.json` beside the tasks (#233) | not counted | `a_file_not_named_by_a_number_is_not_a_task` |
| a directory named `1.json` (#233) | not counted, the count is not frozen | `a_directory_named_like_a_task_is_not_a_task` |
| a task file mid-rewrite, under 2 s old | the previous fact stays | `a_task_file_caught_mid_rewrite_keeps_the_previous_count` |
| a task file over 64 KiB, under 2 s old | the previous fact stays | `a_task_file_over_64_kib_keeps_the_previous_count` |
| a task file over 64 KiB, 2 s old or more (#233) | counted as open | `a_task_file_over_64_kib_for_two_seconds_counts_as_open` |
| a task file that is not an object, under 2 s old, on a first read | no task fact | `a_first_read_with_a_file_that_is_not_an_object_has_no_task_fact` |
| a task file that is not an object, 2 s old or more | counted as open | `a_task_file_that_is_not_an_object_for_two_seconds_counts_as_open` |
| a broken task file dated a minute in the future | counted as open | `a_broken_task_file_dated_in_the_future_counts_as_open` |
| more than 256 task files (#233) | the lowest 256 ids, the same set every read | `over_256_task_files_the_lowest_256_ids_are_counted` |
| a directory entry that yields an error (#233) | the previous fact stays | none: no portable way to make `read_dir` yield an entry error in a test |
| a listed file that vanishes before it is opened | the previous fact stays, gone at the next read | none: the gap between listing and opening cannot be hit from a test; the dangling-symlink test that stood in for it now meets the regular-file rule instead |
| a task completed and the clearing begun between two reads, the read catching it half cleared | that task leaves the total (`2/2` of 3), as a deleted one would | none: a known limit, the cost of telling a deleted task from a cleared one |
| harbour started after the list was cleared | no task fact | none: a known limit |
| `CLAUDE_CONFIG_DIR` set, a `/resume` to another id, an ended card | as phase 49 | unchanged phase 49 tests |

Drawing line 4 does not change, so the narrowest sidebar and a CJK name stay as phase 49's
drawing table has them.

## Tasks

1. The sweep of phases 46 to 49: issues and comments, `### Sweep` below.
2. Tracking by task id in `src/transcript.rs` (`tests/transcript/tasks.rs` red first), the
   `src/AGENTS.md` entry, the known limit, the fragment.

## Outcome

### Sweep

Range: phases 46 to 49 (`scripts/sweep-report.sh`, after phase 45's sweep). The owner's
standing answer made every finding an issue; a finding that repeats an open issue became a
comment on it, as phase 45's sweep did. No rule, `AGENTS.md` or `docs/process.md` changed.

- Plans state how another program behaves without checking it (46: legacy terminal rows written
  from memory, two wrong; 49: Claude Code deleting a finished task list, unknown until round 1):
  issue #235.
- Round 1 found states the table missed in 46, 47 and 49 (Shift and Ctrl forms of keys, Claude
  Code moving folder after the start, a finished list's files deleted): repeats #111, comment there.
- Three of four phases used all three review rounds; 46 and 47 merged a closing ship read only
  by the orchestrator (repeats #84, comment there), 47 left two `src/` nits in its pull request
  body and 49 left #233 (repeats #221, comment there).
- Rerendering the screenshots rewrote untouched images in 47 (`→ none`): repeats #218, comment
  there.
- Guard rows green on their first run in 46: repeats #96, comment there.
- One-offs, not findings: a test file near the 700-line cap (46), a builder run cut off by a
  session restart (48), the demo shots growing with taller cards (49).
- Metrics: review rounds 3, 3, 1, 3; build start to the clean round 33 min 30 s, 46 min,
  17 min 9 s, 59 min 1 s; the reviewer's tokens a round about 109k, 130k, 104k, 125k; the
  builder's last notice 226113, 289645, 189206, 267183; the orchestrator's context at the start
  93826, 221630, 106924, 95694 (fresh sessions, unlike the steady climb of #220). No trend in four
  values. Process drift: none; every phase had a worktree, a review and a metrics row.

Phase 49's ready, merged and released were read from GitHub into `plans/metrics.md` (v0.44.0).

### Tracking by task id

`listing` reads only regular files named `<digits>.json` (by `DirEntry::file_type`, so a
directory or a symlink is skipped), sorted by numeric id, the lowest 256; it returns each id's
completed flag, or `Partial` on an entry error or a file that fails while it is under 2 s old
(`WRITING`); an older failure counts as open. `List` keeps `seen` (each id's last flag) and
`done` (the ids that vanished as done), and the fact is done = present completed + `done`,
total = present + `done`, so the old `largest`, `last` and `cleared` are gone. Nine tests are new
or changed: eight red first, and the future-dated one, green on its first run, shown red by
breaking the age check, then restored; the 256-file test became the sorted one, and the dangling-symlink
test was deleted, since the regular-file rule now skips that symlink. `docs/known-limits.md`
gained the restart and the half-cleared race. Fragment `kind: fix`.

## Hurdles

- `tooling` restoring `src/transcript.rs` with `mv` from a copy older than the last build left cargo thinking the library fresh, so the restored code ran the broken build and a test failed after the restore; a `touch` fixed it → none
