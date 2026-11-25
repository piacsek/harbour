# Phase 45: old ended cards tidy themselves away

Issue #195. Cards of sessions that ended pile up in the sidebar: every exited session, and every
conversation that comes back stopped after a restart. Harbour now removes a card that has not run
for a while by itself, as `x` would, after 6 hours by default.

## Verified facts

- **An exited card knows when harbour saw its exit**: `Entry::exited_at`, an `Instant` set by
  `settle_exited` (`src/app/read.rs`). A stopped card (`session: None`) is only ever a recalled
  one (`recall` in `src/app/remember.rs`), and it carries no time of its own.
- **remember.rs ages a remembered file by its own modification time** (`Memory::forget_stale`,
  30 days, only for a file whose transcript is gone). That file is rewritten by a move, a rename
  and every regroup that changes a position, so it says when harbour last wrote it, not when the
  conversation last changed. The transcript's modification time does say that; nothing reads it
  yet but `transcript::meta`, against a session's start.
- **`x` on an ended card** goes through `answer_yes(Question::End)`: `session::end_all` on its
  sessions, then `remove_selected`, which drops the reservation and calls `forget`, which keeps a
  file another card of this harbour shares or a live session or another harbour holds
  (`held_ids`).
- **`Entry::running` is true while the session or any of its editors runs**, so an exited card
  whose editor still runs (with edits maybe unsaved) counts as running.
- **The first frame reads the files** (`read_at` starts `None`), and `remember_all` recalls the
  stopped cards in that read, before the frame draws.
- **The demo builds its `App` with `App::demo`**, never from the config, so it never tidies.

## Decided

- The orchestrator's decisions are in issue #195 and the build brief; these are the builder's.
- `tidy_after` is a string: `"<n>h"`, `<n>` ASCII digits for a whole number of hours from 1 up
  to `u32::MAX`, or `"off"`. `"0h"`, `"6"`, `"+6h"`, `" 6h"`, `"6H"`, a number and anything else
  are refused with `expected a whole number of hours above 0, as "6h", or "off"`. The variable
  takes the same text, an empty one counts as unset as for the other variables, one that is not
  UTF-8 is ignored and `harbour config` says so, as for `HARBOUR_PREFIX`. A bad variable is
  refused by its name.
- A stopped card's age is its transcript's modification time, read in one listing of the
  projects folder at each check; a card whose transcript is not found (or no projects folder) is
  never tidied. The remembered file's own time is not used (see above).
- The cards due are picked by a pure function, `tidy::due` in `src/tidy.rs`: a card that is not
  running, not the selected one, with an age of at least `tidy_after`.
- The check runs at the first frame and then every 60 s (`TIDY_EVERY`). A check that falls while
  a question, the directory picker or the session finder, the usage popup, the rename popup or a
  waiting prefix is open is put off to the first frame after it closes, so the prefix then a digit
  never lands on a card that moved.
- `App::with_tidy_after(Duration)` takes any duration and `App::with_tidy_every(Duration)` the
  interval; only tests call the second (`SEAMS`). Without `with_tidy_after` nothing is tidied.
- The removed card's sessions are ended as `x` ends them; the selection stays on its card.

## States

| What | State | What the user sees |
| --- | --- | --- |
| card | running (session or an editor) | never tidied |
| card | exited, its editor still running | never tidied (it runs) |
| card | exited for `tidy_after` or longer | removed at the next check |
| card | exited for less | kept |
| card | stopped, transcript changed `tidy_after` ago or longer | removed at start, or at a check |
| card | stopped, transcript newer | kept |
| card | stopped, transcript gone or not found | kept (recall already drops a gone one) |
| card | stopped, transcript changed in the future (clock) | kept: age 0 |
| card | the selected one | kept while selected; removed at a check after the selection moves |
| card | the only card | kept: it is the selected one |
| card | stopped, open elsewhere (live pid or another harbour) | its transcript grows, so kept; if idle that long, removed here and its file kept for the holder |
| card | removed, conversation | still with Claude Code: `claude --resume` finds it |
| remembered file | of a removed card, held by no one | forgotten, as `x` forgets it |
| remembered file | shared with another card of this harbour | kept |
| screen | a question, a picker, the finder, the usage or rename popup, a waiting prefix | no check until it closes |
| setting | `"6h"` (default), `"1h"`, `"off"` | tidies after 6 or 1 hours, or never |
| setting | `"0h"`, `"6"`, `"+6h"`, `6`, `"6H"` in the file | refused, naming file, line and `tidy_after` |
| variable | `HARBOUR_TIDY_AFTER` set and good | outranks the file; `harbour config` says `variable HARBOUR_TIDY_AFTER` |
| variable | bad | refused, naming `HARBOUR_TIDY_AFTER` |
| variable | empty | the file's, else the default |
| variable | not UTF-8 | ignored; `harbour config` says so |
| config file | missing, or no `tidy_after` | `"6h"` (default) |
| `--demo` | any | never tidies |
| log | a check that removed `n` cards | `tidy cards=<n>`, never a title |

No text is drawn by this phase, so no narrow-sidebar or CJK rows; `docs/details.png` is expected
to stay the same.

## Tasks

