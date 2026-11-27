# Phase 54: tests that need no terminal run their sessions over pipes

Phase 51 capped PTY use across the machine with 16 slots, so overlapping test runs queue instead
of failing with `failed to openpty: Unknown error -6` (#219). That treats the symptom: most
`tests/app` tests only need a child whose output is drawn and whose input arrives, not a
terminal (#247).

## Verified facts

- **portable-pty already implements its `Child` trait for `std::process::Child`**
  (`portable-pty-0.9.0/src/lib.rs:271`), so a session holds one `Box<dyn Child + Send + Sync>`
  whatever its transport, and only the PTY master is optional.
- **A piped child keeps the controlling terminal of the test run.** portable-pty calls `setsid`
  and `TIOCSCTTY` for a PTY child; `std::process::Command::setsid` is unstable and
  `unsafe_code` is forbidden, so a pipe child only gets `process_group(0)`. Probed under a real
  terminal (`script -q /dev/null cargo test ...`): the child opened `/dev/tty`, a write to it
  reached the terminal, and a read stopped the process group (`T`, `SIGTTIN`, since it is a
  background group), so it cannot read keystrokes but hangs. In CI there is no terminal and the
  open fails.
- **A pipe has no output processing,** so a bare `\n` would leave the cursor in its column; the
  pipe reader writes each `\n` as `\r\n`, as a terminal's default `ONLCR` does.

## Decided

- The owner's: pipes by default in `tests/app`. Every `Screen` runs over pipes and holds no slot;
  a test that needs a terminal calls `needs_a_terminal()` first and gets a PTY and a slot.
- The owner's: sessions reuse portable-pty's `Child` for both transports instead of a `Link`
  enum; `master: Option<Box<dyn MasterPty + Send>>` is `None` over pipes.
- The orchestrator's: a test whose script runs `stty raw -echo` moves to pipes when it passes,
  since that command only turns a terminal's line discipline off and a pipe has none. A test
  whose script reads `stty size`, traps `WINCH`, checks `[ -t` or opens `/dev/tty` stays on a
  PTY even when it passes.
- The orchestrator's: the fake fzf (`tests/fixtures/finder/fzf`) exits 2 with
  `acme fzf needs a terminal` when its stderr is not one, so a finder test left on pipes fails at
  once instead of hanging on `/dev/tty`.

## Tasks

1. `Transport::Pipes` proven in `tests/session/pipes.rs`: output, stderr, `\n` as `\r\n`, input,
   exit status, pid, ending the process group, drained, a missing directory, a missing program.
2. `App::with_transport` reaches the agent, a resume, the usage popup, the picker, the finder
   and the editor; each child in `tests/app/piped.rs` says it has no terminal.
3. `tests/app` screens pipe by default; the tests that fail over pipes, or read the terminal,
   call `needs_a_terminal()`.
4. The PTY peak and the run time measured before and after; `SLOTS` reconsidered.
5. The traps in `tests/AGENTS.md`, `src/AGENTS.md` for `session.rs` and `app/start.rs`.

## Outcome

- **The PTY peak of one run fell by about two fifths.** Counting only the PTYs the test
  binaries hold (`lsof /dev/ptmx`, sampled about every half second), three full `cargo test`
  runs each peaked at 182, 102 and 110 on `main` (median 110) and at 95, 62 and 58 on this
  branch (median 62); `cargo test --test app` alone at 94 and 57, `tests/session` at 36 both
  times. The run time did not get worse: medians 183 s on `main` and 166 s here. One sample
  swings by tens of PTYs, so a single run proves little. The first count, of every PTY on the
  machine, swung from 60 to 170 between two runs of the same tree, since the owner's tmux,
  harbour and Neovim hold PTYs too.
- **179 of `tests/app`'s 1,104 tests stay on a PTY,** plus the two that prove the switch:
  159 failed over pipes (Enter ending `read -r`, echo, `stty size`, `WINCH`), 9 more run the
  fake fzf, which reads `/dev/tty` (2 failed in one of two runs, 7 passed only because they
  never reached that read), and 11 of `tests/app/side.rs` passed but start programs that trap
  `WINCH` and read `stty size`. The other 923 open no PTY.
- **`SLOTS` stays at 16.** The slots cap concurrent PTY tests machine-wide; 16 still keeps
  `tests/app` near its old speed, and the tests left on a PTY open several each (an agent, an
  editor, a picker).
- **A pipe session hangs up its group when it goes away, as a terminal does.** Review round 2
  found that a test run killed by Ctrl-C or a timeout left every piped child running, some
  spinning on `read -r line || continue` at end of input. Each pipe session now starts a watcher
  in a process group of its own that reads a pipe only the session holds; when the session is
  dropped or the test process ends, the watcher hangs up the session's group and kills it after
  `HANGUP_GRACE`. The watcher starts before the child and learns its group over that pipe, so
  no step that can fail comes after the child is running (round 3); killing the test binary with
  SIGKILL in the middle of `tests/app` left no child behind. A watcher inside the group could not do it: on macOS its `kill -HUP 0` ended
  the watcher but not the rest of the group, and dash's `kill` takes no `--`.
- `src/app/mod.rs` crossed the 700-line limit with the transport; `start`, `start_with` and
  `start_in` moved unchanged to `src/app/start.rs`.

## Hurdles

- `tooling` Counting every PTY on the machine gave numbers that swung by 110 between two runs of
  the same tree; only the test binaries' own PTYs gave a stable count → none
- `test-seam` A piped child keeps the test run's controlling terminal and the safe `setsid` is
  unstable, so a script reading `/dev/tty` freezes on a developer's terminal and fails in CI →
  rule in `tests/AGENTS.md`
- `review-loop` Moving children off a PTY also moved them off the terminal's hangup, so an
  interrupted test run left them behind; found in round 2, fixed with a watcher → code
- `process` The step-2 commit was pushed with `src/app/mod.rs` over the 700-line limit, since the
  size test ran before the last edit of the wiring → code
