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
| Neovim, leftmost window, normal mode | `Ctrl-h` | Claude Code |
| Neovim, anywhere else | `Ctrl-h` | Neovim |
| Neovim not answering on its socket | `Ctrl-h` | Claude Code |
| Another editor | `Ctrl-h` | Claude Code, always |
| The editor | `Ctrl-l` | the editor |

## States

| State | What the user sees |
| --- | --- |
| Not started | the pane's top border reads `claude \| nvim` with `claude` marked; the pane shows Claude Code |
| `Ctrl-l` from Claude Code, editor starts | the pane shows the editor, focused; the border reads `claude \| nvim: <directory>` with the editor marked, the home directory shown as `~` |
| Running, shown | keys, paste and the mouse go to the editor; `Ctrl-h` as in the keys table |
| Running, hidden behind Claude Code | the border names its directory, Claude Code marked; the next `Ctrl-l` shows it again with its buffers when Claude Code is still in that directory |
| Claude Code moved to another repository or worktree | `Ctrl-l` starts a second editor there; the first keeps running hidden; moving back and `Ctrl-l` shows the first again |
| The transcript cannot be found or read (not `claude`, no line with `cwd`, unreadable) | the editor opens at the root of the session's start directory |
| Claude Code's directory is gone | nothing starts, focus stays on Claude Code; the bottom of the sidebar says `Not a directory: <path>` |
| Not in a repository | the editor opens in Claude Code's directory itself |
| The editor cannot start (not installed, not executable) | nothing starts, focus stays on Claude Code; the sidebar says `Could not start nvim: is it installed and on PATH?` |
| The editor exits with 0 | the pane shows Claude Code, focused; no message; the next `Ctrl-l` starts a fresh editor |
| The editor exits with an error or a signal | as with 0, and the sidebar says `nvim exited 1` (or `nvim exited (SIGKILL)`) until the next start |
| A hidden editor exits | it is forgotten; an error exit is named the same way |
| Not answering on its socket (busy, or its socket could not be made) | `Ctrl-h` goes to Claude Code after at most the timeout; it is never stuck |
| Switching the selected session with an editor open | the pane shows what the newly selected session last showed; the other session's editor keeps running hidden |
| The session's Claude Code exits while its editor is open | the row says `exited`; the editor keeps running and stays reachable until the row is ended |
| `x` on a running session | `End? y/n`; with unsaved changes in one of its Neovims `Unsaved changes. End? y/n`; yes ends Claude Code and its editors |
| `x` on an exited session whose editors still run | the same question; yes ends the editors and removes the row |
| `q` with any session or editor running | `Quit? y/n`, or `Unsaved changes. Quit? y/n` when a Neovim says so; yes ends all |
| Narrow terminal (under 60 columns) | the focused pane takes the screen as before; the border's directory loses its start first (`…/acme/api`), then the names are cut, all by display width |
| The picker popup open while an editor exists | every key, paste and mouse event goes to the picker; nothing reaches an editor |
| The help view or a question open | as before: the next key closes or answers it; the editor gets nothing |

## Decisions

- **`HARBOUR_EDITOR` names one program, run without a shell and without arguments**, like the
  other two settings; empty counts as unset. It is Neovim, and gets `--listen <socket>`, when its
  file name is `nvim`.
- **Every editor starts with no file argument**, in its directory; what it shows first is the
  editor's own choice (an empty buffer for Neovim).
- **Only the default `claude` is started with `--session-id`.** A program named by
  `HARBOUR_SESSION_COMMAND` might not take the option, so its editor opens at the root of the
  session's start directory. The transcript is found by its file name under every project
  directory, so harbour does not reproduce Claude Code's name for that directory, and its path
  is kept once found.
- **The root is the nearest directory up from Claude Code's that holds `.git`** (a directory, or
  the file of a worktree or submodule), found without running git; none means the directory
  itself. A directory that is gone is not walked up, so `Session::start` refuses it by its own
  path rather than opening a parent repository.
- **The socket lives in a private temporary directory** (0700), removed with the editor.
  Each question opens a fresh connection with a 100 ms timeout on every read and write, so an
  answer that arrives late never lands on the next question. No answer means "go to Claude
  Code" for `Ctrl-h` and "no unsaved changes known" for the questions.
- **The tab strip is in the pane's top border**: `claude | <editor>` before the first editor
  starts for the session, `claude | <editor>: <directory>` with the directory of the editor the
  pane would show; the shown one is bold, the other dim.
- **An editor's exit message reuses the row's wording** (`exited 1`, `exited (SIGKILL)`),
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
9. The demo's editor and the screenshots; end-to-end tests with the fixture editor and with a
   real Neovim where one is installed.

## Outcome

## Hurdles
