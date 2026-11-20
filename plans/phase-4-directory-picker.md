# Phase 4: choose the directory a new session starts in, by fuzzy search

## Context

Every session starts where harbour was started. Issue #7 asks for a directory picker on `n`:
harbour does not draw one, it runs a picker command in a popup, in a terminal of its own, and
starts the session in the directory the command prints. The default is fzf, fed by harbour with
the directory harbour was opened from first and the list zoxide ranks below it.
`HARBOUR_DIR_PICKER` names another command. Once the directory is confirmed, the new session's
pane takes the focus, so the user can type to it at once (added by the owner during the phase).

## Verified facts (2026-11-20, `main` at 7ce71e3)

- fzf 0.74.0 and zoxide 0.10.0 on the build machine. fzf run as `fzf <list >out` in a terminal
  draws its interface on that terminal (it opens `/dev/tty`), and with an empty query shows the
  lines in input order with the first one selected: Enter writes the first line to `out` and
  exits 0, Esc writes nothing and exits 130. Checked in a scratch tmux server.
- `zoxide query --list` prints one absolute directory per line, ranked; it leaves out
  directories that no longer exist unless `--all` is given. A deleted directory can still reach
  the picker through `HARBOUR_DIR_PICKER` or a race, so harbour checks what the picker prints.
- `Session::start` (`src/session.rs`) starts its command in `std::env::current_dir()` through
  `portable_pty::CommandBuilder::cwd`; the PTY is the command's stdin, stdout and stderr, so a
  picker's standard output can only be told apart from its drawing by redirecting it, which
  harbour does with `/bin/sh -c 'exec "$@" >"$out"'`.
- `portable_pty` resolves the program on `PATH` itself and fails with "No viable candidates"
  when it is missing; behind the `/bin/sh` wrapper that check is lost, so harbour resolves the
  picker's program on `PATH` before starting it.
- A session's name is `"{directory} {n}"`, where `directory` is the folder name of harbour's own
  working directory (`App::new` in `src/app.rs`).
- The help view and the README key table both come from `KEYS` in `src/help.rs`;
  `tests/readme.rs` pins the table to it.
- The end-to-end tests and the screenshot script run the binary with the user's `PATH`, so with
  fzf installed, `n` would open the real fzf over the real zoxide database. Both must run the
  binary on a `PATH` that has neither.
- `tempfile` 3 is already in `Cargo.lock` as a dev-dependency.

## States

| State | What the user sees |
| --- | --- |
| `n`, Enter, with fzf | the popup lists harbour's directory first and selected; a session starts there, named after it, and its pane has focus |
| `n`, type, Enter | a session starts in the directory fzf found, named after that directory, and its pane has focus |
| `n`, Esc | the popup closes, nothing is started, focus stays where it was |
| fzf installed, zoxide not | the popup lists only harbour's directory |
| zoxide lists harbour's directory too | it shows once, first |
| fzf not installed and `HARBOUR_DIR_PICKER` unset | `n` starts the session in harbour's directory at once, as before; focus stays on the sidebar |
| the picker prints nothing, or exits with an error | the popup closes and nothing is started |
| the picker prints a path that is not a directory (missing, deleted, a file) | nothing is started; the bottom of the sidebar says `Not a directory: <path>` |
| the picker prints a relative path | it is read from harbour's directory, where the picker ran |
| `HARBOUR_DIR_PICKER` names a program that cannot start | nothing is started; the bottom of the sidebar says `Could not start <program>: <cause>` |
| while the popup is open | every key, paste and mouse event goes to the picker; nothing reaches the sidebar or a session |

## Decisions

- **`HARBOUR_DIR_PICKER` names one program, run without a shell and without arguments**, like
  `HARBOUR_SESSION_COMMAND`. A command line with arguments goes in a script.
- **Only the first line the picker prints counts.** The rest is ignored, read as bytes, so a
  path that is not UTF-8 keeps its bytes.
