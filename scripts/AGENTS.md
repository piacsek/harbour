# scripts/AGENTS.md

What each script does, and how the gates, the dev install and `ship.sh` behave. The root
`AGENTS.md` keeps only the gates list and the rules; the detail lives here.

## The scripts

```
scripts/gates.sh              the quality gates, one run at a time across the worktrees of the repository (a lock in its git common directory); fails loudly, never pipe it through tail
scripts/gates-lock.sh         sourced, not run: the gates lock `gates.sh` takes, and `ship.sh` holds across its gates and its dev install
scripts/scrub-check.sh        denylist grep over what a commit would take (see "The scrub check")
scripts/dev-install.sh        release build symlinked as ~/.local/bin/harbour-dev
scripts/ship.sh               gates + dev-install (unless skippable) + stage + scrub + commit (none when nothing is staged and commits are unpushed) + scrub of the outgoing commits and branch name + push, aborts on any failure; the first push of a branch opens the draft PR (commit subject as title, the template as body)
scripts/dev-install-skippable.sh  exit 0 with the reason when `ship.sh` may leave `harbour-dev` alone, else 1 with what needs the install
scripts/script-tests-skippable.sh exit 0 with the reason when `gates.sh` may leave out the `scripts` test target, else 1 with the first path that needs it
scripts/docs-only.sh          exit 0 with the reason when `gates.sh` may leave out every test but the ones that read Markdown, else 1 with the first path that needs them
scripts/feeds-binary.sh       reads paths on stdin, prints the ones that feed the binary; the one list of what does not (`tests/`, `docs/`, `changes/`, …)
scripts/flake-inputs.sh       reads NUL-separated paths on stdin, prints the ones the flake build reads the same way; the one list of what the `flake` job of `ci.yml` builds for
scripts/flake-build-needed.sh `<base>`: the `paths` job of `ci.yml`; `build=true` when a flake input changed since the base or nothing can be decided (the reason on stderr), else `build=false`
scripts/flake-ok.sh           `<paths result> <build> <flake result>`: the `flake-ok` gate of `ci.yml`; passes when the build succeeded or was rightly skipped
scripts/fragments.sh          sourced, not run: reads the pending fragments and refuses a malformed or repeated one, at the commit its caller names
scripts/version-check.sh      `<base> <head>`: the `version` job of `version.yml`; reads the head as git objects, runs nothing from it
scripts/next-version.sh       prints the version the pending fragments ask for, nothing when none is pending
scripts/release-commit.sh     `<version>`: writes the release into `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md` and confirms it reads back; commits nothing
scripts/release-rewrite.sh    sourced, not run: the release rewrite both `release-commit.sh` and `version-check.sh` use
scripts/releases.sh           sourced, not run: the state of a tag's release, and the newest complete release above a version (`formula-version.sh` and `stable-version.sh`)
scripts/release-unfinished.sh `<tag>`: whether `build` and `publish` still have work, so a rerun never rebuilds a published release
scripts/previous-tag.sh       `<tag>`: the highest `vX.Y.Z` tag below it, read with `gh`, which `publish` names as the previous release for the generated notes
scripts/release-tag.sh        the `tag` job of `release.yml` (refuses outside GitHub Actions): resume an unfinished tag, else tag what is pending, else nothing
scripts/homebrew-formula.sh   prints the tap formula for a released version
scripts/formula-version.sh    the `tap` job's decision; never a version below the formula's
scripts/stable-version.sh     the `stable-build` and `stable` jobs' decision, asked by both: the release the `stable` branch moves to, never below the version it holds
scripts/sweep-report.sh       `[<first phase>]`: the sweep's data from `plans/`, as Markdown; counts and groups, judges nothing
scripts/screenshots.sh        renders docs/details.png from `harbour --demo` and docs/help.png from `harbour --help`, plus one `shoot` per extra view (keys (the demo after `C-/ ?`), pane, question, quit (after `C-/ q`), ended (the demo with its blocked session ended, a count gone from the title), editor, usage (after `C-/ u`), narrow, zoom (the demo after `C-/ z`), sessions (the demo after `C-/ s`), side-by-side (the demo after `C-/ \`, the demo editor beside Claude Code), side-swapped (the same after `C-/ }`), prefix (the demo after `C-/` alone, the waiting mark on the pane's border), picker, accent (the interface without `--demo`, through a config file written into the scratch `HOME` and removed after it), icons (the same with `nerd_font = true` and an editor script named `nvim`, its session a script that writes a registry file and a transcript naming a pull request and two running subagents in a scratch repository on a branch, the border's and the card's glyphs shown), usage-loading (the interface without `--demo`, its session command a script named `claude` in the scratch `HOME` that sleeps, so the report never finishes, opened with `C-/ u` and captured once the bar shows one fixed frame, so the image is the same every run), changelog (the demo after `C-/ c`, from a scratch build in the scratch `HOME` whose `CHANGELOG.md` holds three made-up releases, built into `target/released/`); truecolor.py converts ANSI)
scripts/sweep-issue.sh        the `issue` job of `sweep.yml`: opens "Sweep due before phase N" with the report once the next phase is a multiple of 5, never twice
```

