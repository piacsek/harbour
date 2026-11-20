# Phase 12: open a file in a session's editor from outside harbour

## Context

Issue #12. A script, another terminal or a tool that knows a file path has no way to show that
file in harbour: the user has to find the session, open its editor and type `:edit`. `harbour
edit <file>` hands the file to the running harbour whose session holds it (the file is in that
session's repository or worktree), which selects the session and shows its editor with the file
open and the keys in it. The editor must be Neovim: the file goes over the RPC socket harbour
already gives each Neovim editor (phase 6), or on the command line of a Neovim it starts.

## Verified facts (2026-11-20, `main` at e349092, NVIM v0.12.6, macOS)

Checked in scratch setups only: `nvim --clean --headless --listen <scratch socket>` with a
scratch `HOME` and `XDG_*`, driven by a second `nvim --clean -l` acting as an RPC client, and
Unix sockets in a scratch `mktemp -d`.

- `nvim_cmd({cmd = "edit", args = [path], magic = {file = false, bar = false}}, {})` opens the
  path as one literal file name: a name holding a space, `%`, `#`, `|`, `'`, `"`, a newline, or
  starting with `+` opened that exact file (`expand("%:p")` equal to the path sent), and a
  missing file opened as a new buffer with that name.
- Without `magic`, the defaults are not literal: `%` and `#` were expanded to the current and
  alternate file names. `magic` has to be sent with both fields `false`.
- macOS (APFS) refuses to create a file whose name is not UTF-8 ("illegal byte sequence"), so no
  such file can exist there; sent anyway, Neovim kept the bytes as the buffer name and failed
  with E303 (swap file), returned as an RPC error.
- `nvim_cmd` works in every mode where `nvim_get_mode` says `blocking: false`: normal, insert
  (stays in insert), replace, visual block, operator pending, command line (the command line
  stays open). Harbour already refuses to talk to a blocking Neovim (phase 6).
- With `hidden` off and a modified buffer, `nvim_cmd` returns an error,
  `Vim:E37: No write since last change (add ! to override)`, and nothing changes. `--clean` and
  the defaults have `hidden` on, so the edit opens and the modified buffer stays hidden.
- A Unix socket bound at one name, `chmod` 0600 and renamed keeps accepting at the new name; the
  file mode reads `0o140600`. Once its file is removed, a connect gives `ENOENT`. A socket file
  left by a closed listener gives `ECONNREFUSED`.
- `AF_UNIX` paths are limited to 104 bytes on macOS, 108 on Linux; Python's bind refused a
  scratch path over that ("AF_UNIX path too long"). The macOS `TMPDIR` is 49 bytes, so a
  `tempfile::tempdir()` home plus `.local/state/harbour/edit/` is about 86 bytes before the
  socket's name: names must stay short.
- `nix` 0.28 has `getuid` and `getpeereid` behind its `user` feature; harbour enables `signal`
  and `socket` only, so this phase adds `user` (no new crate).

## How `harbour edit` reaches a running harbour

Each running harbour (not `--demo`) listens on a Unix socket of its own in
`$XDG_STATE_HOME/harbour/edit/` (an absolute `XDG_STATE_HOME`, else an absolute
`HOME` + `.local/state`, else nowhere: the same rule as phase 5's `status/`, a folder of its
own so phase 10's state files are never touched). A request is one connection: the sender
writes it, shuts its side, reads one reply line and closes.

- **The socket.** Its name is `<pid>-<n>.sock`, `n` counting the binds of this process (short,
  see Verified facts). Harbour binds it as `<pid>-<n>.new`, sets it to 0600 and renames it to
  `.sock`, so a sender never sees a live name refuse. Before binding, harbour sweeps every
  `<digits>-<digits>.sock` or `.new` entry of the folder that is a socket (not a symlink, not a
  regular file) and refuses a connect (`ECONNREFUSED`): a dead harbour's. Every 5 s it checks
  that its own path is still the inode it bound (`symlink_metadata`, `ino`), and binds a fresh
  name when not (removed by hand, or swept while busy). On exit it removes its own file.
- **The request.** `ask` or `open`, a space, the time it expires in Unix milliseconds, a NUL,
  then the absolute path as bytes. At most 4096 bytes of path; harbour reads at most 4200 bytes
  and gives each connection 100 ms to arrive whole, else drops it unanswered.
- **The replies.** To `ask`: `holds <millis>` (the last use of the session that would get the
  file) or `none`. To `open`: `opened` or `refused <message>`. One line each.
- **The sender** resolves the path (relative to its own directory, symlinks resolved; a missing
  file is allowed when its folder exists; a directory is refused), checks the folder, sends
  `ask` to every socket of harbour's naming in it at once with one 2 s deadline, picks the
  harbour whose answer is the latest `holds`, sends it `open` with a 5 s deadline, and exits 0
  on `opened`. Everything else prints one line to stderr and exits 1.

| Property | How |
| --- | --- |
| Private | The folder is created 0700 and set to 0700, checked with `symlink_metadata` to be a real directory owned by the user (`getuid`); the socket is 0600. The sender refuses a folder that is a symlink, not owned by the user, or open to group or others, and connects only to entries whose own type is a socket. Never in `/tmp` or another world-writable place: when the state folder is missing there is no channel. |
| Only harbour's naming | Harbour sweeps and the sender connects only to `<digits>-<digits>.sock` (and harbour to `.new`) entries; anything else in the folder is never read, connected to or removed. Harbour removes only its own file on exit, and others' only after a refused connect. |
| No other user, no planted file | Only the user can create entries in a 0700 folder they own; on each accepted connection harbour also checks `getpeereid` and drops a peer of another uid unanswered. A planted regular file or symlink is not a socket and is skipped. |
| Data, never code | The path travels as bytes after a NUL, at most 4096 bytes, never decoded as UTF-8. Harbour sends it to Neovim as a msgpack binary in `nvim_cmd({cmd = "edit", args = [path], magic = {file = false, bar = false}}, {})`: no command string is built, `%` and `#` are not expanded (`magic.file`), `\|` does not end the command (`magic.bar`), a space, quote or newline is part of the name (one argument), and a leading `+` or `-` cannot occur because the path is absolute. A Neovim harbour starts gets it after `--` on its command line, also as one argument. |
| The sender never hangs | Non-blocking connects; one deadline for all the `ask`s (2 s) and one for the `open` (5 s), covering connect, write and read. On a deadline: a message and exit 1. |
| Stale requests swept, never replayed | A request lives only in its connection: nothing is written to disk, so nothing is left to replay when a sender dies. The expiry in the request is the sender's deadline: harbour answers an expired request with nothing and does not act on it, so a sender that gave up never has its file opened later. Dead harbours' sockets are swept as above. |
| The draw and key thread | Harbour accepts without blocking at each frame, at most 4 connections a frame, with 100 ms each to read. An `open` to a running Neovim makes two RPC calls (`nvim_get_mode`, `nvim_cmd`) under one 1 s deadline on that thread. Both are Known limits lines. |

## Which session gets the file

- A session holds a path when the path is inside one of its directories: the root of its
  repository or worktree (`editor::root` of where it works, as Ctrl-l opens), or the directory
  of one of its editors. All compared after resolving symlinks, on both sides.
- In one harbour: the selected session when it holds the path, else the holder used last. A
  session's last use is the last key, paste or mouse event harbour handled while it was
  selected; it starts at its start. Exited sessions count.
- Across harbours: each answers `ask` with its own pick and that pick's last use; the latest
  wins. A tie goes to the first answer.
- In the session: the editor whose directory is the deepest that holds the path; when none
  does, the editor at the session's root, started (Neovim with `--listen` and `-- <path>`) when
  not running, exactly as Ctrl-l would.

## States and what the user sees

`harbour edit` exits 0 with no output only on success; every other row prints
`harbour: <message>` on stderr and exits 1.

| State | In harbour | `harbour edit` prints | Exit |
| --- | --- | --- | --- |
| Neovim editor running in the session that holds the file | the session is selected, its editor shown and focused, the file open in it | nothing | 0 |
| Editor not running in that session | Neovim starts at the session's root with the file, shown and focused | nothing | 0 |
| The file does not exist, its folder does | the same, as a new buffer | nothing | 0 |
| Two sessions in one harbour hold it, one selected | the selected one gets it | nothing | 0 |
| Two sessions hold it, neither selected | the one used last gets it | nothing | 0 |
| Two harbours hold it | the harbour whose holder was used last gets it; the other is unchanged | nothing | 0 |
| No session holds it | unchanged | `no harbour session holds <path>` | 1 |
| No harbour running (no folder, an empty one, only dead sockets) | (none) | `no harbour is running` | 1 |
| `HARBOUR_EDITOR` is not Neovim | unchanged | `a file can only be handed to Neovim, and HARBOUR_EDITOR is <name>` | 1 |
| Neovim is waiting for a key (`blocking`) | unchanged | `Neovim is waiting for a key; answer it and try again` | 1 |
| Neovim does not answer (no socket, or no reply in 1 s) | unchanged | `Neovim did not answer` | 1 |
| Neovim refuses (E37 with `hidden` off, a swap file) | unchanged; Neovim shows nothing new | `Neovim refused: <its error>` | 1 |
| Neovim cannot start | the pane names the failure as Ctrl-l does | `could not start <name>: <error>` | 1 |
| A question (End?/Quit?) or the directory picker is open | unchanged | `harbour is waiting for an answer; try again after it` | 1 |
| The help view is open | it closes, then as the first row | nothing | 0 |
| Harbour busy past the deadline | the expired request is dropped, nothing opens | `no answer from harbour` | 1 |
| The path is a directory | (no request) | `<path> is a directory` | 1 |
| Its folder does not exist | (no request) | `<path>: No such file or directory` | 1 |
| Path longer than 4096 bytes | (no request) | `the path is longer than 4096 bytes` | 1 |
| The state folder is not private (symlink, other owner, group or other bits) | (no request) | `<folder> is not private; refusing to use it` | 1 |
| The socket path would not fit an `AF_UNIX` address | harbour does not listen | `no harbour is running` (Known limits line) | 1 |
| No `XDG_STATE_HOME` or `HOME` | harbour does not listen | `no harbour is running` | 1 |
| `harbour edit` with no file or more than one | (no request) | the usage | 1 |
| `harbour --demo` | never listens | `no harbour is running` when it is the only one | 1 |

## Decisions

Product decisions the issue did not settle, each taken the most conservative way.

- Only Neovim: with any other `HARBOUR_EDITOR`, `harbour edit` is refused even when no editor
  runs yet, so a file is never opened by a program that cannot be checked or told.
- A question or the picker open refuses the request instead of dismissing it: the user's
  answer is never chosen for them. The help view only shows text, so it closes.
- No line or column argument: the issue asks for a file only.
- Exited sessions still count as holding their directory; the editor opens beside them as
  Ctrl-l does.
- The demo never listens, so `harbour edit` never reaches made-up sessions.
- Peer uid checked on every connection though the folder is private: cheap, and it makes the
  socket safe even if the folder's mode is changed by hand.

## Tasks

1. The plan, its rows in `PLAN.md` and `plans/metrics.md` (with phase 11's backfill), the
   fragment.
2. `harbour edit` in the usage, `--help`, the README usage and `tests/cli.rs`; the path
   resolution and its refusals (directory, missing folder, too long, wrong arguments).
3. The channel: harbour's listener (folder, bind and rename, sweep, self-check, removal), the
   sender's `ask` and `open` with their deadlines, the expiry, the peer check.
4. The app answers `ask` and `open`: which session holds the file, last use, refusals.
5. Neovim gets the file: `nvim_cmd` with `magic` off to a running one, `-- <path>` to a new one;
   refusals from Neovim; the fixture Neovim in-process, a real `nvim --clean` end to end.
6. The terminal interface listens; end-to-end tests run `harbour edit` against harbour in a
   scratch tmux server.
7. Guidelines: the layout lines and Known limits in `AGENTS.md`, the README checked, the
   screenshots.

## Outcome

## Hurdles
