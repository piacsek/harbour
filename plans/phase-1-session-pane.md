# Phase 1: agent sessions in a sidebar, each with a live Claude Code pane

## Context

harbour becomes a hybrid IDE for agent work: a sidebar of agent sessions on the left third of the
screen, and on the right two thirds a live terminal for the selected session. This phase gives
harbour its first interface. Run on a terminal with no argument, it opens that screen, starts
Claude Code sessions on request, and lets the user move between the sidebar and the session with
`Ctrl-l` and `Ctrl-h`. The editor tab, worktrees, what each session is doing, and sessions that
outlive harbour are later phases (see "Later phases" in `PLAN.md`).

## Verified facts (2026-11-19, `main` at 877b754)

From a throwaway spike that ran the design for real, driven in a scratch tmux server and then by
hand in Ghostty inside tmux 3.7c, with Claude Code 2.1.328. The spike is not in the repository.

- `ratatui` 0.30.2, `crossterm` 0.29.0, `portable-pty` 0.9.0, `vt100` 0.16.2 and `tui-term` 0.3.4
  build together on the pinned toolchain with no `unsafe` in the caller. `tui-term` 0.3.4 needs
  `vt100` 0.16, not the newer 0.18.
- Claude Code draws correctly in a pane parsed by `vt100`: logo, input box, footer, truecolor,
  the spinner and a streamed reply. It runs on the alternate screen and asks for mouse reporting,
  so the wheel reaches it only if harbour forwards mouse events.
- Resizing the outer terminal and forwarding the new size to the PTY and the parser makes Claude
  Code redraw at the new width.
- Keys encoded as plain legacy sequences work in Claude Code: text, Enter, Esc, arrows, `Alt-b`,
  `Ctrl-a`, `Ctrl-e`, `Ctrl-w`, `Ctrl-u`, Shift+Tab, and Alt+Enter for a newline. Bracketed paste
  works.
- Under tmux without extended keys, Shift+Enter and Ctrl+Enter arrive as a plain Enter, and tmux
  3.7c does not answer crossterm's keyboard enhancement query.
- The child stalls unless harbour answers its terminal queries: device attributes, cursor
  position, status, and foreground and background colour.
- Dropping the PTY on quit ended the child, and everything it had started, within 300 ms. No
  process was left behind.
- A child that exits by itself leaves a dead pane. In the spike, focus could not leave it.
- `vt100` gives a skin-tone emoji one cell too many and splits a joined emoji. Other wide text,
  flags and combining accents were right.
- Inside tmux with `vim-tmux-navigator`, tmux keeps `Ctrl-h` and `Ctrl-l` for itself unless the
  pane's foreground process matches the plugin's pattern. With `harbour` added to that pattern,
  both keys reach harbour. Checked with the live binding on the spike's process name.
- `harbour` with no argument prints the usage on stderr and exits 1 today, and `tests/cli.rs`
  runs it without a terminal.

## Decisions

- **No argument on a terminal opens the interface.** When stdin or stdout is not a terminal, no
  argument keeps today's behaviour (usage on stderr, exit 1), so `tests/cli.rs`, the formula's
  test and the flake jobs are untouched. Every existing argument keeps its behaviour.
- **A session is a child of harbour.** It starts in harbour's working directory, lives in a PTY
  harbour owns, and ends when harbour quits. Quitting with a session still running asks first.
  Sessions that outlive harbour need their own spike and are a later phase.
- **`vt100` and `tui-term` for this phase.** The emoji fault is a known limit, not a blocker. A
  second parser is not weighed until a user-visible fault that matters shows up. Its panics are
  contained: a call into `vt100` that panics gives that one session a fresh screen at its size
  with the modes the command had turned on (keypad, cursor keys, bracketed paste, mouse, hidden
  cursor) and asks the command to repaint, and harbour and the other sessions carry on. Replacing
  the parser is in the backlog of `PLAN.md`.