## The root files outside `src/`

```
CHANGELOG.md                  only its `# Changelog` heading on `main`; each release commit writes its sections
CLAUDE.md                     a symlink to `AGENTS.md`, for tools that read only that name; on a checkout without symlink support it holds only the text `AGENTS.md`, the file to read
rust-toolchain.toml           the one Rust version (rustfmt and clippy with it) that local gates and CI run; see Gates, ship and install
flake.nix, flake.lock         the Nix flake: the binary built from `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` and `src/` with the Rust `rust-toolchain.toml` names, through the locked `rust-overlay`, without the components it lists (rustfmt, clippy); feeds the binary
```

## The scrub check

No employer data, ever: no org or repo names, PR numbers or titles, logins, hostnames, cluster
or kube-context names, ticket prefixes. Tests and examples use invented `acme/*` data; plans and
docs use `<owner>/<repo>` placeholders. `scripts/scrub-check.sh` matches the local denylist at
`~/.config/harbour-dev/denylist` (or the file `HARBOUR_DENYLIST` names) against what
`ship.sh` publishes, three times: as the first gate and again between `git add -A` and
`git commit`, it reads the contents and the paths of the index and of the working tree's
tracked and untracked files that are not ignored; between `git commit` and `git push`,
`--outgoing` reads the branch name and, for every commit no remote has yet, its message, the
paths it adds or keeps and the lines it adds (a symlink's target and a merge's conflict
resolution included), never a removed line or a deleted path, so a branch can remove a term
`main` still holds. Each match says which of the four it was in and names its commit. A refused
push leaves the commit local: `git reset --soft HEAD~1`, fix what is named (`git branch -m` for
the branch name), ship again; an earlier commit has to be rewritten by hand. A refused merge is
kept and fixed with `git commit --amend`: with nothing staged and commits no remote has,
`ship.sh` pushes them without a new commit. The list is kept outside the repo on purpose; the
check is skipped only where that default path is confirmed absent (the nearest existing path
above it is a directory it can search), so CI does not enforce it, and fails, in both modes,
when the path is there but is not a readable file, when its absence cannot be confirmed, when
`HARBOUR_DENYLIST` names one that is not, when the list is empty, or when `git grep` or
`git log` cannot run. It does not read the contents of binary files, the author and committer of
a commit, the PR body, or anything pushed without `ship.sh`.

## Gates, ship and install

Strict TDD: one failing test, minimal code, refactor. The failing test goes at the lowest level
that owns the behaviour (`tests/AGENTS.md`, "Where a test belongs"); for a script that is its
test in `tests/scripts/`. No code comments; names and tests carry the intent.

Dead code is strictly forbidden. An item nothing calls is deleted in the same change that
orphans it, never kept "for later". A `pub` item that only tests call is a test seam and must
be on the `SEAMS` list in `tests/dead_code.rs`. `#[allow(dead_code)]` appears nowhere.

Gates, all required before a task is done, wrapped by `scripts/gates.sh`:

```
scripts/scrub-check.sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The gates and CI run the same Rust: `rust-toolchain.toml` pins it for every `cargo` that
rustup provides, and the workflows install the same version. A toolchain bump is one pull
request that changes the channel there, the `toolchain:` of `ci.yml` and `release.yml` (and,
for new components, their `components:`), `rust-version` in `Cargo.toml` (its `major.minor`),
and the locked `rust-overlay` in `flake.lock`, updated without Nix on the host by:

```bash
docker run --rm -v "$PWD":/src -w /src nixos/nix sh -c "git config --global --add safe.directory /src; nix --extra-experimental-features 'nix-command flakes' flake update rust-overlay"
```

`tests/toolchain.rs` fails while `rust-version` differs from the channel's `major.minor`; the
workflows' lines are checked only by the bump's own CI run. The `version` check refuses
a channel change that keeps the locked `rust-overlay` revision, so the update must actually move
the `rust-overlay` revision: on a day the overlay has not moved since the lock, the check
refuses the bump until it does, at most about a day since the overlay commits daily. The `tag`
job of `release.yml` stays on the runner image's `stable` on purpose (why: `.github/AGENTS.md`).
The Nix flake reads the same file: `fromRustupToolchainFile ./rust-toolchain.toml` from its
locked `rust-overlay`, so `flake.nix` names no version and a bump needs no hash, but the overlay
only knows the Rust releases up to its locked commit, hence the lock update in every bump.
`tests/toolchain.rs` fails while `flake.nix` stops reading the file, stops leaving out its
`components` (`.override { extensions = [ ]; }`: the build never runs rustfmt or clippy) or names
any Rust version itself, or the file loses `profile = "minimal"` (the default profile's
`rust-docs` doubled conveyor's macOS `flake` job).

Outside CI (`CI` unset), `gates.sh` runs one at a time per repository, so two ships on one
machine queue instead of loading each other's tests. Before its first gate it takes a lock
(`scripts/gates-lock.sh`) that
every worktree shares: a symlink `harbour-gates.lock` in the git common directory
(`git rev-parse --git-common-dir`), made with `ln -sn`, which fails when the link exists, its
target the holder's pid and worktree. It removes the link when it exits: passed, failed, or on
a hangup, an interrupt or a termination; a TERM sent to the `gates.sh` process alone takes
effect once the running gate ends (bash runs the trap after its foreground child), while a
Ctrl-C reaches the whole foreground group and ends both at once. A second run prints
`==> waiting for the gates of pid <pid> in <worktree> to finish` once per holder, checks again
every second, and gives up after 30 minutes with `gates: gave up after 1800 s waiting for ...`
and exit 1; `HARBOUR_GATES_WAIT` sets that bound in seconds, and a value that is not all digits,
or has more than 9, is refused before the first gate. A holder whose pid no longer runs, or now
runs a command that names neither `gates.sh` nor `ship.sh` (`ps -ww -p <pid> -o command=`; a
`ps` that fails counts as running),
as a run killed with SIGKILL leaves, is taken over with
`==> taking over the gates lock of pid <pid> in <worktree>, whose gates no longer run`. The taker
first makes a guard link named for that pid, `harbour-gates.lock.takeover-<pid>`, so two waiters
never both take one stale lock; a guard whose own holder is gone is taken over the same way, by a
guard named for that pid (`...takeover-<pid>.takeover-<pid>`), never deleted by a run that does
not hold the guard above it. A lock it cannot read (a regular file or a directory at that path)
is waited for like a held one, as `whoever holds <path>`, and the give-up line names the `rm` or
`rmdir` that clears it, the path quoted; a failed claim that then finds the path gone claims
again at once, but only once: after that it waits a second between tries and gives up at the
bound with `could not create the lock link '<path>'`.
Outside a repository, or when the common directory cannot be written, it takes no lock and says
so in one line; the uncontended path prints nothing. `ship.sh` takes the same lock after its
`cargo fmt` (quick, so left outside) and holds it across `gates.sh` and `dev-install.sh`, so the
next run never starts its tests beside a release build; it exports `HARBOUR_GATES_HELD`, its pid
and worktree, and `gates.sh` takes no lock of its own only while that names its parent pid and
the link still names it, so a stale value is never trusted. A worktree branched before the lock
reached `main` runs gates that neither take nor respect it, so it merges `origin/main` before its
next ship.

Outside CI (`CI` unset), a docs-only branch skips most gates: when `scripts/docs-only.sh` finds
that every path between the merge base with `origin/main` and the working tree, untracked files
included, is a `.md` file that does not feed the binary (`scripts/feeds-binary.sh`) and is
outside `changes/` (the fragments) and `.github/` (the script tests read its pull request
template), `gates.sh` runs the scrub check, `cargo fmt --check` and
`cargo test --test readme --test plans --test agents_size`, the only targets that read this repository's Markdown
otherwise, prints `==> skipping the tests: <reason>; CI runs them`, and skips clippy and every
other test. A path ending in `.md` counts whatever it is, so the root `CLAUDE.md` symlink does.
No change, no repository, no `origin/main` or no merge base runs everything. A test that starts
reading another Markdown file of this repository joins that list in `gates.sh`.
Otherwise, outside CI, `gates.sh` leaves out the `scripts` test target (`tests/scripts/`, most
of a warm test run) when `scripts/script-tests-skippable.sh` finds nothing between the merge
base with `origin/main` and the working tree, untracked files included, under `scripts/`,
`tests/scripts/`, `tests/fixtures/`, `.github/`, `changes/`, or in `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md` or `flake.nix`: the paths those tests read from this repository. It then runs
`cargo test --lib --bins` with a `--test` for every other target it finds as `tests/*.rs` or
`tests/*/main.rs` (but `e2e` when tmux is installed, since its own step runs it), then `cargo test --doc`, and prints `==> skipping the script tests: <reason>; CI runs them`. No
repository, no `origin/main` or no merge base runs everything. The skip is safe because the
`shard` legs of `ci.yml` run the tests themselves (`cargo nextest run --locked`), not `gates.sh`, and the `check`
gate over them is required to merge into `main`. A script test that starts reading another path of this
repository puts that path on the list in the script.
With tmux installed, `gates.sh` then runs the `#[ignore]` end-to-end tests last, in a run of
their own: `cargo test --test e2e -- --include-ignored --test-threads=2`. At one thread per core
they lost a timing race in 6 runs of 10 beside other load on the build machine; at 2 threads
they passed every run (`tests/AGENTS.md`).
`tests/scripts/` runs the scripts themselves in a scratch directory and a scratch git
repository, with `cargo`, `gh` and the scripts they call replaced by symlinks to
`tests/fixtures/record-call.sh`, which appends its name and argv to `$CALLS_LOG` and exits 1 for
a call whose argv starts with `$FAILING_CALL` (how a ship test says the branch has no pull
request yet). Where a script reads what `gh` or `curl` prints (`release-tag.sh`,
`release-unfinished.sh`, `formula-version.sh`, `previous-tag.sh`, `homebrew-formula.sh`), the
program is a symlink to `tests/fixtures/reply-call.sh` instead: it records the same line, exits 1
with `$FAILING_STDERR` on stderr for a call whose argv starts with `$FAILING_CALL`, else prints
every file in `$REPLIES` whose name is part of its argv. `tests/scripts/gates_lock.rs` holds a
gate open with `tests/fixtures/held-gate.sh`, which records its call the same way, then, with
`GATE_HELD` set, creates that file and waits until `<GATE_HELD>.release` exists, and exits with
`GATE_STATUS` (it also stops once that file is gone), and `tests/fixtures/held-install.sh`
stands in for the dev install of a ship the same way with `INSTALL_HELD`; so the lock tests wait
on files the test
controls, never on time; with `GATE_ALONE` set it also records, in `<GATE_ALONE>.overlaps`,
every call that found another run inside the gate, which the race of eight runs over a dead lock
and a dead guard reads. Each run there is a `Gates` that starts in a process group of its own,
bounds its lock wait to 10 s by default (0 where a run must give up at once, more where it must
outwait other runs), and, when dropped unfinished, a panic included, kills that group and fails
the test while any of it still runs; a live holder is a `sleep` whose command names `gates.sh`.

