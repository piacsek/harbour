# Phase 5: the counts in the tmux status line

## Context

With harbour off screen, in another tmux window, the user cannot see that an agent is waiting
on them. tmux-agents prints such counts for Claude Code sessions in plain tmux panes, but it
cannot see the sessions harbour starts: harbour removes `TMUX` from their environment, so they
belong to no pane. Issue #8 asks for `harbour status`, which prints the counts of every running
harbour's sessions as tmux status-line text, for `status-right`. Only counts, only harbour's own
sessions.

Phase 5 is a sweep phase, so its `## Outcome` opens with the sweep of phases 0 to 3. Phase 4
(PR #14, the directory picker) is still open, so its hurdles are not on `main` and are not swept.

## Verified facts (2026-11-20 UTC, `main` at 57065de)

- `src/state.rs` already turns each session into a state (working, blocked, idle, exited,
  unknown) and counts them for the sidebar title inside `state::title`, as `[working, blocked,
  idle and exited, unknown]`. `App::draw` calls it on every draw, after `read_registry`, which
  reads the registry at most every 500 ms.
- tmux-agents (`src/status.rs`) prints its segments in the order blocked, working, idle, unknown,
  each `#[<style>]<glyph> <count>#[default]`, joined by one space, a state with no agent left
  out; its styles are `fg=red,bold`, `fg=yellow`, `dim` and `fg=brightblack`. Issue #8 asks for
  `#[fg=red]`, not bold, and for the sidebar's glyphs, so unknown is `·` here, not tmux-agents'
  `○`.
- tmux 3.7c accepts `brightblack` as a colour name (`man tmux`, "bright variants brightblack,
  brightred, ..."), and refreshes `#()` commands at `status-interval`, 15 s by default.
- Rust 1.98.1 (the pinned toolchain) has `File::lock`, `File::try_lock_shared` and
  `TryLockError::WouldBlock` in `std`, stable since 1.89. They are `flock` on Unix: the kernel
  drops the lock when its holder dies, however it dies.
- The e2e launcher and `scripts/screenshots.sh` already unset `XDG_STATE_HOME` beside their
  scratch `HOME`, so a harbour they start writes its state under the scratch `HOME`.
- The `help` shot of `scripts/screenshots.sh` is 7 rows high, one more than today's usage; a
  longer usage needs a taller shot (phase 1 hurdle: a fixed height cut it off silently).
- `tests/cli.rs` pins the usage text word for word; `tests/readme.rs` pins `--version`,
  `--help` and `changelog` in the README usage. The Homebrew formula's test needs only
  `--version` and `usage: harbour` on an unknown argument.

## How a running harbour tells `harbour status`

Each harbour on a terminal (not `--demo`) owns one publication in
`$XDG_STATE_HOME/harbour/status/`, else `~/.local/state/harbour/status/`, under a name no other
publication has: its process id, the time it was made in nanoseconds and a counter
(`<name>` below):

- `<name>.lock`, empty, created new, which it holds an exclusive `flock` on for as long as the
  publication lives;
- `<name>.counts`, one line `<blocked> <working> <idle> <unknown>`, written to `<name>.tmp` and
  renamed over, so a reader never sees half of it, and only when a count changed.

When a write fails, or a check (at most every 5 s) finds either file gone, the publication is
dropped (its files removed, its lock released) and a fresh one, with a new name, is made at
most every 5 s.

Once a new publication holds its lock, harbour sweeps the folder: for each regular file named
`<digits>-<digits>-<digits>.lock` on which an exclusive non-blocking lock succeeds, it removes
that name's `.counts`, `.tmp` and `.lock`. Nothing else in the folder is touched.

`harbour status` lists the `*.counts` files, takes a non-blocking shared lock on the matching
`.lock` for each: if the lock is free, its harbour is gone and the file is skipped; if it is held,
the counts are added. It never waits, writes nothing and deletes nothing.

## States

| State | `harbour status` prints | Exit |
| --- | --- | --- |
| no harbour has ever run (no directory) | nothing | 0 |
| no harbour running (directory empty) | nothing | 0 |
| a harbour with no session (all counts zero) | nothing | 0 |
| one harbour with sessions | its counts, states with zero left out | 0 |
| two harbours with sessions | the sum of both | 0 |
| a summary left by a harbour that crashed or was killed (lock free) | nothing for it; the next harbour that makes a publication removes it | 0 |
| a file in the folder not named like a publication, or a symlink named like one | nothing for it; never removed, never followed | 0 |
| a new harbour's lock removed by another harbour's sweep before it locked it | nothing for it until its check (at most every 5 s) finds the lock gone and a fresh publication is made | 0 |
| a crashed harbour's pid reused by another process | nothing for it: the lock died with the old process | 0 |
| a summary that cannot be read or parsed | nothing for it | 0 |
| a `.counts` with no `.lock` beside it | nothing for it | 0 |
| a harbour that just started, before its first draw | nothing for it (no counts yet) | 0 |
| a harbour that quit normally | nothing: it removed both files | 0 |
| no `HOME` and no `XDG_STATE_HOME` | nothing | 0 |
| `harbour --demo` running | nothing for it: the demo writes no summary | 0 |
| the state directory cannot be created or written (harbour side) | harbour runs as before, publishing nothing | n/a |
| the status folder is a symlink or not a directory (harbour side) | harbour runs as before, publishing nothing | n/a |
| a stale pair with this harbour's pid, a reader holding its lock (harbour side) | harbour's own counts: its name differs from the stale pair's | 0 |
| two harbours with one pid on two hosts sharing the folder | the sum of both: neither touches the other's files | 0 |
| a write fails after an earlier one succeeded | nothing for that harbour until a fresh publication works, tried at most every 5 s | 0 |
| the status folder removed and made again while harbour runs | nothing for it until its check (at most every 5 s) finds the files gone and a fresh publication is made | 0 |

The line, for one blocked, one working, three idle and no unknown session:

```text
#[fg=red]◉ 1#[default] #[fg=yellow]● 1#[default] #[dim]○ 3#[default]
```

followed by a newline; when nothing is counted, nothing at all is printed.

## Decisions

- **The counts are the sidebar title's counts.** `○` counts idle and exited sessions together, as
  in the title, so the status line and the sidebar never disagree. One function counts for both.
- **Liveness is a lock, not a pid check.** `kill(pid, 0)` would trust a crashed harbour's file
  again as soon as its pid is reused; a `flock` dies with its holder, so a stale file can never
  count again. Two files per publication, since an atomic rename would replace the locked file.
- **Each publication has a name of its own** (pid, creation time, counter), so a stale pair, a
  reader holding its lock, or a harbour with the same pid on another host sharing the folder
  never stops or touches it.
- **A harbour that makes a publication removes the dead ones; `harbour status` removes nothing.**
  A harbour closed with its pane gets SIGHUP and leaves its pair, and with unique names nothing
  would ever reuse it: at 10 a day `harbour status` would pass 200 ms within a year (about 58 ms
  per 1000 stale pairs, release build, macOS 26.5 on arm64, median of 50 runs: 8.5 ms with an
  empty folder, 66 ms with 1000). With unique names a dead pair is safe to tell: an exclusive
  non-blocking lock on its `.lock` succeeds only when nobody holds it. The sweep runs where a
  publication is made (at start, and at most every 5 s after a failure), not in the reader:
  tmux runs `harbour status` from every client every few seconds, so it stays a reader that
  writes, deletes and waits on nothing, and the cost of a sweep is paid once per harbour start.
  A dead pair now lives until the next harbour starts. A sweep that wins the race against a new
  harbour that created its lock but had not locked it yet removes that lock; the new harbour's
  check finds it gone and makes a fresh publication, the same path as any failure. The sweep
  only touches regular files named `<digits>-<digits>-<digits>.lock` and that name's `.counts`
  and `.tmp` in its own folder, and removing a path never follows a symlink.
- **The demo publishes nothing**, so made-up acme sessions never reach the user's real status
  line.
- **Publishing is best effort.** A state directory harbour cannot create or write leaves the
  interface as it was; nothing is drawn about it. Taking the lock never waits.
- **Any failure drops the whole publication.** A failed write, or a check that finds the lock or
  the counts gone, removes the publication's files and releases its lock, so `harbour status`
  skips it instead of showing old counts; nothing re-locks an old handle. A fresh publication is
  made at most every 5 s (`RETRY` in `src/summary.rs`), and the same interval spaces the checks
  while the counts stay the same, so a broken folder costs at most one attempt per 5 s.
- **The status folder is private**: harbour sets `harbour/status` itself to `0700`, whether it
  created it or found it, and creates its files `0600`; it publishes nothing when that folder is
  a symlink or not a directory. Folders above it keep the modes they have. An `XDG_STATE_HOME`
  that is empty or relative counts as unset, as the XDG spec says.
- **`harbour status` takes no option** and always exits 0, so a status line never shows an
  error; the tmux line to add goes into `--help`.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. This plan with the sweep, its rows in `PLAN.md` and `plans/metrics.md`, and the phase 3
   backfill.
2. `harbour status` prints nothing without summaries, the usage and README list it, and the
   fragment.
3. `harbour status` adds up the summaries of running harbours, in the issue's format, and skips
   stale, unreadable and lock-less ones: one test per row of the table, against the real binary.
4. A running harbour writes its summary when a count changes (private, atomic) and removes it on
   quit; `state` counts once for the title and the summary.
5. The binary wires it up: an e2e test runs harbour in a scratch tmux server and reads its
   counts with `harbour status`; the help shot is rerun.

## Outcome

### Sweep

Range: phases 0 to 3, read from `scripts/sweep-report.sh` and each plan's `## Outcome` and
`## Hurdles`. Phase 4 (PR #14) was still open, so its hurdles were not on `main`. This is the
first sweep. The owner was away, so nothing below was applied: every line is a **proposal
awaiting the owner's yes**, and no `AGENTS.md` changed.

| # | Finding | Phases | Proposal (awaiting the owner) | Where |
| --- | --- | --- | --- | --- |
| 1 | A test passed on its first run because earlier code already did it, three times, each `→ none` and each proved by breaking the code by hand | 1, 2, 3 | Rule: a test that is green on its first run is not red: break the code it covers, show that red run, restore, and say so in the update | root `AGENTS.md`, "TDD is never skipped" |
| 2 | The text guard hook blocked a whole shell command for holding an em dash, so a PR body was never written and a stale one went up instead | 2 | Rule: PR bodies, plans and other prose are written with the Write tool, never through a shell argument or heredoc (already the orchestrator's practice; it recurred once, so a rule rather than a repeat) | root `AGENTS.md`, "Draft PR first" |
| 3 | A shared scratchpad let a stale `body.md` from an earlier phase be uploaded | 2 | Decline as its own rule: the "fetch it back and compare" rule added in phase 2 already catches it | none |
| 4 | Width and layout bugs found by review, not by the plan: questions too long for 80 columns, the title and reason cut at the narrowest sidebar, then counted in characters | 1, 3 | Rule: a plan whose change draws text adds the narrowest sidebar and a CJK name as rows of its state table | root `AGENTS.md`, "Plan before code" |
| 5 | Review rounds found faults in the previous round's own fixes (phase 1, four of six rounds; phase 3, round 2 on round 1's width fix) | 1, 3 | Decline as a rule: "Two strikes, then redesign" already covers it and held in phase 3 | none |
| 6 | One-off tooling dead ends with `→ none` (cargo-audit not installed locally, pty exhaustion when tests run beside the gates, a stale e2e binary in a shared `target/`, a `sed` that rewrote other tests, Codex capacity) | 1, 2, 3 | Decline: no two are the same problem | none |
| 7 | A plan's task order broke because the dead-code rule pulls wiring forward | 3 | Decline: one occurrence; a plan names the caller of each new `pub` item in the same task | none |

Metrics (from `plans/metrics.md`): review rounds 6, 3, 3 for phases 1 to 3; start to release
4 h 13 min, 44 min, 1 h 3 min; Claude reviewer tokens per round 86k to 140k (phase 1 115k to 140k, phase 2 86k to 127k, rising
over its rounds, phase 3 115k to 125k), with no rise from phase to phase; builder
notices per phase 12, 4, 4. Three comparable rows are too few to call a trend in tokens; rounds
dropped from 6 to 3 and held. Codex and orchestrator usage remain unknown in every row.

### Build

- `harbour status` prints the summed counts of every running harbour in the issue's format and
  exits 0 in every state of the table; each row has a test against the real binary
  (`tests/status.rs`), the writer has its own (`tests/summary.rs`), and two e2e tests read a
  harbour in a scratch tmux server, then see nothing once it quits.
- `src/summary.rs` is a new module; `src/app.rs` and `src/tui.rs` changed by a few lines each,
  so the phase 4 branch, which edits both, merges with little to resolve.
- `state::counts` counts once for the sidebar title and the summary, so the two never disagree.
- The App wiring moved from task 5 into task 4: the dead-code rule fails a `pub` writer nothing
  calls. Task 5 kept the e2e tests and the taller help shot (9 rows for the 8-line usage). This
  is the second time (sweep finding 7), so the next sweep may turn that decline into a rule.
- Every Decision above was taken without the owner, as the most conservative reading of issue #8.
- Review round 1 (no must): the lock is now taken without waiting, a failed write stops the old
  counts being counted, and the status folder is set to `0700` and refused when it is a symlink.
- Review round 2 (no must) found four holes in round 1's recovery, which re-locked the old lock
  handle: a folder made again got counts with no lock beside them, a failure retried on every
  draw, two hosts sharing a pid could delete each other's files, and a reader holding a stale
  `<pid>.lock` stopped a new harbour with that pid publishing. Each publication now has a name of
  its own, any failure drops it whole, and a fresh one is made at most every 5 s; a test covers
  each of the four.
- Unique names meant a pair left by a harbour closed with its pane was never reused, so the
  folder would grow without bound. A harbour that makes a publication now sweeps the dead ones
  (exclusive lock free, harbour's own naming, regular files only); `harbour status` still
  removes nothing.

## Hurdles

- `process` The dead-code rule pulled the App wiring from task 5 into task 4, as in phase 3 → none
- `test-seam` Tests green on their first run (the quit e2e test, Summary removal, the reader's liveness) were proved by breaking the code by hand, as in phases 1 to 3 → none
- `test-seam` The first e2e status test raced the registry read: it saw `·` (unknown) before `●` and asserted on any non-empty line; it now waits for the line it asserts, as `tests/AGENTS.md` says → none
- `review-loop` Round 1 found three failure states the table had not listed (a lock held by another host, a write failing after a success, a status folder that already existed or was a symlink) → code
- `design` Round 2 found four more holes in the same recovery mechanism (re-locking an old handle under a pid name); per "Two strikes, then redesign" it was replaced by one publication per instance with a unique name, dropped on any failure and made again at most every 5 s → code
- `design` The redesign's unique names left nothing to reuse a killed harbour's pair, so the folder grew without bound until the orchestrator caught it before round 3; a harbour now sweeps dead publications when it makes one → code
