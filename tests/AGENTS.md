# tests

What has gone wrong in the tests before, and how the suites are set up to avoid it. Rules are in
the root `AGENTS.md`, working agreements in `docs/process.md`. Most traps below were hit in
conveyor, where these suites come from; the ones about sessions were hit in harbour.

## Where a test belongs

These rules override a TDD skill or personal instruction that prefers outside-in or
integration-first tests. One cosmetic change once cost edits in three files (one added space
after the Clawd glyph: `tests/tabs.rs`, `tests/app/side_border.rs`, `tests/e2e/config.rs`).

- **Each behaviour has an owner test at the lowest level that does it**, and a change's red test
  goes there. A string drawn by a pure function (`tabs::claude`, `card::lines`, `state::title`,
  `help::rows`) is owned by that function's table (`tests/tabs.rs`, `tests/card/`, ...); a string
  only the app draws (the ` | ` between the pane's names, a focused border) by one test under
  `tests/app/`; a script's behaviour by its test in `tests/scripts/`.
- **The owner pins the exact literal, once.** Every other test that shows the same text builds
  its expected text from the production function (`tabs::claude(Some(ClaudeIcon::Clawd))`), or
  checks presence and order (`in_order(top, &["claude", "acme-edit"])`: the glyph before the
  name, the name before the editor), never the literal again.
- **Borders are checked by role, not by glyph.** A pane's border types are `FOCUSED_BORDER` and
  `UNFOCUSED_BORDER` in `src/app/mod.rs`; the helpers (`focused`,
  `unfocused`, `focused_title`, `shows_titled`, `top_left`, `vertical`, `drawn` in `tests/app/`,
  `corner`, `focused_corner`, `focused_on`, `focused_at`, `unfocused_at`, `vertical` in
  `tests/e2e/borders.rs`) read them, and a test that spells a pane border builds it from their sets. Only
  `focus::a_focused_pane_has_the_thick_border` names a pane border's glyphs (`┌`, `┏`) and
  `marks::the_waiting_prefix_shows_on_the_panes_bottom_border` its `┘`, so a new style fails only
  those owners; both helper files take every `BorderType` from `tests/support/borders.rs`, whose
  exhaustive `match` stops compiling when ratatui adds one or the list misses one. A glyph alone
  may not tell two styles apart (the light dashed sets share `┌` with the plain one, the rounded
  set `│` and `─`), so
  `focused` and `unfocused` pass only a glyph of one style that the other does not draw, and fail
  on a shared one rather than pass with the wrong pane focused; the e2e checks of which pane has
  focus read the corner and the side below it at one display column (`focused_at`,
  `unfocused_at`, `focused_on`). The quadrant sets draw `▌`, the selected card's bar, so `vertical`
  and `drawn` match it: a card row's text is read with `inside`, which skips the bar.
  A popup's own border is not a pane's: the help's and the changelog's plain `│` and `┌ help`
  are named in `tests/app/help.rs` and `tests/app/changelog.rs`. A card's indent is found
  (`column_of`) or trimmed, never counted, outside `tests/card/`.
- **Each level has one job.** Pure tests pin exact strings and every edge of a table (every
  width, every spacing). In-process `tests/app/` tests check what shows, in what order and in
  which pane. `tests/e2e/` proves each user-facing feature (a key, a subcommand, a view, a
  config option) is wired up through the real binary: every feature keeps at least one
  end-to-end test, asserting loosely (presence, order, text built from the production
  function), never the exact literal. What moves down is exact pinning and repeated cases, not
  the feature's one end-to-end test; an end-to-end test that only repeats a case of a feature
  already proven end to end, and covered in process, is deleted.
- **Before adding a test, look for the one that already pins it** at a lower level; add a row
  there, or nothing.
- **A test waits for exactly what it asserts** (see the trap below). A whole-screen comparison
  (`assert_eq!(screen.draw(), before)`) follows a wait for that whole screen or for the screen
  to stop changing (`wait_for_stable` in `tests/app/`: a second without a change, longer than
  the app's slowest periodic redraw, the 500 ms read and check ticks), never a wait for one word
  of it. A line checked on the screen is waited for as that line (`wait_until` with the same
  predicate).
- **Shared helpers live in the binary's `main.rs`** (`tests/app/main.rs`, `tests/e2e/main.rs`;
  the e2e border helpers in `tests/e2e/borders.rs`, since `tests/source_size.rs` caps a file
  at 700 lines), each wait with a test in that binary's `waits.rs` that sees it fail.
- **Script tests guard what ships and releases**: the gates, `ship.sh`, the scrub check, the
  version check and the release scripts. A test of a usage line or a help text that no gate,
  workflow or user reads is not written.
- **No Rust test reads a workflow file.** A change to `.github/workflows/*.yml` is proven by the
  CI run of its own pull request, not by a test pinning its text, and needs no red test first.

## Testing traps hit so far

- **tmux is a test harness, not a dependency.** Only the `#[ignore]` tests of `tests/e2e/` use
  it: they run the real binary in a scratch tmux server of their own (`-L harbour-e2e-<name>-<pid>`,
  `-f /dev/null`, killed when the test's `Server` drops), never the user's server. They panic
  with an install hint when tmux is missing; `gates.sh` runs them with `--include-ignored` and
  `--test-threads=2`, after every other target, when tmux is installed and says it skipped them
  otherwise; the `rest` leg of the `shard` job installs it. A pane whose
  command exits at once loses its output before `capture-pane` reads it, even with
  `remain-on-exit`, so the pane command ends in `echo "harbour exited $?"; exec sleep 600`.

- **No elapsed-time bound is an idle-machine constant.** A busy machine stretches every wait, so an
  upper bound on elapsed time comes from `deadline()` in `tests/support/timing.rs` (2 s, times the
  whole number in `HARBOUR_TEST_SLOWDOWN`, 1 when unset), pulled in with `#[path]` (in `tests/app/`
  once, by `main.rs`). A test of giving up injects `deadline()` and asserts `elapsed >= deadline()`
  and `elapsed < 2 * deadline()`, or only the upper bound where a platform refuses at once (a socket
  that never accepts is refused at once on macOS, so `tests/nvim.rs` asserts only it); a test of
  returning early injects or stalls `5 * deadline()` and asserts `elapsed < deadline()`
  (`ending_sessions_that_exit_on_hangup_returns_before_the_grace_runs_out` in `tests/session/`,
  ending sessions that exit on their hangup); any other deadline a seam or a waiting helper takes is
  `5 * deadline()`, and a stall that must outlast a test's waits is `10 * deadline()`. A test that
  harbour is fast checks the order of events the fakes record, not the clock: each session and
  editor hangs up before any is killed (`tests/app/ending.rs`, `tests/app/editor_ending.rs`: a
  fake's HUP trap writes how many fakes it saw alive (by `ps`, so an unreaped zombie counts as dead)
  and exits once every fake has written, giving up after about `5 * deadline()` of polling, under a
  hangup grace of `5 * deadline()` injected with `with_hangup_grace`; fakes that ignore the hangup
  make quitting wait a grace of `deadline()` once in all), C-h on a silent Neovim sends one mode
  query (`tests/app/neovim.rs`), an open that raises a prompt in a running Neovim is refused once a
  mode query sees it (`tests/app/handing.rs`), the picker is drawn and reads a second key before a
  slow list ends (`tests/app/picker.rs`). A time bound stays only where no event can be ordered
  against it: on `deadline()` for an open whose Neovim exits or prompts (`tests/app/handing.rs`) and
  for writes to a log that cannot be written (`tests/log.rs`); on a test constant mirroring `src`
  with the loosest margin a regression still breaks (`PATIENCE` in `tests/app/neovim.rs`, stretched
  by `scaled` from `tests/support/timing.rs`, `OPEN_PATIENCE` in `tests/app/handing.rs` and `ASKING`
  in `tests/edit.rs`, plus `deadline()`) where the time is the product's own latency and no deadline
  can be injected. C-h on a silent Neovim costs one `PATIENCE` wait, so the time it waits is bound
  by `10 * PATIENCE`: the loosest margin that still breaks if the default grows to `OPEN_PATIENCE`.
  `tests/wall_clock.rs` fails on a `<` or `<=` bound by a
  `Duration::from_` or after `as_millis()` or `as_secs()`, a `Duration::from_` on the left of `>`,
  a `recv_timeout(Duration::from_` not followed by `.is_err()` on its line, and a literal
  `Duration::from_` anywhere inside a call of `with_program`, `with_patience`,
  `with_hangup_grace`, `hand`, `finish`, `retrying_every`, `nvim::mode` or `nvim::eval`. It
  cannot see a literal reached through a `let`,
  a `const` or a helper of its own (a poll loop's `Instant`), a bound split across lines or
  written `>` with the count on the right, or a seam call holding a raw string or a character
  escape longer than one character with a parenthesis.

- **No test starts the real `claude`.** A session runs the program `HARBOUR_SESSION_COMMAND`
  names (`SESSION_COMMAND` in `src/tui.rs`, on the `SEAMS` list), `claude` when it is unset.
  The end-to-end tests set it to `tests/fixtures/agent.sh`, `tests/fixtures/raw-agent.sh` or
  `tests/fixtures/registered-agent.sh` or `tests/fixtures/titled-agent.sh` (which write a registry
  file, and the second a transcript, so only under a scratch `HOME`) or `tests/fixtures/shaped-agent.sh`;
  in-process tests hand `App::new` or `Session::start` a `/bin/sh -c` script; the one executable
  a test writes is a picker whose interpreter is missing. The usage popup of `u` runs the
  session command's program only when its file name is `claude`: `tests/app/usage.rs` links
  `tests/fixtures/args-agent.sh` as `claude` in a scratch directory, and every other test hands
  the report to `App::with_usage` as a script. Every app that presses `u` needs
  `App::with_usage_place` pointing under the scratch `HOME` (`usage_states::app()` does it),
  or `u` refuses for want of a state folder.

- **No test opens the real fzf or reads the real zoxide database.** `n` runs fzf over
  `zoxide query --list` unless `HARBOUR_DIR_PICKER` names another picker, and both are on the
  build machine's `PATH`, and a Linux distribution may install both in `/usr/bin`. The
  end-to-end tests clear `HARBOUR_DIR_PICKER` and `HARBOUR_SESSION_COMMAND` unless the test sets
  them, and run the binary on a `PATH` of links to the tools the fixtures need (`tools` in
  `tests/e2e/main.rs`), where neither is, so `n` starts at once, and with `TMPDIR` under the scratch `HOME`, so a picker still open when the tmux server dies leaves no `harbour-pick*` directory behind; they set `HARBOUR_DIR_PICKER`
  to `tests/fixtures/picker.sh` to test the picker; in-process tests hand the builder `.with_picker` of `App` a `Picker` of `/bin/sh -c`
  scripts. A picker script draws on stderr: its stdout is the directory it picks.
  The session finder runs only a picker whose file name is `fzf`, so its tests use
  `tests/fixtures/finder/fzf`, which answers like the directory picker (`.`) when it is not given
  the finder's `--with-nth=2..`, so `n` still starts a session with it set; it reads what is typed
  from `/dev/tty`, a line starting with Esc exits 130 (so a test closes it with Esc, then
  Enter), an empty line is read past, `broken` exits 2 and `cut:<text>` prints the line
  holding `<text>` without its number and dot. `tests/fixtures/nth-finder/fzf` wraps it and
  prints the line without its number unless given `--accept-nth=..` (a user's `--accept-nth`);
  `tests/fixtures/old-finder/fzf` exits 2 on any `--accept-nth`, as an fzf older than 0.60.

- **No test reads the user's config file.** The interface and `harbour config` read
  `$XDG_CONFIG_HOME/harbour/config.toml`, else `~/.config/harbour/config.toml`, so every launch
  of the binary sets `XDG_CONFIG_HOME`, or removes it and sets a scratch `HOME`, in the function
  that launches it; `tests/real_config.rs` names a launch that does neither. A test of a setting
  writes the file under its scratch `HOME`; in-process tests call `Config::read` with the variables they give it.
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
  `tests/e2e/neovim.rs` keeps four smoke tests that the wiring through tmux and harbour still
  holds (a tmux command run from Neovim acting on harbour's pane and C-h after it, `x` on a
  modified buffer saying it has unsaved changes, `harbour edit` twice, and `harbour edit` starting
  a Neovim stopped at a stale swap prompt, refused with exit 2)
  beside its extended-keys tests. Its `TMUX_TMPDIR` is an existing folder
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
  `cargo test` of its own, with `--test-threads=2`, at a cost of about 40 s a ship. An in-process
  `app` test has failed while another ship tested beside it; another worktree's gates now wait
  behind the gates lock (`scripts/AGENTS.md`), but load from anything else (a test run
  outside `gates.sh`, a build) is not covered.
- **System calls differ between macOS and Linux, and `check` runs on Linux only.** Development
  is on macOS. In phase 12 `nix::unistd::getpeereid` did not exist on Linux, so `check` failed to
  build a branch that passed on macOS, and a connection the listener closed without reading its
  request was reset on Linux (`ECONNRESET`) where macOS gives an empty read. A phase that touches
  sockets or other system calls runs its tests in a Linux container before its first push, from
  a `rust:<channel>` image, the channel of `rust-toolchain.toml` (phase 6 used `rust:1.98.1`;
  phase 12 recorded no command). The end-to-end tests are `#[ignore]` and need tmux, which the
  stock image lacks: covering them means installing tmux in the container (the `rest` leg of `shard` in
  `ci.yml` installs it with `apt-get`) and running them as `gates.sh` does, after the rest, at
  2 threads (see the trap above): `cargo test --test e2e -- --include-ignored --test-threads=2`.
  CI runs the targets with `cargo nextest run` in two shards, the `e2e` target at 2 threads
  (`.github/AGENTS.md`).

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
  end-to-end test of a modified acwrite buffer, under load, the sidebar
  click was lost and its `x` reached Neovim, whose buffer read `acxme`. After each group, wait
  for what it shows and assert on what the wait returned: the typed text, the mode
  (`-- INSERT --`), a pending key in Neovim's `showcmd`, a focused border (`focused_on(screen, "sessions")`), or,
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
  the reader a here-string or a file. `tests/scripts/pipe_readers.rs` reads `scripts/*.sh` as
  shell (quotes, comments and here-documents are text) and fails, naming the file and line, on a
  pipe into `head`, `read` or a `grep` with `-q`, `-m`, `-l`, `-L` or their long names, and on an
  `exit` outside `BEGIN` and `END` in an `awk`, `gawk`, `mawk` or `nawk` program written as one single-quoted word. It
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
  showed on macOS. `tests/scripts/` takes no mutex: its shims are symlinks to tracked scripts
  under `tests/fixtures/` (an `ln` that fails is `tests/fixtures/failing-ln.sh`, driven by
  `LN_FAILS`, `LN_FAILS_ONCE` and `REAL_LN`), and the scripts under test are symlinked into the
  scratch directory, except `a_grep_that_cannot_run_fails_the_filter` in `flake_inputs.rs`,
  which still writes its `grep` and runs it without one. A new test that does write an executable takes a mutex
  shared by every such test for its whole body. Treat a green PR run as no proof against this
  class: it is timing-dependent.
- **A blocked `gates.sh` outlives a test that panics.** Early lock tests left `gates.sh` runs
  waiting on a held gate after their assertions failed, orphaned for minutes in a deleted
  temporary directory, where another builder's `pgrep -f scripts/gates.sh` waited on them. A
  test that starts a `gates.sh` that can wait or block runs it as a `Gates` of
  `tests/scripts/gates_lock.rs`: its own process group, killed on drop, the test failing while
  any of it survives.
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
  the unwanted thing (`settled` in `tests/app/main.rs`, `remembered_after_a_while` in
  `tests/app/keeping.rs`).
- **`#{pane_current_path}` is the directory of the pane's foreground process group leader.**
  `Server::run` starts harbour under a non-interactive `sh -c`, which has no job control, so
  harbour stays in the shell's process group and tmux reports the shell's directory, never
  harbour's. A user's interactive shell gives harbour a group of its own. A test that reads the
  pane's directory starts harbour with `Server::exec`, which makes it the pane's own process. No
  in-process test really moves the process: `App::new` moves nothing, and `tests/app/moving.rs`
  hands the app a recording mover.
- **`extended-keys-format` needs tmux 3.5.** The tests of `tests/e2e/keys.rs` set it (and
  `extended-keys`, which `always` needs from 3.2) on the scratch server before harbour starts;
  when tmux refuses an option they print why and pass, as the Neovim tests do without `nvim`,
  so an older tmux runs them as no-ops instead of failing the gates. `Server::server_option`
  says whether tmux took it; the real-Neovim tests under `extended-keys on` skip the same way.

## Layout

What each test file and fixture covers, moved here from the root `AGENTS.md`. Update this entry,
never the root's Layout, when a test or fixture changes what it covers.

```
tests/           by level (see "Where a test belongs"): `tests/cli.rs` runs the real binary;
                 `tests/scripts/` runs the scripts in a scratch directory and repository, with `tests/fixtures/` standing in for `cargo`, `gh`, `curl` and the scripts they call;
                 `tests/e2e/` runs the real binary in a scratch tmux server (`#[ignore]`, see the traps above), with `HARBOUR_SESSION_COMMAND` pointing sessions at `tests/fixtures/agent.sh` (prints its `TMUX` and `TMUX_PANE`, then what it reads, a line at a time) or `tests/fixtures/raw-agent.sh` (asks for bracketed paste and the mouse, and for the extended keys `ACME_KEYS_ASKED` holds when a test sets it, prints every byte it reads, ESC as `^`, CR as `|`, C-a as `@` and C-] as `&`) instead of `claude`, or at `tests/fixtures/args-agent.sh` (prints its arguments, also linked as `claude` on the scratch `PATH` to see what the default gets), or at `tests/fixtures/registered-agent.sh` (writes a busy registry file under `CLAUDE_CONFIG_DIR`, else `HOME`, so it only runs with a scratch `HOME`) or `tests/fixtures/titled-agent.sh` (the same with a `sessionId` and a transcript) or `tests/fixtures/resumable-agent.sh` (the same, saying `acme agent resumed <id>`, then `acme agent args:[<its arguments>]`, when `--resume <id>` is among its arguments) or `tests/fixtures/fifo-agent.sh` (makes its registry file a FIFO, which holds harbour's draw thread in its read until a test opens it for writing, how `tests/e2e/signals.rs` holds the loop); `tests/e2e/cards.rs` holds the tests of the cards, `tests/e2e/cursor.rs` reads `#{cursor_shape}` of harbour's pane with `tests/fixtures/shaped-agent.sh` (sends `ESC [ <line> SP q` for each line it reads), `tests/e2e/keys.rs` the keys under tmux's `extended-keys` on, off and always, in the xterm and the csi-u form, what an editor that asked for extended keys gets and that Claude Code asking still gets the legacy keys, `HARBOUR_DIR_PICKER` at `tests/fixtures/picker.sh` (prints its `TMUX` and `TMUX_PANE` on stderr, then the line it reads), `HARBOUR_EDITOR` at `tests/fixtures/editor.sh` (prints its first and last argument and its `TMUX` and `TMUX_PANE`, also linked as `nvim` to see what Neovim gets) or at `tests/fixtures/nvim/nvim` (a real Neovim with `--clean`, see the traps above), all three cleared from the environment unless the test sets them, `CLAUDE_CONFIG_DIR` cleared, `TMPDIR` a directory under the scratch `HOME` (so a picker left open leaves nothing behind), `HARBOUR_LOG=1` (a failing test prints the newest log under that `HOME`; `tests/e2e/log.rs` checks it never holds what was typed, pasted or a title), and `PATH` a scratch directory of links to the few tools the fixtures run, so no fzf or zoxide is on it;
                 `tests/log.rs` covers the log's folder, file, stamp, sweep and `newest`; `tests/app/logging.rs`, `logging_lives.rs` and `logging_edits.rs` what the app logs, read from a `Log::sending` channel;
                 `tests/status.rs` runs `harbour status` over summaries a test writes and locks itself, `tests/summary.rs` writes them with `Summary`;
                 `tests/session/` (`keyboard.rs` what a program asked for in keys) and `tests/app/` run sessions in process, as `/bin/sh -c` scripts;
                 `tests/edit.rs` runs `harbour edit` against listeners of `tests/handoff.rs`'s kind, which a test answers itself; `tests/app/handoff.rs` hands the app requests and stands in a socket for Neovim; `tests/app/nearest.rs` covers which holder gets the file and what moves its last use; `tests/e2e/edit.rs` runs `harbour edit` against harbour, and `tests/e2e/neovim.rs` holds four smoke tests and the extended-keys tests against a real clean Neovim; `tests/app/real_neovim.rs` runs that Neovim's TUI in a harbour session and checks `mode`, `eval`, `edit`, `changes`, `wants_ctrl_h`, `wants_ctrl_l` and `open` against it; `tests/app/neovim.rs` stands in a socket for Neovim and covers C-h, swapped C-l (`wants_ctrl_h`, `wants_ctrl_l`), `x` and `q` over it; `tests/idle.rs` (Linux only) counts the listener thread's wake-ups while idle;
                 `tests/app/prefix.rs` covers the prefix and what follows it in process, `tests/app/marks.rs` the waiting mark and the way to the help, `tests/help.rs` the help's rows, `tests/app/jumping.rs` the prefix then a digit, `tests/e2e/keys.rs` the prefix sequences through tmux with extended keys on and off;
                 `tests/app/settling.rs` what each step's result ends harbour in (a caught signal stops it and ends by that signal, an end of input or EIO by HUP, any other error stops it and goes on as the error, a signal after a confirmed quit ends by it and logs nothing more), `tests/app/zoom.rs` covers zoom in process, `tests/app/width_keys.rs` the width keys, `tests/app/side.rs` side by side in process, `tests/app/side_jump.rs` the jumps and `harbour edit` while side by side, `tests/app/side_states.rs` the rest of its states, `tests/app/side_border.rs` its borders, `tests/app/side_mouse.rs` its clicks and gestures, `tests/app/side_swap.rs` the swaps, `tests/app/finding.rs` and `tests/app/finding_states.rs` the session finder in process (`HARBOUR_DIR_PICKER`'s place taken by `tests/fixtures/finder/fzf`, a script named `fzf` that prints its arguments and the list on stderr, then picks the first listed line holding what is typed, never a real fzf; `tests/fixtures/nth-finder/fzf` prints the line without its number unless given `--accept-nth=..`, `tests/fixtures/old-finder/fzf` refuses `--accept-nth` as an old fzf does), `tests/finder.rs` its lines, `tests/picker.rs` the message of a list that cannot be written, `tests/e2e/finding.rs` the prefix then `s` through tmux, `tests/app/zoom_border.rs` the zoomed top row, `tests/zoom.rs` the table of how its headline fits a room;
                 `tests/tmux.rs` runs `select` against a `/bin/sh -c` script in place of tmux (`Tmux::with_program`); `tests/app/navigating.rs` covers C-j and C-k in process the same way; `tests/e2e/navigating.rs` splits the scratch server's window and reads which pane is active, with `tmux` linked onto the scratch `PATH`;
                 `tests/app/moving.rs` hands the app a recording move, never the real one; `tests/e2e/moving.rs` reads `#{pane_current_path}` of a harbour started with `Server::exec`;
                 `tests/app/usage.rs` covers the usage popup of `u` (what reaches it, what it leaves behind, its program), `tests/app/usage_states.rs` what it shows (the loading animation and its clock, nothing, an exit, its size, its width after a shrink and its private folder), `tests/app/usage_bars.rs` the bars a finished report draws, no text before them and `t` while the report loads (that they stay once shown only on Linux, where a PTY still delivers what a child of the report prints after it exits), `tests/usage/` the table of what `parse` reads and `lines` draws (`main.rs`) and what `loading` draws (`loading.rs`);
                 `tests/tabs.rs` is the table of how the pane border's names fit a width, with and without icons;
                 `tests/app/changelog.rs` covers the changelog view of `c` in process (`App::with_changelog` gives it a text), `tests/changelog.rs` the lines `shown` makes;
                 `tests/text.rs` covers measuring, cutting, wrapping and cleaning text, `tests/card/` a card's lines (`main.rs`, `colours.rs` its colours), `tests/repo.rs` reading a repository and branch from scratch `.git` layouts, `tests/transcript.rs` reading a transcript, `tests/memory/` every file state of the remembered sessions (`reserving.rs` the locks and the sweep, `places.rs` the directory bytes and the configurations);
                 `tests/keys.rs` is the table of the bytes each key sends, `tests/input.rs` the table of what each sequence reads as, `tests/mouse.rs` the table of what a session receives for each mouse event, mode, gesture and modifier;
                 `tests/tui.rs` covers entering the terminal (extended keys asked for) and leaving it (mouse capture, bracketed paste and extended keys off, the default cursor shape only after another was sent) and the panic hook, `tests/cursor.rs` when a cursor shape is sent;
                 `tests/config/` reads the settings file in process (every refusal, the precedence) and runs `harbour config`; `tests/e2e/config.rs` runs the interface over a scratch config; `tests/real_config.rs` fails while a launch of the binary under `tests/`, or the launch line of `scripts/screenshots.sh`, leaves the user's `XDG_CONFIG_HOME` or `HOME` in place, naming it;
                 `tests/readme.rs` pins the README shape (see `docs/AGENTS.md`);
                 `tests/agents_size.rs` fails while the root `AGENTS.md` is over 12 KiB, a line of the fenced block inside its `## Layout` section over 120 characters, or the `CLAUDE.md` of `src/`, `tests/`, `scripts/`, `docs/` or `.github/` neither a symlink to `AGENTS.md` nor a file holding only that name, or that directory's `AGENTS.md` missing; `tests/scripts/flake_inputs.rs` also checks that an `AGENTS.md` or `CLAUDE.md` under `src/` is no flake input;
                 `tests/source_size.rs` fails while any Rust file under `src/` or `tests/` is longer than 700 lines, naming it;
                 `tests/checkout_name.rs` fails while a test under `tests/` reads the process's directory (`current_dir()`, the checkout under cargo) or builds a `tests/app/` `Screen` outside `tests/app/main.rs`, naming the file and line (what it cannot see: `tests/AGENTS.md`);
                 `tests/toolchain.rs` fails while `rust-version` in `Cargo.toml` differs from the `major.minor` of the channel of `rust-toolchain.toml`, or `flake.nix` stops leaving out its components;
                 `tests/plans.rs` fails while a plan file is missing from the `PLAN.md` index, a phase plan has no `## Outcome` or a malformed `## Hurdles`, a sweep phase's `## Outcome` has no `### Sweep`, or a phase has no `plans/metrics.md` row;
                 `tests/wall_clock.rs` fails while a test bounds elapsed time by a constant or hands a seam a literal deadline (see the traps above);
                 `tests/dead_code.rs` fails while a `pub` item under `src/` has its name nowhere else in `src/` (a name scan, not a resolver) and is not on its `SEAMS` list, while a seam on that list is no longer declared or used by a test, or while a `dead_code` attribute sits anywhere under `src/` or `tests/`, naming each;
                 a test file that outgrows the limit becomes a folder (`tests/<name>/main.rs` plus one file per topic), so it stays one binary
```
