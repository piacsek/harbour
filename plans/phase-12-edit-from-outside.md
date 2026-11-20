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
  and `socket` only, so this phase adds `user` (no new crate). `getpeereid` exists only on
  macOS and the BSDs; Linux reads the peer through `getsockopt` with `PeerCredentials`.

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
- **The listener** runs on a thread of its own: it accepts without blocking, checks the peer,
  and reads each connection on a short thread of its own, at most 16 in flight (a 17th is
  closed at once). It resolves the path again itself (`..`, symlinks, a dangling symlink
  followed), answers `none` to an `ask` and a refusal to an `open` that does not resolve, and
  hands the rest to the draw loop over a channel with a deadline: the request's expiry, 5 s at
  most. The draw loop takes up to 4 a frame without blocking and drops one past its deadline.
- **The sender** resolves the path (relative to its own directory, symlinks resolved, a
  dangling symlink followed to where it points, at most 40 links; a missing file is allowed when
  its folder exists; a directory is refused), checks the folder, sends
  `ask` to every socket of harbour's naming in it at once with one 2 s deadline, picks the
  harbour whose answer is the latest `holds`, sends it `open` with a 5 s deadline, and exits 0
  on `opened`. Everything else prints one line to stderr and exits 1.

| Property | How |
| --- | --- |
| Private | The folder is created 0700 and set to 0700, checked with `symlink_metadata` to be a real directory owned by the user (`getuid`); the socket is 0600. The sender refuses a folder that is a symlink, not owned by the user, or open to group or others, and connects only to entries whose own type is a socket. Never in `/tmp` or another world-writable place: when the state folder is missing there is no channel. |
| Only harbour's naming | Harbour sweeps and the sender connects only to `<digits>-<digits>.sock` (and harbour to `.new`) entries; anything else in the folder is never read, connected to or removed. Harbour removes only its own file on exit, and others' only after a refused connect. |
| No other user, no planted file | Only the user can create entries in a 0700 folder they own; on each accepted connection harbour also checks the peer's uid (`getpeereid` on macOS, `SO_PEERCRED` on Linux) and drops a peer of another uid unanswered. A planted regular file or symlink is not a socket and is skipped. |
| Data, never code | The path travels as bytes after a NUL, at most 4096 bytes, never decoded as UTF-8. Harbour sends it to Neovim as a msgpack binary in `nvim_cmd({cmd = "edit", args = [path], magic = {file = false, bar = false}}, {})`: no command string is built, `%` and `#` are not expanded (`magic.file`), `\|` does not end the command (`magic.bar`), a space, quote or newline is part of the name (one argument), and a leading `+` or `-` cannot occur because the path is absolute. A Neovim harbour starts gets it after `--` on its command line, also as one argument. |
| The sender never hangs | Non-blocking connects; one deadline for all the `ask`s (2 s) and one for the `open` (5 s), covering connect, write and read. On a deadline: a message and exit 1. |
| Stale requests swept, never replayed | A request lives only in its connection: nothing is written to disk, so nothing is left to replay when a sender dies. The expiry in the request is the sender's deadline: harbour answers an expired request with nothing and does not act on it, so a sender that gave up never has its file opened later. Dead harbours' sockets are swept as above. |
| The draw and key thread | The listener's threads accept, check and read; a flood of silent connections never reaches the draw thread. The draw loop answers at most 4 requests a frame. An `open` to a running Neovim makes its RPC calls (`nvim_get_mode`, retried every 10 ms while the socket is not up, then `nvim_cmd`) on that thread under the request's deadline and 1 s at most: a Known limits line. |
| One deadline to the end | The request's deadline reaches the last step: an open past it starts no editor and sends Neovim nothing. Once `nvim_cmd` has been sent, its outcome may still land after the sender gave up (a swap prompt answered after "Neovim did not answer" still switches the buffer): a Known limits line. |

## Which session gets the file

- A session holds a path when the path is inside the root of its repository or worktree
  (`editor::root` of where it works, the folder Ctrl-l opens Neovim in), compared after
  resolving symlinks on both sides: `harbour edit` sends the resolved path, harbour resolves the
  root.