1. The sweep of phases 41 to 44 (`### Sweep` below), its issues, phase 44's row.
2. `tidy_after` read from the file and `HARBOUR_TIDY_AFTER`, listed by `harbour config`
   (`tests/config/`).
3. `tidy::due` picks the cards (`tests/tidy.rs`).
4. The app tidies at start and every check, skips while something is open, logs `tidy cards=<n>`
   (`tests/app/tidying.rs`); wired from the config in `src/tui.rs`.
5. The end-to-end test (a stopped card tidied at start under `HARBOUR_TIDY_AFTER=1h`, kept under
   the default), `harbour --help`, `src/AGENTS.md`, the fragment.

## Outcome

### Sweep

Range: phases 41 to 44 (`scripts/sweep-report.sh 41`; the sweep before phase 40 was not run, so
phases 36 to 40 were last read by nobody, and only cross-checked here where a hurdle repeats one
of theirs). The owner's standing answer took every proposal as an issue or a decline; no rule in
an `AGENTS.md` or `docs/process.md` changed.

- `tooling` → none, four times. Grouped:
  - Rerendering the screenshots rewrites every image and the untouched ones differ by raster
    noise (44, and 40 before it, both → none; a `PLAN.md` backlog line has waited since
    conveyor): issue #218.
  - `openpty` failing while other tests run (43, and phases 1 and 15 before it, all → none),
    though #70's gates lock queues the gates and ships: a builder's or reviewer's own `cargo test`
    takes no lock: issue #219.
  - `ship.sh`'s `git add -A` took a half-done change into a commit (43): already #110, open since
    the sweep before phase 30, which this repeats; declined as a new issue, the phase added as a
    comment on #110.
  - Waiting on a ship with `pgrep -f` matched itself (43): declined, a one-off in the waiter's
    own command, not in the repository's tools.
- `review-loop`, four: round 1 found states the plan's table missed in 41, 43 and 44 (an editor
  with unsaved changes behind Ctrl+C twice; a config already binding `r`, a card open elsewhere;
  a header alone on the last row, a move before a write): already #111; declined as a new issue,
  the phases added as a comment on #111. Every phase ended at the owner's 2-round limit and left
  its round-2 findings in an issue (#199, #203, #211, #216), none built yet, seven
  `review-follow-up` issues open: issue #221.
- `test-seam`, four: a test that passed with the code broken (42) and tests green on their first
  run (43) repeat #96, open since the sweep before phase 25; declined as a new issue, the phases
  added as a comment on #96. The two of 44 (every in-process test starting in one folder, a read
  before the agent wrote its id) are one-offs, each fixed in code: declined.
- Metrics: review rounds 2, 2, 3, 2 (43's third reviewed a change the owner asked for after
  round 2): flat, held by the owner's 2-round limit. Start to the end of the clean round 35 min,
  35 min, 1 h 50 min, 1 h 6 min, and the reviewer's tokens a round about 96k, 116k, 162k, 132k:
  up then down over four values, no trend. The orchestrator's context at each phase's start rose
  124557, 234086, 324413, 420912 (469587 at the end of 44), one session with no compaction,
  about 100k a phase: issue #220.
- Process drift: none; every phase had a worktree, a review and a metrics row.

Phase 44's ready, merged and released were read from GitHub into `plans/metrics.md` (v0.39.0).

### Tidying

`tidy_after` (`"<n>h"` or `"off"`, `"6h"` by default, `HARBOUR_TIDY_AFTER` first) is read with
the other settings and listed by `harbour config`; a bad value is refused naming file, line and
key, or the variable. The app checks at the first frame, after the stopped cards are recalled,
then every 60 s, and removes as `x` does every card `tidy::due` picks: not running (an exited
card whose editor still runs counts as running), not selected, and ended for at least
`tidy_after`, an exited card from when harbour saw its exit, a stopped one from its transcript's
modification time. A check due while a question, a picker, the finder, the usage or rename popup
or a waiting prefix is open waits for it to close. Each check that removed cards logs
`tidy cards=<n>`. Because the selected card is never tidied, the last card of a sidebar stays.
`src/config.rs` passed the 700-line cap, so `report` and its helpers moved to
`src/config/report.rs`. `harbour --help` gained two lines, so `docs/help.png` was rerendered;
`docs/details.png` came back byte-identical (the demo never tidies). Tasks 2 to 5 shipped
together: each new `pub` item is dead code until the app and `src/tui.rs` use it.

## Hurdles

- `tooling` `src/config.rs` sat at 699 lines, so one setting broke `tests/source_size.rs`; `report` moved to `src/config/report.rs` → code
- `test-seam` the variable's `"off"`, the stopped card kept while its conversation is fresh, the selected card and the three "nothing while open" tests were green on their first run (their stubs already kept cards); each was shown red by breaking the code it covers → code
- `process` tasks 2 to 5 could not ship apart: `Config::tidy_after`, `tidy::due` and the app's seams are dead code until the app and `src/tui.rs` use them (already #97) → backlog
- `tooling` the first ship of the feature stopped at clippy on a `u64` to `usize` cast in the new end-to-end test, which a targeted `cargo test` run does not lint, and the next at `tests/real_config.rs`: the two new `harbour config` tests did not remove the session, picker and editor variables → code