- **A picker that exits with 126 or 127 could not start**, which the sidebar names; any other
  failure is a cancel.
- **Focus moves to the pane only when the picker confirmed a directory.** When fzf is missing and
  `n` starts the session at once, the sidebar keeps the focus, as before this phase: the owner's
  addition names the picker, and the end-to-end tests drive `n` twice in a row from the sidebar.
- **harbour's own directory is filtered out of zoxide's list** rather than passed as
  `--exclude`, so any list command can stand in for zoxide in tests.
- **The list is made inside the popup, never on harbour's thread** (review round 1). The popup's
  shell starts the list command in the background, writing harbour's directory and then the
  command's lines without it into a FIFO, and runs fzf on it. The popup opens at once and Esc
  always works, however slow zoxide is; a timeout was rejected because it only narrows the
  class. Paths and the command reach the shell as positional arguments, never spliced into the
  script.
- **The demo's picker is a script piping a fixed list of made-up `acme` directories into fzf**,
  never zoxide and without harbour's directory, so `docs/picker.png` is the same on every run.
  The screenshot script runs it with `FZF_DEFAULT_OPTS` unset.
- **`tempfile` becomes a runtime dependency**: the FIFO fzf reads and the file the picker writes
  live in a private temporary directory removed when the pick ends. It is already locked as a
  dev-dependency, and no smaller way creates a unique private directory.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The plan, the fragment, and a session that starts in a directory it is given.
2. A popup runs the picker; a directory it prints starts a session there, named after it.
3. A picker that prints nothing or exits with an error starts nothing.
4. A path that is not a directory, and a picker that cannot start, are named in the sidebar.
5. The default: fzf fed with harbour's directory first and zoxide's list without it; no zoxide;
   no fzf starts at once.
6. The confirmed directory's session takes the focus; a cancel leaves it where it was.
7. Paste and the mouse reach the picker while the popup is open.
8. The binary reads `HARBOUR_DIR_PICKER`; the help view and the README say what `n` does.
9. The demo's picker and the `picker` screenshot.

## Outcome

- `n` opens a popup that runs the picker as a session of its own; its standard output goes to a
  file in a private temporary directory, and the first line it prints is the directory. Keys,
  paste and the mouse go to the picker while the popup is open.
- The default picker is fzf over harbour's directory (first, selected) and `zoxide query
  --list` without it. Without zoxide the list is harbour's directory alone; without fzf, `n`
  starts at once in harbour's directory, as before this phase.
- A session is named after the folder it starts in, so rows of one harbour can differ.
- After a confirmed directory the pane has the focus; a cancel, or a directory a session could
  not start in, leaves it where it was. A session's `PWD` is its directory.
- `HARBOUR_DIR_PICKER` names another picker, run with no shell and no arguments; empty counts as
  unset.
- The end-to-end tests clear both settings and run the binary on a scratch `PATH` of links to
  the tools the fixtures need, so the real fzf never opens;
  `tests/fixtures/picker.sh` stands in for a picker and `tests/fixtures/fzf/fzf` for fzf in the
  demo test.
- The demo's picker is fzf over made-up `/home/acme/...` directories; `docs/picker.png` shows it,
  byte-identical across runs of the screenshot script.

## Hurdles

- `test-seam` A fixture picker that drew on stdout had its drawing taken as the directory it picked; picker fixtures draw on stderr → rule in `tests/AGENTS.md`
- `test-seam` The end-to-end tests and the screenshot script ran the binary on the user's `PATH`, where `n` would now open the real fzf over the real zoxide database → rule in `tests/AGENTS.md`
- `api-shape` `App` implements `Drop`, so `with_picker` could not be built with struct update syntax over `App::new` → none
- `design` Review round 1 found zoxide run on harbour's thread with no bound, so a slow list froze the interface; the list moved into the popup's own shell → code
- `tooling` On macOS `/bin/sh` exits 1, not 126 or 127, when `exec` of a script with a missing interpreter fails; the picker wrappers run the program without `exec` → code