- In one harbour: the selected session when it holds the path, else the holder used last. A
  session's last use is the last key, paste or mouse event harbour handled while it was
  selected; it starts at its start. Exited sessions count.
- Across harbours: each answers `ask` with its own pick and that pick's last use; the latest
  wins. A tie goes to the first answer.
- In the session: the editor running at the session's root, started (Neovim with `--listen`
  and `-- <path>`) when not running, exactly as Ctrl-l would. An editor left at an earlier root
  after the session moved is not chosen, as Ctrl-l does not choose it either.

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
| Only a stopped card (a remembered session not resumed) holds it | unchanged, nothing resumed | `no harbour session holds <path>` | 1 |
| A stopped card is selected and a running session also holds it | the running session gets it | nothing | 0 |
| No harbour running (no folder, an empty one, only dead sockets) | (none) | `no harbour is running` | 1 |
| `HARBOUR_EDITOR` is not Neovim | unchanged | `a file can only be handed to Neovim, and HARBOUR_EDITOR is <name>` | 1 |
| Neovim is waiting for a key (`blocking`) | unchanged | `Neovim is waiting for a key; answer it and try again` | 1 |
| Neovim does not answer (no socket, or no reply in 1 s) | unchanged | `Neovim did not answer` | 1 |
| Neovim refuses (E37 with `hidden` off, a swap file) | unchanged; Neovim shows nothing new | `Neovim refused: <its error>` | 1 |
| Neovim cannot start | unchanged | `Could not start <name>: <error>` | 1 |
| Neovim's socket not up yet (two opens back to back) | the second waits for it, up to the deadline, then as the first row | nothing | 0 |
| A question (End?/Quit?) or the directory picker is open | unchanged; it answers the `ask` `none` | `no harbour session holds <path>` (another harbour that holds it gets it) | 1 |
| A question or the picker opens between the `ask` and the `open` | unchanged | `harbour is waiting for an answer; try again after it` | 1 |
| The request reaches the open past its deadline | unchanged, no editor started | `harbour took too long` | 1 |
| The help view is open | it closes, then as the first row | nothing | 0 |
| Harbour busy past the deadline | the expired request is dropped, nothing opens | `no answer from harbour` | 1 |
| Harbours connected but none answered the `ask` in time | unchanged | `no harbour answered` | 1 |
| A path climbing out of the root (`..`), a symlink in the root to a file outside it (dangling or not), a folder whose name only starts like the root | unchanged | `no harbour session holds <path>` | 1 |
| A message holding control characters | (none) | the characters escaped (`\n`, `\u{1b}`) | 1 |
| The path is a directory | (no request) | `<path> is a directory` | 1 |
| Its folder does not exist | (no request) | `<path>: No such file or directory` | 1 |
| Path longer than 4096 bytes | (no request) | `the path is longer than 4096 bytes` | 1 |
| The state folder is not private (symlink, other owner, group or other bits) | (no request) | `<folder> is not private; refusing to use it` | 1 |
| The socket path would not fit an `AF_UNIX` address | harbour does not listen | `no harbour is running` (Known limits line) | 1 |
| No `XDG_STATE_HOME` or `HOME` | harbour does not listen | `no harbour is running` | 1 |
| `harbour edit -- <file>` | as `harbour edit <file>`, for a file whose name starts with `-` | as for that file | as for it |
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
- A stopped card (phase 10's remembered session, no process) never holds a file: `harbour edit`
  never resumes a conversation, since that starts Claude Code, which is the user's call. It
  passes over the card to a running holder, else says no session holds the file.
- While a question or the picker is open, an `ask` is answered `none` instead of a refusal, so
  another harbour that holds the file can take it; an `open` is still refused.
- Where each session works is read again before an `open` only, not before an `ask`: an `ask`
  stays cheap for every harbour, and the harbour that gets the `open` picks the holder and the
  root from the same fresh reading.
- A refused open leaves harbour's screen as it was: the reason goes to the caller only, since the
  user is in another window.
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
   refusals from Neovim; the fixture Neovim in-process.
6. The terminal interface listens; end-to-end tests run `harbour edit` against harbour in a
   scratch tmux server, with the fixture editor and with a real `nvim --clean` (moved here from
   task 5: a real Neovim runs only in the tmux tests).
7. Guidelines: the layout lines and Known limits in `AGENTS.md`, the README checked, the
   screenshots.

## Outcome

- `harbour edit <file>` shipped: the file made absolute, every running harbour asked over a
  socket in its private state folder which session holds it, the one used last told to open
  it; harbour selects that session and shows its Neovim with the file, starting one at the
  session's root when none runs there.
- The channel is one Unix socket per harbour (`<pid>-<n>.sock`, 0600, in a 0700 folder),
  a request read only from the same user, within 100 ms, before its expiry; replies are one
  line. Neovim gets the file after `--` when it starts, else through `nvim_cmd` `edit` with the
  path as bytes and `magic` off, which a real clean Neovim confirmed for `acme %b#c|d e.rs`.
- The app answers up to 4 requests a frame on the draw thread; an open into a running Neovim
  waits at most 1 s for it. Both are in "Known limits".
- A session holds only paths inside its root; the editor used is the one running at that root,
  else a new one, as with Ctrl-l (older editors at other roots are not chosen).
- The fixture editor `tests/fixtures/editor.sh` now prints its last argument too.
- Review round 1 moved the socket off the draw thread (a listener thread, a reader thread per
  connection, at most 16 in flight, a channel to the draw loop), had the receiver resolve and
  check the path itself, followed dangling symlinks on both sides, made a second open wait for a
  Neovim whose socket is not up yet, carried one deadline to the last step, refreshed where each
  session works before an open, and left the screen alone on a refusal. The prefix look-alike
  finding was a false premise (`Path::starts_with` compares whole components); its test stays.
  The moving tests' waits went from 5 s to 12 s (`wait_until_at`, `wait_until_moved` in
  `tests/app/moving.rs`); the 600 ms `settled` wait asserts that nothing moves and stays.
