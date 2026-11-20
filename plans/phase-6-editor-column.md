# Phase 6: an editor beside each session's Claude Code pane

## Context

Issue #4: while an agent works in a session, the user wants to read and edit the files it works
on without leaving harbour, reached with the keys that already move between the sidebar and the
pane. The pane shows Claude Code or the editor, one at a time, per session. The editor is Neovim
unless `HARBOUR_EDITOR` names another program, opens at the root of the repository or worktree
Claude Code is in at that moment, and belongs to that directory for good: when Claude Code moves
to another worktree, `Ctrl-l` opens a second editor there and the first keeps running, hidden.

## Verified facts (2026-11-20, `main` at fe8aaf5)

- NVIM v0.12.6 on the build machine. A headless `nvim --clean --listen <socket>` answers
  `nvim_eval` over its socket: `mode() ==# 'n' && winnr('h') == winnr()` is 1 in the only window,
  0 after `:vsplit` and `:wincmd l`; `mode()` is `i` in insert mode, and the RPC is answered
  there too. `len(filter(getbufinfo({'bufmodified': 1}), {_, b -> getbufvar(b.bufnr, '&buftype')
  ==# ''}))` is 1 after typing into the empty buffer. Checked with `nvim --server --remote-expr`.
- `claude --help` lists `--session-id <uuid>`: "Use a specific session ID for the conversation".
- Claude Code writes its transcript to `<CLAUDE_CONFIG_DIR or ~/.claude>/projects/<the start
  directory with every non-alphanumeric character as ->/<session id>.jsonl`, one JSON object a
  line. Most lines carry `cwd`, some (`queue-operation`) do not. In the transcripts on the build
  machine, `cwd` moves into a worktree when Claude Code `cd`s there, and the file stays in the
  project directory of the start directory.