Ship with `scripts/ship.sh "<message>"`: `cargo fmt`, gates, `dev-install.sh` (unless skipped,
see Install), commit, push, all under `set -e`, refusing to run on `main`, opening the draft PR
on the first push of a branch (title: the first line of `<message>`; body:
`.github/pull_request_template.md`, or empty when that file is missing). Do not hand-roll the
chain: the interactive shell here is zsh, where `PIPESTATUS` is undefined and
`test "" -eq 0` is true, so a `gates.sh | grep` guard once silently passed a clippy failure into
a commit in conveyor.

Run the wrapper, not the commands: piping `gates.sh | tail` once hid a `cargo fmt --check`
failure and an unformatted commit slipped through. Chaining gates with `&&` and then `;` before
`git commit` does the same.

Install: `scripts/dev-install.sh` builds the release binary and symlinks it as
`~/.local/bin/harbour-dev`; `harbour` on PATH is always the Homebrew release. Never
`cargo install` the crate into a PATH dir. A milestone is done only after the gates pass
**and** `ship.sh` has either run `dev-install.sh` or said why it skipped it, so `harbour-dev`
is the committed code of every branch that changed what the binary is built from.

`ship.sh` skips the install, printing `skipping dev-install: <reason>`, only when both hold
(`scripts/dev-install-skippable.sh`): `harbour-dev` resolves to another worktree's build, and
nothing between the merge base with `origin/main` and the working tree, untracked files
included, is outside `tests/`, `docs/`, `plans/`, `scripts/`, `changes/`, `.github/`,
`.claude/`, an `AGENTS.md` or `CLAUDE.md` at any depth (`src/` included), `PLAN.md`, `README.md`, `LICENSE`
and `.gitignore`. That list lives in `scripts/feeds-binary.sh`, the one definition of "feeds
the binary", which `dev-install-skippable.sh` calls. Everything else counts as feeding the
binary (`src/`, `Cargo.toml`, `Cargo.lock`, the bundled `CHANGELOG.md`, any path nobody
listed), and a missing or dangling
`harbour-dev`, or no `origin/main`, installs too. So a branch of docs, tests, scripts or CI
never takes `harbour-dev` over, and `harbour-dev` then stays the other worktree's build, not
this branch's. Once `harbour-dev` is this worktree's build every ship runs `dev-install.sh`:
cargo's own freshness check makes it a no-op when nothing changed, and it rebuilds after a merge
of `main` that no diff of the commit would show. Embedding a file from one of the listed
directories into the binary means taking that directory off the list in the script.