- Merged with phase 10 (v0.9.0): stopped cards never hold a file; the channel's folder,
  `harbour/edit/`, stays apart from phase 10's `harbour/sessions/`.

## Hurdles

- `tooling` On macOS, `setsockopt` read and write timeouts fail with EINVAL once the peer has closed, so a reply to a sender that already left errored before the write; timeouts are now set before writing and their errors ignored → code
- `test-seam` A temporary `Listener` in a test was dropped at the end of its statement, taking its socket with it, and the test failed for that reason alone; the tests keep a binding → none
- `test-seam` The symlink case first went through the picker, which canonicalizes the directory it prints, so the test could not go red; it starts the session through `with_home` of a symlinked folder instead → code
- `tooling` `moving::claude_code_moving_into_a_worktree_moves_harbour_to_the_worktree_root` timed out once under the full load of a ship (passes alone and in full runs); the ship passed on retry → none
- `ci` `nix::unistd::getpeereid` does not exist on Linux, so `check` and the Linux `flake` failed to build a branch that built and passed on macOS; the peer check reads `SO_PEERCRED` on Linux, verified in a Linux container before the push → code
- `process` Early in the phase some production code was written before its test had been run red, against the TDD rule; later tasks kept red first and broke by hand every test that passed at once → none
- `review-loop` Review round 1 found 7 should findings in the channel and the open (socket I/O on the draw thread, the receiver trusting the sender's path, dangling symlinks, back-to-back opens, a refusal changing the screen, the deadline not reaching the last step, a stale root) → code
- `design` Reading sockets on the draw thread let a flood of silent connections freeze the keys for seconds; the listener became a thread of its own with a reader per connection and a channel, and only the Neovim RPC stays on the draw thread → rule in `AGENTS.md`
- `ci` On Linux, a connection the listener closes without reading its request is reset, so the cap test's read failed with `ECONNRESET` where macOS gives an empty read; the test takes either as closed unanswered, verified in a Linux container before the push → code