- `Session::start` (`src/session.rs`) is the one place that refuses a directory that is gone
  (`Not a directory: <path>`), removes `TMUX` and `TMUX_PANE` from the child's environment, and
  resolves the program on `PATH` itself (a missing one fails with "is it installed and on
  PATH?"); `refusal` in `src/app.rs` turns the error into the sidebar's message.
- `mouse::route` keys a gesture to a session id, so giving every editor an id of its own ends a
  gesture when the pane switches between Claude Code and the editor.
- Review round 1 (2026-11-20): the reviewer's Neovim leaves `nvim_eval` unanswered at a "Press
  ENTER" prompt and with `Ctrl-w` pending; `nvim_get_mode` is api-fast per Neovim's `:help
  api-fast` and answers there with `blocking: true`.
- Review round 2 (2026-11-20), the reviewer's real `nvim --clean` over RPC: with an operator
  pending (`d`, `c`, `y`, `gu`) `nvim_get_mode` answers mode `no` with `blocking: false` and
  `mode()` is `n`; after insert-mode `Ctrl-o` it answers `niI`. `Ctrl-w`, `g`, `z`, `f`, `"`,
  `r`, a count, a partial mapping, and insert-mode `Ctrl-v`, `Ctrl-r` and `Ctrl-k` answer
  `blocking: true`. Rechecked here through harbour by `tests/e2e/neovim.rs` with NVIM v0.12.6.
- A Unix socket whose listener never accepts: once its backlog is full, `connect` blocks on
  Linux (a non-blocking one gives `EAGAIN`) and is refused at once on macOS. Seen in a
  `rust:1.98.1` container and with Python on the build machine.
- Claude Code's registry `<CLAUDE_CONFIG_DIR or ~/.claude>/sessions/<pid>.json`, read on the
  build machine by field name only, no value printed: its files carry `sessionId` and `cwd`
  (beside `pid`, `status`, `updatedAt` and others; some hold only a peer token). In the 4 live
  sessions with a `sessionId`, `cwd` equalled the process's own directory (from `lsof`); in 2 a
  transcript named `<sessionId>.jsonl` existed and its last `cwd` equalled the registry's; in the
  other 2 no transcript had that name yet. **Assumption**, not verified: Claude Code rewrites
  `sessionId` after `/clear` or `/resume` and `cwd` when it changes directory.
- `src/app.rs` is 555 lines; the source-size test allows 700.
- The end-to-end tests run the binary on a scratch `PATH` of links (`tools` in
  `tests/e2e/main.rs`), where nvim is not; the screenshot script runs the binary with the user's
  `PATH`, where it is.

## Keys

| From | Key | Goes to |
| --- | --- | --- |
| Sidebar | `Ctrl-l` or Enter | whatever the pane last showed for that session |
| Claude Code | `Ctrl-l` | the editor of the directory Claude Code is in, started on first use |
| Claude Code | `Ctrl-h` | the sidebar |
| Neovim, mode exactly `n` or `nt`, not blocking, leftmost window | `Ctrl-h` | Claude Code |
| Neovim, anywhere else (another window, any other mode, blocking) | `Ctrl-h` | Neovim |
| Neovim not answering on its socket | `Ctrl-h` | Claude Code |
| Another editor | `Ctrl-h` | Claude Code, always |
| The editor | `Ctrl-l` | the editor |
| Sidebar | `e` | the session's next editor, in the order they started, wrapping, focused; nothing without one |

## Neovim states

One rule settles `Ctrl-h` and the questions: harbour asks `nvim_get_mode` first, and only a
Neovim that answers it with `blocking: false` is asked `nvim_eval`; both share one 100 ms
deadline. `Ctrl-h` leaves Neovim only when the `mode` field is exactly `n` or `nt`, `blocking` is false
and the window is leftmost; every other mode keeps the key in Neovim. A key Neovim is waiting
for goes to Neovim, and an editor harbour cannot ask may have unsaved changes. Measured with a
real Neovim (see Verified facts):

| Neovim state | `Ctrl-h` goes to | `x` / `q` ask |
| --- | --- | --- |
| Normal mode (`n`), leftmost window | Claude Code | `End? y/n`, or `Unsaved changes. End? y/n` |
| Normal mode (`n`), another window | Neovim | as above |
| Insert mode (`i`) | Neovim | as above |
| Operator pending (`d`, `c`, `y`, `gu`: mode `no`, not blocking) | Neovim | as above |
| Insert-mode `Ctrl-o` (mode `niI`, not blocking) | Neovim | as above |
| `Ctrl-w`, `g`, `z`, `f`, `"`, `r`, a count or a partial mapping pending (blocking) | Neovim | `May have unsaved changes. End? y/n` |
| "Press ENTER" prompt (blocking) | Neovim | `May have unsaved changes. End? y/n` |
| Command line | Neovim (blocking, or mode `c`) | `May have unsaved changes. End? y/n` when blocking, else as normal mode |
| Terminal buffer, terminal mode | Neovim (mode `t`) | as normal mode |
| Terminal buffer, normal mode (`nt`, after `Ctrl-\ Ctrl-n`), leftmost window | Claude Code | as normal mode |
| No answer (busy, gone, socket not made) | Claude Code, after at most 100 ms | `May have unsaved changes. End? y/n` |
| Answers its mode, not `nvim_eval` | Claude Code | `May have unsaved changes. End? y/n` |
| Not Neovim (another `HARBOUR_EDITOR`) | Claude Code | `May have unsaved changes. End? y/n` |

`q` reads the same with `Quit?`; the strongest of its editors' answers wins. A modified buffer
counts as unsaved when its `buftype` is empty (a file) or `acwrite` (written through a
`BufWriteCmd` handler, as many plugins do): both are work the user would lose. A terminal,
prompt, `nofile`, `nowrite`, help or quickfix buffer holds nothing to save and does not count.

## Where Claude Code was last seen

Each session holds one directory, the one Claude Code was last seen in, and `Ctrl-l` opens the
editor at its repository root. It is updated at every `Ctrl-l`, and nothing older than what it
holds can overwrite it:

The id it follows is harbour's own `--session-id` until the registry names one; from then on it
is the id the registry named last, and a registry that is gone, half written or names no id keeps
it. A `sessionId` that is not a plain name (letters, digits, `-`, `_`) is followed like any other,
but its transcript is never looked up, so it counts as missing. The registry's `cwd` applies only
while no transcript has ever given a directory. Each cell names the test that pins it (in
`tests/app/following.rs` unless it says `directory.rs`):

| State of the lookup | The followed transcript has a `cwd` | It has none (no line with `cwd`) | It is missing (not written yet, gone, or the id is not a plain name) |
| --- | --- | --- | --- |
| No registry yet (harbour's own id) | that `cwd`: `directory.rs` `the_editor_opens_at_the_root_of_the_directory_the_transcript_names_last` | unchanged, the start directory: `without_a_registry_a_transcript_with_no_directory_leaves_the_start_directory` | unchanged, the start directory: `directory.rs` `without_a_transcript_the_editor_opens_at_the_root_of_the_start_directory` |
| The registry names harbour's own id | that `cwd`, over the registry's: `a_registry_naming_harbours_own_id_follows_its_transcript` | the registry's `cwd` while no transcript gave one: `a_registry_naming_harbours_own_id_gives_its_directory_while_the_transcript_has_none` | the registry's `cwd` while no transcript gave one: `a_registry_naming_harbours_own_id_gives_its_directory_while_the_transcript_is_missing` |
| The registry names another id | that `cwd`: `directory.rs` `after_claude_code_changes_its_session_the_editor_follows_the_transcript_the_registry_names` | the directory found last: `another_id_whose_transcript_has_no_directory_keeps_the_directory_found_last` | the directory found last: `directory.rs` `a_new_session_id_without_a_transcript_keeps_the_directory_claude_code_was_in`; with none found, the registry's `cwd`: `directory.rs` `the_registry_names_the_directory_while_its_session_has_no_transcript`; a name that is not plain: `directory.rs` `a_session_id_that_is_not_a_plain_name_is_not_looked_up` |
| The registry gone or half written after naming an id | that id is still followed and its `cwd` taken: `a_registry_gone_or_half_written_keeps_following_the_id_it_named`, `a_registry_gone_after_naming_an_id_still_follows_that_transcript` | the directory found last, or the registry's last `cwd`: `a_registry_gone_keeps_its_last_directory_while_no_transcript_gave_one` | the directory found last, never harbour's own transcript again: `a_registry_gone_after_naming_an_id_with_no_transcript_never_goes_back_to_harbours_own`; a transcript that disappears: `directory.rs` `a_transcript_that_disappears_leaves_the_directory_last_seen` |
| Claude Code exited | nothing is read (the registry's pid may be reused), unchanged: `after_claude_code_exits_a_new_directory_in_its_transcript_is_not_read` | unchanged, as left | unchanged, as left: `directory.rs` `after_claude_code_exits_the_editor_stays_at_the_directory_last_seen` |

Editors are kept per directory, so one left behind when the directory moves keeps running
hidden. The way back to each is `e` in the sidebar: it shows the session's editors one after the
other, in the order they started, so no editor with unsaved buffers is ever out of reach.

## States

| State | What the user sees |
| --- | --- |
| Not started | the pane's top border reads `claude \| nvim` with `claude` marked; the pane shows Claude Code |
| `Ctrl-l` from Claude Code, editor starts | the pane shows the editor, focused; the border reads `claude \| nvim: <directory>` with the editor marked, the home directory shown as `~` |
| Running, shown | keys, paste and the mouse go to the editor; `Ctrl-h` as in the keys table |
| Running, hidden behind Claude Code | the border names its directory, Claude Code marked; the next `Ctrl-l` shows it again with its buffers when Claude Code is still in that directory |
| Claude Code moved to another repository or worktree | `Ctrl-l` starts a second editor there; the first keeps running hidden; moving back and `Ctrl-l` shows the first again, and `e` in the sidebar shows each in turn |
| Claude Code changed session (`/clear`, `/resume`) | `Ctrl-l` reads the registry's `sessionId` and follows that transcript's `cwd` (see Where Claude Code was last seen) |
| The registry's session has no transcript yet (after `/clear`, before the next prompt), or its transcript is gone | the directory Claude Code was last seen in |
| Claude Code exited after a change of session | the directory it was last seen in; its editors stay reachable |
| A transcript line over 1 MiB (a pasted image) | it is skipped; the directory stays the one found last until the next line with `cwd` |
| The registry's `sessionId` is not a plain name (letters, digits, `-`, `_`) | it is followed, but its transcript is never looked up and counts as missing: the directory found last, or the registry's `cwd` while no transcript gave one (see Where Claude Code was last seen) |
| The registry and both transcripts cannot be read (not `claude`, no line with `cwd`, unreadable) | the editor opens at the root of the session's start directory |
| Claude Code's directory is gone | nothing starts, focus stays on Claude Code; the bottom of the sidebar says `Not a directory: <path>` |
| Not in a repository (a `.git` at the home directory itself does not count, compared by the folder, not its spelling, as for the card) | the editor opens in Claude Code's directory itself |
| The editor cannot start (not installed, not executable) | nothing starts, focus stays on Claude Code; the sidebar says `Could not start nvim: is it installed and on PATH?`, or, when a narrow terminal hides the sidebar, the bottom of the pane says it until the next key |
| The editor exits with 0 | the pane shows Claude Code, focused; no message; the next `Ctrl-l` starts a fresh editor |
| The editor exits with an error or a signal | as with 0, and the sidebar says `nvim exited 1` (or `nvim exited (Killed: 9)` for a signal) until the next start; with the sidebar hidden, the bottom of the pane says it until the next key |
| A hidden editor exits | it is forgotten; an error exit is named the same way |
| Not answering on its socket (busy) | `Ctrl-h` goes to Claude Code after at most the timeout; it is never stuck; `x` and `q` say `May have unsaved changes.` |
| The socket's directory cannot be made (the temporary directory gone or not writable) | Neovim starts without `--listen`; `Ctrl-h` goes to Claude Code; `x` and `q` say `May have unsaved changes.` |
| The temporary directory is too long for a socket path (macOS allows 103 bytes) | the socket goes under `/tmp` instead |
| Switching the selected session with an editor open | the pane shows what the newly selected session last showed; the other session's editor keeps running hidden |
| The session's Claude Code exits while its editor is open | the row says `exited`; the editor keeps running, not dimmed, and stays reachable until the row is ended; only Claude Code's own screen is dimmed |
| `x` on a running session | `End? y/n`; with unsaved changes in one of its Neovims `Unsaved changes. End? y/n`; with an editor harbour cannot ask (see Neovim states) `May have unsaved changes. End? y/n`; yes ends Claude Code and its editors |
| `x` on an exited session whose editors still run | the same question; yes ends the editors and removes the row |
| `q` with any session or editor running | `Quit? y/n`, `Unsaved changes. Quit? y/n` when a Neovim says so, or `May have unsaved changes. Quit? y/n`; yes ends all |
| A question in the narrowest sidebar (20 columns at 60, 26 at 80) | it wraps by words, measured in display columns, at the bottom of the sidebar, and shows whole |
| Narrow terminal (under 60 columns) | the focused pane takes the screen as before; the border's directory loses its start first (`…/acme/api`), then the editor's name is cut, all by display width; once not one column of it fits, the border shows no names |
| A failure message where the sidebar or pane is narrow | it wraps by words, measured in display columns, and a word wider than the line breaks by columns |
| The picker popup open while an editor exists | every key, paste and mouse event goes to the picker; nothing reaches an editor |
| The help view or a question open | as before: the next key closes or answers it; the editor gets nothing |
| The help view at 80x24 or 120x20 | every key and note shows (18 rows) |

## Decisions

- **`HARBOUR_EDITOR` names one program, run without a shell and without arguments**, like the
  other two settings; empty counts as unset. It is Neovim, and gets `--listen <socket>`, when its
  file name is `nvim`.
- **Every editor starts with no file argument**, in its directory; what it shows first is the
  editor's own choice (an empty buffer for Neovim).
- **Only the default `claude` is started with `--session-id`.** A program named by
  `HARBOUR_SESSION_COMMAND` might not take the option, so its editor opens at the root of the
  session's start directory. The transcript is found by its file name under every project
  directory, so harbour does not reproduce Claude Code's name for that directory.
- **A session holds one directory Claude Code was last seen in** (see its table). Rounds 1 to 3
  each found a hole in an order of sources looked up afresh at every `Ctrl-l`; round 4 replaced
  the order with this one value, which only the current session id's transcript moves while
  Claude Code runs, so an older source can never take it back.
- **`e` in the sidebar shows the session's editors in turn**, so an editor left in a directory
  Claude Code is no longer in stays reachable. The help view made room for its row by dropping
  the blank line before the legend.
- **The root is the nearest directory up from Claude Code's that holds `.git`** (a directory, or
  the file of a worktree or submodule), found without running git; none means the directory
  itself. A directory that is gone is not walked up, so `Session::start` refuses it by its own
  path rather than opening a parent repository.
- **The socket lives in a private temporary directory** (0700), removed with the editor, named
  `harbour-nvim` and 6 random characters, under `/tmp` when the temporary directory would make
  its path longer than the 103 bytes macOS allows. When that directory cannot be made, Neovim
  starts without a socket and counts as one harbour cannot ask. Each question opens a fresh connection, and
  `nvim_get_mode` and `nvim_eval` share one 100 ms deadline, so an answer that arrives late
  never lands on the next question. The connect is non-blocking and bounded by the same deadline,
  so a Neovim that stopped accepting cannot hold the key thread; the reply is the response to the
  request id sent, any notification before it is skipped, and at most 1 MiB is read. No answer means "go to Claude Code" for `Ctrl-h` and
  "may have unsaved changes" for the questions (see Neovim states).
- **Questions sit at the bottom of the sidebar, wrapped by words**, bold, above the failure
  message, instead of in the sidebar's bottom border, where the narrowest layouts cut them.
- **The tab strip is in the pane's top border**: `claude | <editor>` before the first editor
  starts for the session, `claude | <editor>: <directory>` with the directory of the editor the
  pane would show; the shown one is bold, the other dim.
- **`Ctrl-h` leaves Neovim only from mode exactly `n` or `nt`, not blocking, in the leftmost window.**
  Round 2 showed `mode()` reads `n` with an operator pending, so the rule reads the `mode` field
  of `nvim_get_mode` itself instead of `mode()`. Round 3 added `nt`, a terminal buffer's normal
  mode, which Neovim calls normal mode too.
- **A failure message shows where the user is looking**: at the bottom of the sidebar, or, when
  a narrow terminal hides it, at the bottom of the pane, where the next key clears it so it does
  not cover Claude Code's input for good.
- **The tab strip shows both names or none**: a lone `claude | ` with no editor name read as a
  broken border.
- **Editors take ids from the counter of rows, but not their numbers**: a session's name counts
  sessions only, so opening an editor does not make the next session `sub 3` instead of `sub 2`.
- **A real Neovim runs in the end-to-end tests of `tests/e2e/neovim.rs` only**, with `--clean`
  and every directory it reads or writes under the scratch `HOME`, and only when `nvim` is on the
  `PATH` the tests run with; without one they pass and say why. Issue #4 asks for them, and they
  are what checks the mode rule against Neovim itself.
- **An editor's exit message reuses the row's wording** (`exited 1`, `exited (Killed: 9)`),
  prefixed with the editor's name; editors harbour ends itself are forgotten without a message.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The plan, the fragment; `Ctrl-l` from Claude Code starts the editor in the session's
   directory and shows it in the focused pane; keys reach it.
2. `Ctrl-h` from an editor goes back to Claude Code; `Ctrl-l` shows the same editor again; the
   sidebar's `Ctrl-l` and Enter go to what the pane last showed; the tab strip.
3. The editor exits (0, an error, a signal) or cannot start; a hidden one exits.
4. Paste and the mouse reach the shown editor; switching sessions; the picker popup.
5. Neovim: `--listen`, and `Ctrl-h` asks it; no answer goes to Claude Code.
6. The directory: the repository root; `claude` with `--session-id`; the transcript's `cwd`; one
   editor per directory.
7. `x` and `q` end the editors; the questions say when a Neovim has unsaved changes.
8. The binary reads `HARBOUR_EDITOR`; the help view, its notes and the README.
9. The demo's editor and the screenshots; end-to-end tests with the fixture editor. The fake
   Neovim of `tests/app/neovim.rs` answers the same RPC in process; review round 2 added
   end-to-end tests with a real, clean Neovim (`tests/e2e/neovim.rs`) where one is installed.

## Outcome

- Every row of the States table has a test: in process (`tests/app/editor*.rs`,
  `tests/app/neovim.rs`, `tests/app/directory.rs`) with `/bin/sh` scripts standing in for the
  editor, Neovim and `claude`, and end to end (`tests/e2e/editor.rs`) with
  `tests/fixtures/editor.sh`, also linked as `nvim`, on the scratch `PATH`.
- New modules: `src/editor.rs` (the program, the editor, its socket), `src/nvim.rs` (`nvim_get_mode`,
  then `nvim_eval`, over msgpack-RPC within one 100 ms deadline), `src/transcript.rs` (Claude Code's `cwd` from its
  transcript), `src/tabs.rs` (the border strip). `src/app.rs` became `src/app/mod.rs` and
  `src/app/draw.rs` to stay under 700 lines.
- Editors are not rows: a session with its editor open is counted once by the sidebar title
  and by `harbour status`, and a test says so.
- `main` was merged in after phase 5 shipped as v0.5.0; the counts are published at the top of
  `App::draw` as on `main`.
- `x` and `q` ask each Neovim they would end about unsaved changes on the key thread, up to
  100 ms each; the known-limit line in `AGENTS.md` names it beside `C-h` and `C-l`.
- The help view had no free row at 20 rows: the editor keys went into the `C-l/Enter` and `C-h`
  rows and one note, and the picker note was shortened, so it still fits in 18 inner rows.
- The demo has a made-up `nvim` (a script printing an `acme` file, no socket); `docs/details.png`
  shows `claude | nvim` in the pane border and `docs/editor.png` shows the editor open.
- Review round 1 fixes: `nvim_get_mode` before `nvim_eval` and "may have unsaved changes" for
  an editor harbour cannot ask; `Ctrl-h` to a blocking Neovim; the directory looked up at every
  `Ctrl-l` through the registry; questions wrapped in the sidebar; only the exited program's
  screen dimmed; the help at 18 rows (the blank line before its notes dropped, the notes
  shortened) fits 80x24; the quitting test returns to the sidebar and checks that the run ended
  before the drop; the socket kept short.
- Review round 2 fixes: `Ctrl-h` leaves Neovim only from mode `n` (an operator pending and
  insert-mode `Ctrl-o` stay); modified `acwrite` buffers count as unsaved; the connect is bounded
  by the deadline (its red seen on Linux only, in a container, since macOS refuses at once); a
  notification before the reply is skipped; opening an editor no longer skips a session number;
  the C-Backspace note no longer says where C-h goes. Ten end-to-end tests run a real
  `nvim --clean` when one is installed (`tests/e2e/neovim.rs`), and pass without one, as in CI.
- Review round 3 fixes: a failure message shows at the bottom of the pane while a narrow
  terminal hides the sidebar, until the next key, wrapped by display columns; `Ctrl-h` leaves
  Neovim from `nt` too, checked against a real Neovim; after `/clear` the editor stays at the
  directory last found instead of the one Claude Code started in; the tab strip shows no names
  once the editor's name cannot show.
- Review round 4 fixes: the directory lookup became one value per session, the directory Claude
  Code was last seen in (its own table above), so an editor opened after `/resume` or `/clear`
  stays the one `Ctrl-l` shows after Claude Code exits; `e` in the sidebar shows each of a
  session's editors in turn; Neovim starts without a socket when its temporary directory cannot
  be made; three tests that started in the checkout folder start in a scratch `acme-api`, and the
  whole `app` target passed in a copy of the tree under a 60-character folder name.
- Review round 5 fixes: the merge of `main` (phase 8's cards) left one transcript reader, main's
  incremental `Transcript`, kept per session for both the card and the editor, and every width,
  cut and wrap goes through `src/text.rs`; the followed id is sticky once the registry has named
  one, and the whole lookup is the state table above, one test per cell; a `.git` at the home
  directory itself no longer makes the home a repository for the editor or the card, which now
  share one walk up (`repo::root` and `repo::place`), comparing the home by device and inode so
  a home reached through a symlink still counts as home.
- When harbour is killed by a signal or its terminal window is closed, each editor's empty
  socket directory stays in the temp directory: declined, a known limit in `AGENTS.md`. A
  transcript line over 1 MiB is skipped, so its `cwd` waits for the next line: declined.
- Every Decision above was taken without the owner, as the most conservative reading of issue #4.

## Hurdles

- `review-loop` Round 1 found that no answer from Neovim was read as "nothing unsaved", and the
  reviewer's own Neovim showed blocking states that leave `nvim_eval` unanswered; the plan had no
  table of Neovim's states, now added → code
- `review-loop` Round 2 found a second hole in the same Neovim rule: an operator pending reports
  mode `no` without blocking and `mode()` reads `n`, so `Ctrl-h` left Neovim mid-operator. The
  table had been written from what the fake Neovim was told to answer; the class went once the
  end-to-end tests asked a real, clean Neovim (`tests/e2e/neovim.rs`), as issue #4 asked → code
- `tooling` The unbounded connect blocks only on Linux; macOS refuses at once, so its test could
  not go red on the build machine and was run red in a `rust:1.98.1` container → none

- `test-seam` The Neovim routing test failed about one full run in five: the fake Neovim wrote
  its reply in pieces and closed, and on macOS setting a read timeout on a socket whose peer
  has closed fails, so `eval` gave up with the whole reply buffered. A real bug, found only by
  tracing a stress loop; the timeout error is now ignored, and tests that need an answer get a
  long patience through the `with_patience` seam → code
- `test-seam` Under load (load average 9 to 16 on the build machine), an in-process test now and
  then ran out its 5 s `wait_for` before a picker or agent started, and `wait_for` returned
  quietly, so one failed later on a missing file with no screen shown; the helper that waits
  for the agent now asserts what it saw. The 5 s deadline itself was left as is → none
- `tooling` The editor shot differed between two runs of `scripts/screenshots.sh`: its HOME was
  a `/tmp` tempdir, macOS gives the session's directory as `/private/tmp/...`, so the border
  showed the random path instead of `~/acme-api`; the script now resolves HOME with `pwd -P`.
  A user whose HOME goes through a symlink sees the full path too → none
- `review-loop` Round 4 found a third hole in the same directory lookup (after an exit following
  `/resume`, harbour's first transcript took back an older directory); two strikes, so the order
  of sources became one value per session that only the current transcript moves, with its own
  table in the plan → code
- `test-seam` Three new tests started their session in the checkout folder and asserted its name
  as drawn, so they failed in a worktree with a long folder name, against the rule in
  `tests/AGENTS.md`; they now start in a scratch folder with a short invented name → code
- `review-loop` Round 5 found a fourth hole in the directory lookup (a failed registry read sent
  it back to harbour's own transcript); its full state machine is now a table of states and
  inputs in the plan, each cell pinned by a test, written before the fix, as the plan rule for
  external state asks → code
- `tooling` The home `.git` end-to-end test failed while the unit tests passed: macOS gives the
  session's directory as `/private/var/...` while `HOME` reads `/var/...`, so a comparison of
  paths missed the home; folders are now compared by device and inode → code
