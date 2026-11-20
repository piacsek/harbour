# tests

What has gone wrong in the tests before, and how the suites are set up to avoid it. Rules and
working agreements are in the root `AGENTS.md`. Most traps below were hit in conveyor, where
these suites come from; the ones about sessions were hit in harbour.

## Testing traps hit so far

- **tmux is a test harness, not a dependency.** Only the `#[ignore]` tests of `tests/e2e/` use
  it: they run the real binary in a scratch tmux server of their own (`-L harbour-e2e-<name>-<pid>`,
  `-f /dev/null`, killed when the test's `Server` drops), never the user's server. They panic
  with an install hint when tmux is missing; `gates.sh` runs them with `--include-ignored` when
  tmux is installed and says it skipped them otherwise; the `check` job installs it. A pane whose
  command exits at once loses its output before `capture-pane` reads it, even with
  `remain-on-exit`, so the pane command ends in `echo "harbour exited $?"; exec sleep 600`.

- **No test starts the real `claude`.** A session runs the program `HARBOUR_SESSION_COMMAND`
  names (`SESSION_COMMAND` in `src/tui.rs`, on the `SEAMS` list), `claude` when it is unset.
  The end-to-end tests set it to `tests/fixtures/agent.sh`, `tests/fixtures/raw-agent.sh` or
  `tests/fixtures/registered-agent.sh` (which writes a registry file, so only under a scratch `HOME`);
  in-process tests hand `App::new` or `Session::start` a `/bin/sh -c` script, so no test writes
  an executable.

- **dash is `/bin/sh` on Linux.** A session script's `read` returns non-zero when a trapped
  signal (the window change of a resize) interrupts it, so a loop written `while read` ends on
  Linux and not on macOS; write `while :; do read -r line || continue; ...; done`. A trap runs
  only once the foreground command (`read`, `dd`) returns, so a test that needs the trap to run
  sends the script a byte first.

- **A screen assertion waits for the text it asserts, never for a neighbouring signal.** A
  session's exit can be seen before its last output is parsed, and one write can reach the
  screen in two reads. `an_exited_session_keeps_its_last_screen_dimmed` waited for the sidebar
  to say exited, then asserted `acme agent done` in the pane, and lost that race on CI. Wait
  for every text the test asserts (`wait_for_all` in `tests/app/`, `wait_for_screen_all` in
  `tests/e2e/`), the whole line rather than its prefix, and end a value the test parses (a pid)
  with a marker to wait for.

- **`awk` is not the same program everywhere.** macOS ships BSD awk, the ubuntu runner's `awk`
  is `mawk`, and either may be `gawk` where someone installed it; BSD awk (version 20200816)
  and `mawk` count bytes, `gawk` in a UTF-8 locale counts characters, so in conveyor a script
  that measured `" → "` as 5 (its UTF-8 bytes) passed on one machine and failed on another. Measure a non-ASCII separator with `length()` and find it
  with `index()`, and before pushing a script that handles one, run its tests once with `gawk`
  first on `PATH` as `awk` and `LC_ALL=C.UTF-8`.
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
- **A sidebar row carries the name of the checkout's folder.** A session is named after the
  directory harbour starts in, so in-process tests name their rows after the folder the tests
  run from, and the sidebar is a third of the screen: in a folder named longer than about 22
  characters, `exited 3` fell off the end of the row at 120 columns and ten `tests/app/` tests
  failed in a review worktree. A test that asserts on a row (or waits for `exited`) builds its
  screen with `Screen::wide`, which sizes the screen to the folder name; one that cannot widen
  (the narrow layout) asserts only on the row's start, and a test that prints a path sizes the
  session to it. Before pushing such a test, run it once from a worktree whose folder name is 40
  characters or more.
