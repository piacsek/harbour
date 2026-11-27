# Phase 51: tests that open terminals share a few slots across the machine

Tests under `tests/session/` fail with `failed to openpty: Unknown error -6` when several test
runs share the machine (#219). macOS allows 511 PTYs (`kern.tty.ptmx_max`).

## Verified facts

- **No PTY outlives its test.** With the watch loop `lsof | grep -c ptmx` at a baseline of 32,
  `cargo test --test session` peaks at 99 and returns to 32; with `--test-threads=1` it peaks
  at 35.
- **One run holds about 67 PTYs at its peak,** so about 7 overlapping runs exhaust the pool:
  builders', reviewers' and CI's `cargo test` in different worktrees, which the gates lock does
  not cover.
- **About 1,090 of 2,806 tests open a PTY,** counted by logging each `Session::spawn` and the
  scripts' `on_a_terminal` by test name: about 876 of `tests/app`'s 1,077, 57 of
  `tests/session`'s 70, 2 of `tests/scripts`, 1 of `tests/tui`, and every one of the 152
  `tests/e2e` tests through its scratch tmux server.

## Decided

- The owner's: a machine-wide cap. A fixed number of slot files under
  `$TMPDIR/harbour-pty-slots/`, each taken with an exclusive non-blocking `flock`
  (`File::try_lock`); a test opening PTYs holds one, and waits while all are taken. The kernel
  drops the lock when the holder ends, whatever ends it. nextest's test groups were declined:
  they cap one run, and #219 is many runs at once.
- The builder's: one slot per test thread, kept in a thread-local and freed when the thread
  ends, so a test that builds two screens never waits on itself.

## Tasks

1. Taking a slot locks exactly one slot file (`tests/support/pty_slots.rs`, its own test
   binary `tests/pty_slots.rs`).
2. Other threads get other slots, and one waits while all are taken.
3. One slot per thread, freed when the thread ends, a panic included.
4. `tests/app` screens take a slot.
5. `tests/session` sessions take a slot.
6. The e2e tmux servers, the scripts' and `tests/tui`'s PTYs take a slot.
7. The number of slots chosen by timing `tests/app` at 4, 8 and 16; the trap in
   `tests/AGENTS.md` updated.

## Outcome

## Hurdles
