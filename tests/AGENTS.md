# tests

What has gone wrong in the tests before, and how the suites are set up to avoid it. Rules and
working agreements are in the root `AGENTS.md`. Most traps below were hit in conveyor, where
these suites come from; the ones about sessions were hit in harbour.

## Testing traps hit so far

- **tmux is a test harness, not a dependency.** Only the `#[ignore]` tests of `tests/e2e/` use
  it: they run the real binary in a scratch tmux server of their own (`-L harbour-e2e-<name>-<pid>`,
  `-f /dev/null`, killed when the test's `Server` drops), never the user's server. They panic
  with an install hint when tmux is missing; `gates.sh` runs them with `--include-ignored` and
  `--test-threads=2`, after every other target, when tmux is installed and says it skipped them
  otherwise; the `check` job installs it. A pane whose
  command exits at once loses its output before `capture-pane` reads it, even with
  `remain-on-exit`, so the pane command ends in `echo "harbour exited $?"; exec sleep 600`.

- **No test starts the real `claude`.** A session runs the program `HARBOUR_SESSION_COMMAND`
  names (`SESSION_COMMAND` in `src/tui.rs`, on the `SEAMS` list), `claude` when it is unset.
  The end-to-end tests set it to `tests/fixtures/agent.sh`, `tests/fixtures/raw-agent.sh` or
  `tests/fixtures/registered-agent.sh` or `tests/fixtures/titled-agent.sh` (which write a registry
  file, and the second a transcript, so only under a scratch `HOME`);
  in-process tests hand `App::new` or `Session::start` a `/bin/sh -c` script; the one executable
  a test writes is a picker whose interpreter is missing.

- **No test opens the real fzf or reads the real zoxide database.** `n` runs fzf over
  `zoxide query --list` unless `HARBOUR_DIR_PICKER` names another picker, and both are on the
  build machine's `PATH`, and a Linux distribution may install both in `/usr/bin`. The
  end-to-end tests clear `HARBOUR_DIR_PICKER` and `HARBOUR_SESSION_COMMAND` unless the test sets
  them, and run the binary on a `PATH` of links to the tools the fixtures need (`tools` in
  `tests/e2e/main.rs`), where neither is, so `n` starts at once, and with `TMPDIR` under the scratch `HOME`, so a picker still open when the tmux server dies leaves no `harbour-pick*` directory behind; they set `HARBOUR_DIR_PICKER`
  to `tests/fixtures/picker.sh` to test the picker; in-process tests hand the builder `.with_picker` of `App` a `Picker` of `/bin/sh -c`
  scripts. A picker script draws on stderr: its stdout is the directory it picks.

- **A real Neovim runs only clean, in `tests/app/real_neovim.rs` and `tests/e2e/neovim.rs`.**
  Each test looks for `nvim` on the `PATH` the tests run with; without one it prints why and
  passes, so CI, which has no Neovim, runs them as no-ops. With one, it links it as `real-nvim`
  on a scratch `PATH` (the one deliberate way that `PATH` finds it) and runs
  `tests/fixtures/nvim/nvim`, a wrapper named `nvim` (so harbour gives it `--listen`) that runs
  `real-nvim --clean`: no configuration or plugin is read, and with `HOME`, `TMPDIR` and
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `XDG_CACHE_HOME` under the scratch
  `HOME`, nothing is written outside it. A mapping a test needs (`<C-h>` to `<C-w>h`) is typed
  in that Neovim. Every other test uses `tests/fixtures/editor.sh` or a `/bin/sh -c` script.
  Neovim's TUI runs its server (`real-nvim --embed`) in a process group of its own: killing the
  tmux server does not end it, and one stuck at a swap prompt ignores SIGTERM and outlives the
  run. Dropping a `Neovim` of either file ends what it started, then SIGKILLs every `real-nvim`
  whose command line names the test's scratch `HOME`, and fails the test while one was still
  alive; a test of `tests/e2e/neovim.rs` that leaves one behind on purpose ends it with
  `end_every_neovim`.
- **The Neovim contract tests run its TUI in a harbour `Session`, never headless.** A
  `--headless` Neovim never reaches the prompt states (NVIM v0.12.6): a multi-line `:echo`
  leaves mode `n`, a stale swap file opens without asking. So `tests/app/real_neovim.rs` starts
  the wrapper through `Editor::start` (a PTY, no tmux, no harbour process), under
  `/usr/bin/env -u NVIM -u NVIM_APPNAME -u VIMINIT -u TMUX -u TMUX_PANE` (so a Neovim under test
  never reaches the tmux cargo runs in, and a run inside tmux matches CI) with the scratch variables, finds the
  editor's private socket as the one `harbour-nvim*/socket` under the scratch `TMPDIR`, types
  keys with `session.send`, and waits for each state over that socket (`nvim::mode`, or an
  `nvim::eval` that returns 1) before it asserts on `wants_ctrl_h`, `changes` or `open`. There
  the prompts show: press-enter is mode `r` blocking, the swap prompt `r?`, a pending `C-w`
  mode `n` blocking. `Session::end` ends the embed server too, even at a swap prompt.
  `tests/e2e/neovim.rs` keeps five smoke tests that the wiring through tmux and harbour still
  holds (C-h to Claude Code, a tmux command run from Neovim acting on harbour's pane and C-h
  after it, `x` over unsaved changes, `harbour edit` twice, and `harbour edit` starting a Neovim
  stopped at a stale swap prompt, refused with exit 2). Its `TMUX_TMPDIR` is an existing folder
  under the scratch `HOME` (tmux ignores one that does not exist), so a tmux command run there
  without `TMUX` finds no server instead of the user's default one.

- **dash is `/bin/sh` on Linux.** A session script's `read` returns non-zero when a trapped
  signal (the window change of a resize) interrupts it, so a loop written `while read` ends on
  Linux and not on macOS; write `while :; do read -r line || continue; ...; done`. A trap runs
  only once the foreground command (`read`, `dd`) returns, so a test that needs the trap to run
  sends the script a byte first.

- **The end-to-end tests lose their screen waits when the machine is busy.** At one test thread
  per core the `e2e` target starts a tmux server, a harbour and sometimes a Neovim per thread.
  Beside 24 CPU-burning processes on the 12-core build machine, 6 runs of 10 lost a test to a
  10 s screen wait (the swap prompt test 3 times, the name full of ex specials twice, the demo's
  made-up editor twice); at 2 threads, 0 of 10 failed. So `gates.sh` runs the target last, in a
  `cargo test` of its own, with `--test-threads=2`, at a cost of about 40 s a ship. Load from
  another worktree's test run is not covered: an in-process `app` test has failed while another
  ship tested beside it.
- **System calls differ between macOS and Linux, and `check` runs on Linux only.** Development
  is on macOS. In phase 12 `nix::unistd::getpeereid` did not exist on Linux, so `check` failed to
  build a branch that passed on macOS, and a connection the listener closed without reading its
  request was reset on Linux (`ECONNRESET`) where macOS gives an empty read. A phase that touches
  sockets or other system calls runs its tests in a Linux container before its first push, from
  a `rust:<channel>` image, the channel of `rust-toolchain.toml` (phase 6 used `rust:1.98.1`;
  phase 12 recorded no command). The end-to-end tests are `#[ignore]` and need tmux, which the
  stock image lacks: covering them means installing tmux in the container (the `check` job of
  `ci.yml` installs it with `apt-get`) and running them as `gates.sh` does, after the rest, at
  2 threads (see the trap above): `cargo test --test e2e -- --include-ignored --test-threads=2`.
  CI itself runs `cargo test --locked -- --include-ignored`: every target together, at the
  default thread count.

- **A screen assertion waits for the text it asserts, never for a neighbouring signal.** A
  session's exit can be seen before its last output is parsed, and one write can reach the
  screen in two reads. `an_exited_session_keeps_its_last_screen_dimmed` waited for the sidebar
  to say exited, then asserted `acme agent done` in the pane, and lost that race on CI. Wait
  for every text the test asserts (`wait_for_all` in `tests/app/`, `wait_for_screen_all` in
  `tests/e2e/`), the whole line rather than its prefix, and end a value the test parses (a pid)
  with a marker to wait for.

- **Keys sent to a program inside harbour are followed by a wait for their visible effect.**
  tmux delivers each `send_keys` at once, and harbour and the program take them in their own
  time, so the next group can land before the last one was handled. In the
  end-to-end test of a modified acwrite buffer (since moved in process), under load, the sidebar
  click was lost and its `x` reached Neovim, whose buffer read `acxme`. After each group, wait
  for what it shows and assert on what the wait returned: the typed text, the mode
  (`-- INSERT --`), a pending key in Neovim's `showcmd`, a focused border (`┏ sessions`), or,
  where nothing shows, an `:echo` marker that Neovim can only print once it took every key
  before it (`settled` in `tests/e2e/neovim.rs`).

- **`awk` is not the same program everywhere.** macOS ships BSD awk, the ubuntu runner's `awk`
  is `mawk`, and either may be `gawk` where someone installed it; BSD awk (version 20200816)
  and `mawk` count bytes, `gawk` in a UTF-8 locale counts characters, so in conveyor a script
  that measured `" → "` as 5 (its UTF-8 bytes) passed on one machine and failed on another. Measure a non-ASCII separator with `length()` and find it
  with `index()`, and before pushing a script that handles one, run its tests once with `gawk`
  first on `PATH` as `awk` and `LC_ALL=C.UTF-8`.
- **A reader in a pipeline reads to the end.** Under `set -o pipefail`, a writer still writing
  when its reader leaves dies of `SIGPIPE` (status 141) and fails the script: `version-check.sh`
  piped the lock into an `awk` that called `exit` once it had the version, and CI lost that race
  now and then on a correct lock. Below a pipe buffer it is timing, so a green run proves
  nothing; past it, it fails every time. In a script or a workflow, never pipe into `head`,
  `grep -q`, `grep -m`, `sed q`, `read`, a loop that `break`s, or an `awk` that calls `exit`
  outside `BEGIN` and `END`: remember the first answer (`!found && ... { print; found = 1 }`),
  `next` past the rest, set the status in `END`, take the first line with `sed -n 1p`, or feed
  the reader a here-string or a file. `tests/scripts/pipe_readers.rs` reads `scripts/*.sh`
  and the `run:` steps of `.github/workflows/*.yml` as shell (quotes, comments and
  here-documents are text) and fails, naming the file and line, on a pipe into `head`, `read` or
  a `grep` with `-q`, `-m`, `-l`, `-L` or their long names, and on an `exit` outside `BEGIN` and
  `END` in an `awk`, `gawk`, `mawk` or `nawk` program written as one single-quoted word. It
  cannot see, so this rule covers: an `awk` program held in a variable, read from a file
  (`-f`), double-quoted or only partly single-quoted, or given in a here-document; a reader
  started through another program (`xargs`, `env`, `sh -c`, `eval`); `sed q`, `perl`, `python`
  or a loop that `break`s; a `case` inside `$( )`, whose `)` ends the substitution early for the
  scan.
- **A scratch repository's `scripts/` are symlinks to the real scripts.** `link_script` links
  them, so a test that writes to one of those paths (`committed(&work, "scripts/<name>.sh", …)`)
  writes through the link and overwrites the tracked script in the worktree, and every later
  test runs the overwritten one. A test that needs a changed file under `scripts/` uses a name
  no test links.
- **Tests that write an executable and run it in the same test binary race on fork.** On Linux
  CI one test's `fs::write` was still open when another test's `Command` forked, the child
  inherited the fd and `exec` failed with `ExecutableFileBusy` ("Text file busy"); it never
  showed on macOS. `tests/scripts/` needs no mutex because it writes no executable: every shim
  is a symlink to a tracked script under `tests/fixtures/`, and the scripts under test are
  symlinked into the scratch directory. A new test that does write an executable takes a mutex
  shared by every such test for its whole body. Treat a green PR run as no proof against this
  class: it is timing-dependent.
- **A scratch `HOME` does not isolate what outranks it.** A variable exported in the shell wins
  over a scratch `HOME`: `HARBOUR_DENYLIST` names the real denylist whatever `HOME` is, and
  `GIT_DIR` or a global git config reaches into the scratch repository. The `ship.sh` and
  `scrub-check.sh` tests remove `HARBOUR_DENYLIST` next to their scratch `HOME`, and `isolated`
  (`tests/scripts/helpers.rs`) drops the git variables and configs; any new launcher that sets a
  scratch `HOME` removes what outranks it too.
- **A sidebar row carries the name of the folder its session starts in.** Harbour starts
  sessions in the directory it runs from, so in-process tests once named their rows after the
  checkout's folder, and three phases had tests that failed only in a worktree with a long name.
  Every `tests/app/` screen is now built by `Screen::of` (or `Screen::wide`, 200 columns), which
  gives the app a scratch folder named `acme-home` (`home()`, `directory()` in
  `tests/app/main.rs`), so a row reads `acme-home 1` wherever the tests run; `Screen::at` takes
  the home as an argument for a test that needs its own. A test that builds an `App` and starts a
  session before `Screen::of` gives it `.with_home(home())` itself; a relative path is read from
  `home()`, not the checkout. In phase 6 three new tests broke this rule while it was enforced by
  review alone, so `tests/checkout_name.rs` now reads every Rust file under `tests/` and fails,
  naming the file and line, on a `current_dir()` call (the process's directory, which cargo sets
  to the checkout; a `Command`'s `.current_dir(<dir>)` is left alone) and on a `Screen` built as a
  struct literal outside `tests/app/main.rs`. Its `ALLOWED` list holds the two lines of
  `tests/app/moving.rs` that compare the process's directory before and after, never draw it. It
  cannot see, so this rule covers: `CARGO_MANIFEST_DIR` or a path built from it handed to a
  session, an app's home, a picker or a socket (every current use reads a file of the repository
  or names a fixture); the directory read another way (`$PWD`, `std::env::var("PWD")`, `"."`, a
  relative path); and an `App` started outside a `Screen`, which takes the process's directory
  as its home.
- **Every wait fails the test when its deadline passes**, naming what it waited for and the last
  screen, path, line or count it saw. Waits once returned the last screen silently, so a wait
  whose text could never show (a stale demo name, a card's old layout, a numbered session that
  raced) passed as a mere pause. A new wait helper fails the same way and gets a test of its own
  that sees it fail (`waits.rs` in `tests/app/` and `tests/e2e/`). A test that means "nothing
  happens" uses a wait that says so: it keeps drawing for a fixed time and stops early only on
  the unwanted thing (`settled` in `tests/app/moving.rs` and `tests/app/switching.rs`,
  `remembered_after_a_while` in `tests/app/keeping.rs`).
- **`#{pane_current_path}` is the directory of the pane's foreground process group leader.**
  `Server::run` starts harbour under a non-interactive `sh -c`, which has no job control, so
  harbour stays in the shell's process group and tmux reports the shell's directory, never
  harbour's. A user's interactive shell gives harbour a group of its own. A test that reads the
  pane's directory starts harbour with `Server::exec`, which makes it the pane's own process. No
  in-process test really moves the process: `App::new` moves nothing, and `tests/app/moving.rs`
  hands the app a recording mover.