- **Legacy key encoding only.** harbour does not track which keyboard protocol the child asked
  for. Shift+Enter is sent as ESC CR, which Claude Code reads as a newline.
- **`Ctrl-h` always returns to the sidebar from a Claude Code pane.** Claude Code loses it as a
  backspace alias. `Ctrl-l` passes through, since nothing is to the right of the pane. A terminal
  that sends 0x08 for Ctrl+Backspace (xterm's default and those that follow it) arrives as `Ctrl-h`
  too, so Ctrl+Backspace in the pane focuses the sidebar instead of deleting; the help view says so.
- **The mouse is forwarded to the pane** as the child asked for it: what its mode reports
  (presses, releases, the wheel, drags, moves), with Shift, Alt and Ctrl, in its encoding. A gesture
  belongs to where its button went down: one that starts in the pane reaches the session up to its
  release, one that starts anywhere else never does. It also belongs to the session shown when it
  started: showing another session ends it, so no other session gets part of it. Claude Code opens
  a link on Ctrl-click, so the modifiers matter. A click on the sidebar focuses the sidebar.
- **The sidebar shows what harbour knows by itself:** a name (`<directory> <n>`) and whether the
  process is running or has exited. What the agent is doing is a later phase.
- **The tmux pattern is documented in the help view,** not fixed by harbour. harbour never edits
  the user's tmux configuration.
- **A seam replaces Claude Code in tests.** The command a session runs comes from one place that
  tests and the demo can point at a scripted program, listed in `SEAMS` of `tests/dead_code.rs`.
  No test starts the real Claude Code.

## Keys

| Focus | Key | Effect |
| --- | --- | --- |
| Sidebar | `j` / `k` | select the next or previous session |
| Sidebar | `n` | start a session and select it |
| Sidebar | `Ctrl-l` or Enter | focus the pane of the selected session |
| Sidebar | `x` | end the selected session, or remove it when it has exited |
| Sidebar | `?` | show or hide the help |
| Sidebar | `q` | quit, after a yes when a session is running |
| Pane | `Ctrl-h` | focus the sidebar |
| Pane | anything else | sent to the session |

## Session states

| State | Sidebar row | Pane | `Ctrl-l` | `Ctrl-h` | `x` |
| --- | --- | --- | --- | --- | --- |
| No session yet | empty, a hint to press `n` | the same hint | nothing | nothing | nothing |
| The command cannot start (not on `PATH`, not executable) | no row added | the error, naming the command and the cause, at the bottom of the sidebar until a start works; also in the pane when no session is left | as before | as before | as before |
| Running | filled dot, name | the live terminal | focuses the pane | focuses the sidebar | ends it after a yes |
| Exited with 0 | hollow dot, name, "exited" | its last screen, dimmed | focuses the pane | focuses the sidebar | removes the row |
| Exited with another code or a signal | hollow dot, name, "exited" and the code or signal | its last screen, dimmed | focuses the pane | focuses the sidebar | removes the row |
| Exited while its pane has focus | as its exit row | as its exit row | does not apply | focuses the sidebar | does not apply |
| The terminal is too small for both columns | the sidebar alone, or the pane alone when it has focus | as left | focuses the pane | focuses the sidebar | as its row |

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The tmux end-to-end plumbing from conveyor: `#[ignore]` tests that run the real binary in a
   scratch tmux server, `gates.sh` running them with `--include-ignored`, `check` installing tmux.
2. On a terminal, no argument opens an empty screen (sidebar, pane, the hint) and `q` leaves it
   with the terminal restored. Without a terminal, nothing changes.
3. The key encoder: a pure function from a key event to the bytes a terminal sends.
4. A session: a command in a PTY whose output is parsed and drawn in the pane, with the terminal
   queries answered.
5. `n` starts a session, the sidebar lists it, `j` and `k` select, the pane follows the selection.
6. Focus: `Ctrl-l` and Enter into the pane, `Ctrl-h` back, the border shows which side has it,
   and keys reach the session only while the pane has focus.
7. Resizing reaches every session, shown or not, and the narrow layout of the states table.
8. Paste and the mouse reach the session.
9. Exit states: every row of the states table for a session that ended or never started.
10. `x`, and `q` with its question. No process is left after either.
11. The help view with the key table and the tmux pattern, the README usage, and the README
    key-table pin from conveyor.
12. The demo mode, the `details` shot, `docs/details.png` in the README with its pin, and the
    adversarial-review skill's `drive-demo.sh`, all from conveyor.

## Outcome

- All 12 tasks shipped, one commit each (0c84da9 to 8609546). On a terminal, `harbour` with no
  argument opens a sidebar of Claude Code sessions beside a live pane, and `harbour --demo` opens
  the same with made-up `acme/*` sessions. Without a terminal, no argument still prints the usage
  and exits 1, and so does `--demo`.
- Where it differs from the plan as first written:
  - The terminal is narrow under 60 columns: the sidebar shows alone, or the pane alone while it
    has focus.
  - A session's screen is never under 2 rows or 2 columns, since `vt100` panics below that.
  - A click in the pane focuses it, as a click on the sidebar focuses the sidebar.
  - The questions are short (" End? y/n ", " Quit? y/n ") at the bottom of the sidebar. Only a
    plain `y` (Shift allowed) is a yes. After `x` and `y` the row shows as exited and a second
    `x` removes it.
  - The help covers the whole screen and any key or click closes it. It also says that
    Ctrl+Backspace arrives as `Ctrl-h` on most terminals.
  - A command that cannot start is named with its cause at the bottom of the sidebar until a
    start works, and in the pane when no session is left. It can happen while sessions run (the
    binary removed mid-run), which the plan had not foreseen.
  - The usage names `--demo` and says what no argument on a terminal does. `tests/cli.rs` pins it.
- The review forced two redesigns, each after a part had been patched twice ("two strikes, then
  redesign"):
  - `vt100` panics are contained instead of guarded case by case. Every call into `vt100` runs
    under `catch_unwind`; a panic gives that one session a fresh screen at its size, with the
    modes its program had turned on (keypad, cursor keys, bracketed paste, mouse mode and
    encoding, hidden cursor), and asks the program to repaint. The panic hook stays silent for a
    contained panic, and any other panic turns off mouse capture and bracketed paste before the
    terminal is restored. The alternate screen and the title are not carried: harbour draws
    neither.
  - Mouse forwarding is one pure model, `mouse::route`, covered by the table in `tests/mouse.rs`.
    A gesture belongs to where its button went down and to the session shown then; the session
    gets only what its mouse mode asked for, with Shift, Alt and Ctrl, in its encoding.
- Ending a session hangs up its whole process group, then kills it after 200 ms, through `nix`
  (the `signal` feature only, no `unsafe`). Quitting hangs up every session first and waits once.
- Review: 6 rounds, each with both reviewers (Claude and Codex). The last round was unprimed and
  clean. Each round's fixes shipped as one commit, except round 1, whose CI failure needed a
  second.
- Declined for good, now a known limit in `AGENTS.md`: a process that ignores hangup, left behind
  by a session that exited by itself, is not ended. Once the leader is reaped its pid may be
  reused, so harbour never signals that group again.
- Dropped in round 6 as not a fault: with a button held in session A, `Ctrl-h`, `j`, then `k` back
  to A resumes A's gesture. A then receives the release of a button pressed in A, which is the
  right recipient, and no other session receives anything.
- Minor and left as they are (also listed in the pull request body):
  - Showing another session in the middle of a gesture means the first session never gets its
    release.
  - The start-failure text wraps by characters, not display width, which only matters for
    non-ASCII text in the error.
  - Sidebar keys ignore modifiers, so `Ctrl-n`, `Ctrl-x` and `Ctrl-q` act like `n`, `x` and `q`.
- Not verified in this phase: how the real Claude Code behaves when ended (hangup, then kill after
  200 ms), since no test or reviewer may start it; and `Ctrl-h` and Ctrl-click in terminals other
  than the ones driven in tests.
- CI's test step failed twice during review fixes, each with its own cause and fix: a demo race
  the 2x2 size floor exposed (sessions started before the first draw lost their output), and
  dash's `read` returning on a trapped window-change signal.

## Hurdles

- `test-seam` The red run of the dropping-a-session test left a stray process, killed by hand → none
- `test-seam` Several tests went green on their first run because earlier steps had over-implemented (the other answer, `y` quits, exit while focused, the grandchild drop); each was proved by breaking the code → none
- `process` Twice two tests were written in one edit (exit code and signal) → none
- `design` `vt100` panicked four times over the phase (a 0-size pane, a line wrapping on a 1-row screen, a wide character in 1 column, a wide character cut by a resize); the first ones got guards and a size floor, the fourth got containment with `catch_unwind` → code
- `design` A panic in the reader thread poisoned the parser lock and left harbour hung with the terminal in raw mode, and red runs hung the test binary, which had to be run in the background and killed → code
- `tooling` `drive-demo.sh` got a relative binary path while its pane runs in another directory, so the binary was not found → code
- `tooling` zsh does not split `$size` in loops, so the adversarial review's size sweep must run under bash → rule in `.claude/skills/adversarial-review/SKILL.md`
- `test-seam` A pane whose command exits at once lost its output in tmux, and some end-to-end tests raced the screen → code
- `api-shape` The questions were too long for an 80-column sidebar → code
- `design` Grandchildren that ignore hangup survived ending a session; ending the process group needed `nix` rather than `unsafe` → code
- `tooling` cargo-audit is not installed locally, so only CI ran it → none
- `ci` A CI run of an older commit was read as the result of HEAD; runs are now matched to the commit → none
- `test-seam` The large-paste test passed vacuously at first: the tty dropped the paste before `stty raw` took effect → code
- `api-shape` No in-process way triggers a panic, so the hook is tested through a `leave_terminal` seam and a real `vt100` panic through a session → none
- `design` Start failures were assumed unreachable once a session runs, but the binary can disappear mid-run → code
- `test-seam` `--demo` started sessions before the first draw, at a 0x0 pane, so their output was lost; it passed locally and failed on CI → code
- `test-seam` The repaint and wide-character tests passed on macOS but failed on Linux: dash's `read` returns when a trapped window-change signal arrives, and a trap runs only after the foreground command returns → rule in `tests/AGENTS.md`
- `tooling` Running a test binary alongside the gates exhausted macOS ptys (`openpty` error -6) and looked like a test failure → none
- `test-seam` The screen tests could not tell sidebar text from pane text, so a `region` helper asserts on one column range → code
- `tooling` The help screenshot's fixed height silently cut off a longer usage, and nothing checks a shot's size against its text → none
- `design` Mouse forwarding was found faulty three rounds running; it became one pure `route` function with a table test, and binding a gesture to its session then became a type change → code
- `test-seam` Mouse routing was reachable only through `App` and a live session, so each fault needed a slow reproduction; the pure function made each case one line → code
- `review-loop` A fresh parser after a panic looked like a full recovery; only an unprimed round asked what the reset forgets (the modes and the hidden cursor) → code
- `review-loop` Four of the six rounds found faults in code the previous round's fixes had added (the drag fix, the containment, the mouse model), not in the original tasks → none
- `tooling` The Codex reviewer failed once with "Selected model is at capacity" 17 seconds in; a relaunch worked → none
- `tooling` In rounds 1 and 2 the Codex launch put `--sandbox read-only` after the prompt, so it travelled inside the prompt text; the job still ran read-only → none
- `process` CI's test step failed twice across review rounds and the orchestrator carried on because the causes differed, where the stop rule reads "twice on the same step" → none
