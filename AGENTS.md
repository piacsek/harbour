# AGENTS.md

Notes for agents and humans working on `harbour`. Keep this file short and
factual; verify against the code before treating anything here as live state.

## What this is

harbour: Harbour: where shipping begins. It is a terminal workspace for agent work.
Phase 1 gave it its first interface: with no argument on a terminal it opens a sidebar of agent
sessions beside a live pane; it also answers `--help`, `-h`, `help`, `--version`, `-V`, `version`,
`changelog` and `status` (phase 5: the session counts of every running harbour, for the tmux
status line). Phase 0 scaffolded the guidelines, guardrails, CI and the automatic release
from the `rust_tui` recipe, which descends from conveyor, an earlier project by the recipe's
author; the conveyor history below is that project's. `PLAN.md` is the index of phases and the
backlog; each phase's plan, outcome and hurdles are in `plans/`.

## This repository is public

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

## Layout

The layout names what each file or directory is for; read its functions from the code, and its
rules from the `AGENTS.md` of its directory.

```
src/main.rs      the binary: `--version`/`-V`/`version`, `--help`/`-h`/`help`, `changelog`, `status` (the counts for the tmux status line), the interface with no argument on a terminal, the same with made-up sessions for `--demo` on a terminal, the usage on anything else
src/lib.rs                    the library the binary and the tests share
src/tui.rs                    the terminal: enters and restores it, reads its events, draws the app; the program a session runs (`claude`, or what `HARBOUR_SESSION_COMMAND` names)
src/app.rs                    the app: what is on screen and what each key does
src/session.rs                a session: a command in a PTY, its output parsed into a screen that renders as a widget, its terminal queries answered, its exit status, its screen never under 2 rows or 2 columns (vt100 panics below that); every call into vt100 contained, so a panic gives that session a fresh screen with the modes its program had turned on and a repaint instead of taking harbour down, and the panic hook stays silent for it; writes go through a writer thread so a child that does not read never blocks the interface; ending it (or dropping it) hangs up its whole process group, then kills it, only while the child is unreaped; quitting hangs up every session first and waits once
src/demo.rs                   the made-up sessions of `--demo`: `acme/*` scripts, never real data; the registry they write in the temp dir, removed when dropped
src/registry.rs               Claude Code's session file for a pid (`<CLAUDE_CONFIG_DIR or ~/.claude>/sessions/<pid>.json`): its status and what it waits for, read leniently, ignored when written well before the session started
src/state.rs                  a session's state from its exit and its file: the dot, the waiting reason, the title counts and the help legend, each fitted to the sidebar's width
src/summary.rs                the counts a running harbour publishes for `harbour status` (one publication under a name of its own, pid, time and counter: `<name>.counts` beside a `<name>.lock` it holds, under `$XDG_STATE_HOME/harbour/status/`, else `~/.local/state/harbour/status/`, private, removed on quit, never waiting for a lock; a failed write or a missing file drops the whole publication, and a fresh one is made at most every `RETRY`, 5 s) and the reader that adds up those whose lock is still held
src/help.rs                   what the help view shows: `KEYS`, which the README key table mirrors, and `NOTES` (C-Backspace arriving as C-h, the tmux pattern)
src/keys.rs                   the bytes a terminal sends for a key (legacy encoding), which a session receives
src/mouse.rs                  `route`, a pure function: a gesture belongs to where its button went down and to the session shown then, and ends when another session is shown, and the selected session gets only what its mouse mode asked for (`?9` presses, `?1000` presses, releases and the wheel, `?1002` drags too, `?1003` moves too), with Shift, Alt and Ctrl, in its encoding (SGR, UTF-8 or the legacy bytes), clamped to the pane; an empty pane forwards nothing and ends the gesture
src/changelog.rs              the bundled CHANGELOG.md and the rules a release entry must meet
CHANGELOG.md                  only its `# Changelog` heading on `main`; each release commit writes its sections
rust-toolchain.toml           the one Rust version (rustfmt and clippy with it) that local gates and CI run; see Development
flake.nix, flake.lock         the Nix flake: the binary built from `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` and `src/` with the Rust `rust-toolchain.toml` names, through the locked `rust-overlay`; feeds the binary
scripts/gates.sh              the quality gates; fails loudly, never pipe it through tail
scripts/scrub-check.sh        denylist grep over what a commit would take (see "This repository is public")
scripts/dev-install.sh        release build symlinked as ~/.local/bin/harbour-dev
scripts/ship.sh               gates + dev-install (unless skippable) + stage + scrub + commit (none when nothing is staged and commits are unpushed) + scrub of the outgoing commits and branch name + push, aborts on any failure; the first push of a branch opens the draft PR (commit subject as title, the template as body)
scripts/dev-install-skippable.sh  exit 0 with the reason when `ship.sh` may leave `harbour-dev` alone, else 1 with what needs the install
scripts/script-tests-skippable.sh exit 0 with the reason when `gates.sh` may leave out the `scripts` test target, else 1 with the first path that needs it
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
scripts/screenshots.sh        renders docs/details.png from `harbour --demo` and docs/help.png from `harbour --help`, plus one `shoot` per extra view (keys, pane, question, narrow; truecolor.py converts ANSI)
scripts/sweep-issue.sh        the `issue` job of `sweep.yml`: opens "Sweep due before phase N" with the report once the next phase is a multiple of 5, never twice
docs/                         the screenshots `scripts/screenshots.sh` writes (see "Documentation rule")
changes/                      one fragment per pull request that feeds the binary (`kind:` and the changelog bullets); add-only, see Releasing
plans/                        one versioned plan file per phase, ending with its `## Outcome` and `## Hurdles` (see Working agreements)
plans/metrics.md              time and tokens, one row per phase
tests/           outside-in: `tests/cli.rs` runs the real binary;
                 `tests/scripts/` runs the scripts in a scratch directory and repository, with `tests/fixtures/` standing in for `cargo`, `gh`, `curl` and the scripts they call;
                 `tests/e2e/` runs the real binary in a scratch tmux server (`#[ignore]`, see `tests/AGENTS.md`), with `HARBOUR_SESSION_COMMAND` pointing sessions at `tests/fixtures/agent.sh` (prints what it reads, a line at a time) or `tests/fixtures/raw-agent.sh` (asks for bracketed paste and the mouse, prints every byte it reads) instead of `claude`, or at `tests/fixtures/registered-agent.sh` (writes a busy registry file under `CLAUDE_CONFIG_DIR`, else `HOME`, so it only runs with a scratch `HOME`);
                 `tests/status.rs` runs `harbour status` over summaries a test writes and locks itself, `tests/summary.rs` writes them with `Summary`;
                 `tests/session.rs` and `tests/app/` run sessions in process, as `/bin/sh -c` scripts;
                 `tests/keys.rs` is the table of the bytes each key sends, `tests/mouse.rs` the table of what a session receives for each mouse event, mode, gesture and modifier;
                 `tests/tui.rs` covers leaving the terminal (mouse capture and bracketed paste off) and the panic hook;
                 `tests/readme.rs` pins the README shape (see "Documentation rule");
                 `tests/source_size.rs` fails while any Rust file under `src/` or `tests/` is longer than 700 lines, naming it;
                 `tests/toolchain.rs` fails while the `toolchain:` of `ci.yml` or `release.yml` differs from the channel of `rust-toolchain.toml`, or the `components:` of `ci.yml` or of the `build` job of `release.yml` from its `components`, or `rust-version` in `Cargo.toml` from its `major.minor`, or the `tag` job of `release.yml` loses `RUSTUP_TOOLCHAIN: stable`;
                 `tests/plans.rs` fails while a plan file is missing from the `PLAN.md` index, a phase plan has no `## Outcome` or a malformed `## Hurdles`, a sweep phase's `## Outcome` has no `### Sweep`, or a phase has no `plans/metrics.md` row;
                 `tests/dead_code.rs` fails while a `pub` item under `src/` has its name nowhere else in `src/` (a name scan, not a resolver) and is not on its `SEAMS` list, while a seam on that list is no longer declared or used by a test, or while a `dead_code` attribute sits anywhere under `src/` or `tests/`, naming each;
                 a test file that outgrows the limit becomes a folder (`tests/<name>/main.rs` plus one file per topic), so it stays one binary
tests/AGENTS.md  the testing traps hit so far
.github/AGENTS.md CI: what the workflows run and why (`check`, `audit`, the flake build, the `version` check, the release queue, the tap, `stable`), the `main` ruleset, branch cleanup, the sweep issue, the issue forms and Discussions; the release internals, the first release from zero tags included
.github/pull_request_template.md  `## Summary` and `## Changes`: the body `ship.sh` opens a draft PR with
.github/ISSUE_TEMPLATE/       the bug report and feature request forms, and `config.yml` sending questions to Discussions
.github/CONTRIBUTING.md       where a newcomer starts: points here, to `scripts/gates.sh`, `scripts/ship.sh` and the fragment rule, restates none of them
.claude/skills/sweep/          the `sweep` skill: runs `scripts/sweep-report.sh`, finds repeats and trends, proposes what each becomes, writes `### Sweep` on the user's yes
.claude/skills/adversarial-review/ the `adversarial-review` skill: attack the repository, report what is bad, file the fixes as labelled issues on request; `drive-demo.sh` drives `harbour --demo` in a scratch tmux server at a given size
```

## Where the rest lives

This file holds the rules and agreements. The facts about each directory live in an `AGENTS.md`
next to the code; read the one for a directory before changing anything in it.

| File | What it holds | Read it |
| --- | --- | --- |
| `tests/AGENTS.md` | the testing traps hit so far | before writing or debugging a test, a shim or a launcher of the binary |
| `.github/AGENTS.md` | CI: what the workflows run and why (`check`, `audit`, the flake build, the `version` check, the release queue, the tap, `stable`), the `main` ruleset and its required checks, the unguarded `v*` tags, branch cleanup, the sweep issue, the issue forms and Discussions; the release internals: the first release from zero tags, every refusal of the `version` check, the release run, recovering a failed or stuck run, pins and permissions | before touching `.github/` (the issue forms and `CONTRIBUTING.md` included), a required check or a release script, or when a release run fails |

## Behaviour that is easy to break

- **`--help`/`-h`/`help` and `--version`/`-V`/`version`** print to stdout and exit 0; anything
  else prints the usage to stderr and exits 1, and so does no argument when stdin or stdout is not
  a terminal (with both on a terminal it opens the interface). The usage names `--demo` and says
  that no argument on a terminal opens the interface; `tests/cli.rs` pins it. The Homebrew
  formula's `test` block (`--version`, and `usage: harbour` on an unknown argument) and the `flake`
  and `stable-build` jobs (`--version`) rely on both.
- **`status`** prints one line of tmux status-line text (the blocked, working, idle and unknown
  counts of every running harbour) or nothing at all, and always exits 0, whatever it finds on
  disk: tmux runs it from `status-right` every `status-interval` and shows whatever it prints.
  `tests/status.rs` pins it.
- **`changelog`** prints the bundled `CHANGELOG.md` to stdout and exits 0: the heading alone in a
  build from `main`, every release in a release build. `tests/changelog.rs` holds the rules a
  release entry must meet, and `the_changelog_has_a_real_entry_for_the_crate_version` checks that a
  release commit's file has the entry of its own version.

## Known limits

Accepted limits a reviewer treats as decided, one line each, with where they are written down.
Pointing reviewers here is not priming, since the list is in the repo. A limit joins it when a
review finding is declined for good; a reviewer may still report one as must when it shows new
impact.

- **The `version` check judges "feeds the binary" by path only**, can be pointed by cargo at
  manifests outside its copy, and refuses a fragment name git would quote
  (`.github/AGENTS.md`, `version.yml`).
- **`check` builds and tests on Linux only**: the tests never run on macOS, and a macOS-only
  break of the release build itself fails in the release run; the flake's macOS leg catches a
  plain macOS build break (`.github/AGENTS.md`).
- **A session that exited by itself leaves behind what ignores hangup**: once its command is
  reaped its pid can be reused, so harbour never signals its group again, and a background
  process that ignores hangup keeps running after its row is removed or harbour quits, as in any
  terminal (`src/session.rs`).
- **No tag ruleset guards `v*` tags**: the owner can still push or delete one by hand
  (`.github/AGENTS.md`).
- **A Nix outage blocks merges that touch the flake's inputs once the ruleset requires
  `flake-ok`**: a Nix, nixpkgs or `cache.nixos.org` outage then holds them until the ruleset is
  disabled; requiring it is the owner's step by hand (`.github/AGENTS.md`, the flake build, has
  the command that shows whether it is required).
- **The session registry is read, and the counts for `harbour status` written, on the draw
  thread**: a Claude config directory or a state directory on slow storage (a network home)
  delays keys by the time those reads take, at most every 500 ms, by the time a write takes,
  only when a count changed, and by two file checks or one fresh publication at most every 5 s
  (`plans/phase-3-status-dots.md`, `plans/phase-5-tmux-status-line.md`).
- **The release attestation records the `main` commit that started the run, not the release
  commit the tag points at**: its subjects are the right tarballs, but the source commit it
  names is the merge (`.github/AGENTS.md`, "What the attestation proves").

## Documentation rule

`README.md` is minimal and holds only what a user needs, in this order and nothing else: the
`# harbour` heading, the one line "Harbour: where shipping begins.", then `## Installation`, `## Usage`
and `## Development`, with no subsection. `tests/readme.rs` fails when that shape changes, when
the tap, the Nix or the source install line (built from `stable`) goes, when the usage stops showing `--version` and `--help`, or when the key table stops mirroring `KEYS` of `src/help.rs`, the keys the help view shows.
Feature depth belongs in the `AGENTS.md` files, not there.
**Every visible change to the main interface shows in `docs/details.png`**: a new or changed
line, glyph, bar, title, column or footer element. The `details` shot of `scripts/screenshots.sh`
takes it from `harbour --demo`, `README.md` embeds it right below the description, and
`tests/readme.rs` pins it there as the README's only image. Extend the demo (`src/demo.rs`)
until the change appears on screen, rerun the script, and commit the image in the same pull
request; a reviewer reports a missing or stale screenshot as a should. The script also shoots
`docs/help.png`, the output of `harbour --help`, which the README does not embed. Run `scripts/screenshots.sh` (needs tmux, `brew install
charmbracelet/tap/freeze`, Google Chrome or chromium, and the FiraCode Nerd Font in
`~/Library/Fonts`) and commit the new images. Freeze lays out an SVG with the font embedded;
headless Chrome rasterises it, because freeze's own PNG output cannot draw Nerd Font glyphs.
The script adds Menlo (DejaVu Sans Mono on Linux) as the SVG's fallback font family: FiraCode
lacks `✗ ⊘ ▸`, and Chrome's default fallback is wider, so in conveyor every row holding one
shifted and the borders jogged. Without the font file the SVG names the family alone and Chrome
falls back; without Chrome freeze writes the PNG itself; `CHROME` names another browser binary.
The script runs the release binary in a scratch tmux server with `HOME` a tempdir, through `env`
with `XDG_CONFIG_HOME` and `XDG_STATE_HOME` unset, since both outrank `HOME`. Demo data looks
real (SHAs like `8c1f2a7d…`, not `33333333`). Do not screenshot real data: this repository is
public.

Whenever behaviour changes, check that `README.md` is still true: an argument, a subcommand, an
install path. Do not add prose for a feature; if it needs explaining, the binary's help should
carry it.

## Working agreements

All guidelines live in `AGENTS.md` files, versioned with the code (the root one for rules and
agreements, one per directory for that directory's facts); nothing binding lives only in an
agent's private memory. When the user says "update the guidelines", edit the `AGENTS.md`
nearest the code the guideline is about.

- **"Work autonomously" means the whole road to a release, without pausing**, not even for
  the retro: its outcome is written into the phase's plan file in step 4 and the user reads it
  there. When the user says it (or "autonomous", "march towards release"), assume this flow and
  ask only when a decision is genuinely theirs:
  1. Plan in `plans/phase-N-<topic>.md` (next free N in `PLAN.md`), in a fresh worktree, with
     its row in `PLAN.md` and in `plans/metrics.md`. When N is a multiple of 5, the phase opens
     with the sweep (see "The sweep"). The first ship also backfills the previous rows of
     `plans/metrics.md` (see "Time and tokens").
  2. `/my-tdd autonomous`: one task per red-green-refactor cycle, each shipped with
     `scripts/ship.sh` (gates, dev install, commit, push, draft PR on the first push).
  3. `/pr-review` the PR (see "Review before ready": what counts as must or should, when to
     redesign, where the reviewer runs the gates, who fixes). Fix every must and should
     finding, TDD like any change, ship once per round, and review again. Minor findings are
     fixed when cheap, else listed in the PR body. The loop ends only on a round with no
     must/should findings, and that round is **unprimed**: the reviewers get the PR, the repo
     and this file, never the list of earlier findings or what was decided about them. Never
     post the review on the PR.
  4. The plan file's `## Outcome` and `## Hurdles` are filled, its `plans/metrics.md` row
     holds what is known by now (see "Time and tokens"), and, when the branch feeds the binary,
     its fragment `changes/<branch>.md` is on it (no version bump, no `CHANGELOG.md` edit). Re-read
     the PR title and body, fix what went stale, `gh pr ready`, wait for CI green.
  5. Squash-merge, the only method `main` allows. Watch the `release` run the merge starts to
     green; with a fragment, check the release holds the six assets and the tap formula names
     the new version (see "Releasing"). Never tag by hand.
  6. Remove the worktree and report what shipped, with the release link and the times the
     branch could not record (see "Time and tokens").
  Stop and ask instead of proceeding when: a gate fails for a reason the change did not
  cause, CI fails twice on the same step, a review finding needs a product decision, or the
  next step would force-push, push to `main` or delete a release.
- **Cadence.** Outside the autonomous flow above, work runs under `/my-tdd autonomous`: one
  branch per phase (`phase-N-<topic>`), one commit per task through `scripts/ship.sh` (the fixes
  of a review round are the exception: one commit per round, see "Review before ready"), no
  pause between tasks, a pause only for the end-of-phase retro. Retro outcomes go to the plan
  file (`## Outcome` for facts and decisions, `## Hurdles` for what got in the way), its
  `plans/metrics.md` row, and here (rules).
- **Hurdles.** Every plan file ends with `## Hurdles`: one bullet per thing that got in the way,
  `` - `<tag>` <what happened> → <where it went> ``. The tag is one of `review-loop`, `ci`,
  `tooling`, `test-seam`, `process`, `api-shape`, `design`; where it went is `code`, `backlog`,
  `known limit`, `none` or `` rule in `<path>` `` (a file that exists). A phase with nothing to
  say writes `- none recorded`. `tests/plans.rs` fails on any other shape.
- **The sweep.** A phase whose number is a multiple of 5, from phase 5, starts with a sweep
  before its own plan: read the `## Hurdles` of every phase since the last sweep (since phase 0
  for the first) and `plans/metrics.md`, and look for a tag that keeps coming back, a `→ none`
  that happened twice, and the review rounds and tokens drifting up. Each finding becomes a rule
  in the nearest `AGENTS.md`, a backlog item or an issue, or is declined with its reason; the
  sweep's outcome is the `### Sweep` section that opens that phase's `## Outcome`
  (`tests/plans.rs` fails without it). `scripts/sweep-report.sh [<first phase>]` gathers the data
  (hurdles counted and grouped by tag, every `→ none`, the metrics rows and their empty cells,
  from the phase after the last sweep); the `sweep` workflow opens an issue "Sweep due before
  phase N" with that report once phase N−1 is on `main`. The report only counts: telling that
  two hurdles are the same problem, and what becomes of each, is the sweep's judgment, which the
  `sweep` skill (`/sweep`) walks through.
- **Time and tokens.** Every phase has one row in `plans/metrics.md`, written with the outcome,
  so it is on the branch before `gh pr ready`; its Notes cell carries what a number needs to be
  read right (a session shared between phases, a running total). The orchestrator (the main
  session) owns the numbers and hands them to the builder to write: the builder cannot see the
  reviewers' usage. It records only what is known by then:
  - Time: build start (the worktree's creation, the oldest entry of `git reflog show
    --date=iso <branch>`, its last line), draft PR opened, the end of the clean review
    round, the wall time from start to that end, and the number of review rounds. Absolute
    times with a timezone (`2026-11-17 04:56Z`), never "about an hour".
  - Tokens: one figure per agent (builder, each reviewer, the orchestrator), each marked
    measured or estimated with its source; what nobody can read is `unknown`, never a guess. A
    sub-agent's completion notice carries `subagent_tokens`, which is not a running total across
    resumes (in an earlier project a resumed builder's second notice was lower than its first; whether
    cached input counts like fresh input is not known), so the orchestrator copies the figure of
    every notice an agent sent; the Codex reviewer and the orchestrator's own session report none from
    the inside.
  - After the merge: a branch cannot record its own ready, merge or release, so its row leaves
    those cells empty, the report of step 6 gives them, and the first ship of the next phase
    writes them into every row still missing them. Both read them from GitHub (`gh api
    repos/<owner>/<repo>/issues/<N>/timeline --jq
    '.[]|select(.event=="ready_for_review").created_at'`, `gh pr view <N> --json mergedAt`,
    `gh release view <tag> --json publishedAt`), and the total from build start to release, or
    to merge with "no release" for a branch that feeds nothing into the binary.
- **One worktree per branch, by default.** Start every phase or fix in its own worktree off
  `origin/main`: `git worktree add -b <branch> .claude/worktrees/<branch> origin/main`, then
  `git branch --unset-upstream` so the first `ship.sh` push creates `origin/<branch>` instead of
  tracking `main`. Work, gates and ships run from inside it; the main checkout stays on its own
  branch, so two phases (and two agent sessions) never share an index or a `target/`.
  `.claude/worktrees/` is gitignored. Each worktree builds its own `target/` from cold the first
  time. `harbour-dev` points at the worktree that last ran `dev-install.sh`, so re-run it in the
  worktree you are testing. Remove the worktree (`git worktree remove`) once its PR is merged.
- **TDD is never skipped.** Every change to `src/` starts with a failing test, run and seen
  red, before the production code is touched; then the smallest change that goes green, then
  refactor. This holds for one-line fixes, security patches, patch releases and "obvious"
  changes alike. Writing the test and the fix in the same edit is not TDD, even when the test
  is then backed out to prove it fails. Show the red run in the update.
- **Draft PR first.** The first ship on a branch opens the draft PR assigned to the user,
  with the commit subject as its title and `.github/pull_request_template.md` as its body;
  every later commit lands on that PR. The subject is a starting point: before review, rewrite
  the title in plain language that says what the change does for whoever uses it, and fill the
  body. `## Summary` holds at most 3 sentences, split across 2 lines with a blank line between
  them; `## Changes` holds the rest. After every edit of a PR body, fetch it back
  (`gh pr view <N> --json body`) and compare it with what you wrote: an edit can upload the
  wrong file. Before merging, re-read the PR title and body and fix what went stale without
  rewording the user's edits.
- **Screenshots in PRs.** A PR with a visible change embeds a screenshot that shows that
  change in its body, as a commit-pinned `raw.githubusercontent.com` URL. A change to the main
  interface goes into `docs/details.png` once it exists (see "Documentation rule"); for a view
  that one cannot show (another pane, a dialog, a narrow width, a key pressed), add a `shoot`
  line to `scripts/screenshots.sh` for it, commit the image under `docs/` and embed it too.
  Before `gh pr ready`, rerun `scripts/screenshots.sh`; when an image changed, commit it and pin
  its URL to that commit.
- **Diagrams in PRs.** A PR that reorganizes the codebase or introduces a new flow carries a
  Mermaid diagram in its body, in a fenced `mermaid` block that GitHub renders. Show the
  shape of the change: the modules and what moved between them, or the steps of the flow and
  who calls whom. For a reorganization, a before and an after. Keep each diagram small enough
  to read without scrolling, and leave it out when the change has no structure to draw.
- **Review before ready.** Marking a PR ready for review means running the
  [`/pr-review`](https://github.com/piacsek/dotfiles/blob/main/.claude/skills/pr-review/SKILL.md)
  skill on the PR first, and acting on what it finds. It runs a Claude and a Codex reviewer
  over the diff and merges their findings; run it from the main session, because the skill
  launches Codex there. Give both reviewers the intent of the change, point them at this
  file and tell them to read its "Known limits" before reporting, have the Claude reviewer run
  `scripts/gates.sh` itself rather than trust the claim that the gates pass, and forbid edits,
  commits, and any write to GitHub. When the skill asks whether to post the review on the PR,
  answer no. `gh pr ready` comes after the findings are addressed, not before.
  - **Severity by reachability.** A finding is must or should only when the user hits it while
    using harbour, the release pipeline hits it in normal operation, or a pull request could
    execute code or reach a token or secret. A bypass that needs a deliberately forged file,
    which the owner would still have to merge, is minor: fixed when cheap, else listed in the
    PR body. The reviewers are given this definition.
  - **Two strikes, then redesign.** When a second round finds another hole in the same
    mechanism, stop patching it: step back and pick a simpler design that removes the class of
    finding, and ask the user when that changes behaviour. In conveyor, one pull request took
    five rounds of patches to the `version` check and converged only once the check asked cargo
    instead of parsing lines.
  - **The gates run beside the branch, not in it.** The Claude reviewer runs
    `scripts/gates.sh` in its own detached worktree of the PR head (`git worktree add --detach
    <path> <sha>`, removed afterwards), never in the branch's worktree, so fixes do not wait
    for its run.
  - **Fixes go back to the builder.** The fixes of a round go to the agent that built the
    branch, with its context intact, not to a fresh one, and land as one `ship.sh` per round
    of findings rather than one per finding: the review-round exception to "one commit per
    task". Each fix that touches `src/` still starts from a failing test.
- **Merging and releasing.** Squash-merge the phase PR into `main` when CI is green, then
  watch the `release` run the merge starts (see Releasing): no bump, no `CHANGELOG.md` edit, no tag
  by hand. Squash is the only merge method the repository and the `main` ruleset allow (see
  `.github/AGENTS.md`). The squash commit on `main` takes GitHub's default: the pull request
  title followed by `(#N)` (a pull request of a single commit gets that commit's subject
  instead), with the per-task commit messages listed in its body; the per-task commits stay
  readable on the pull request. Never push to `main` directly, and never force-push.
- **Catching up with `main` on a pushed branch is a merge, not a rebase.** A rebase rewrites
  the pushed commits and the next `scripts/ship.sh` push is refused; the no-force-push rule
  then leaves only `git reset --hard origin/<branch>`, `git merge origin/main` and a
  cherry-pick of the new work. The squash into `main` leaves the merge commit behind anyway.
  Squash is the only method partly for this reason: a rebase-merge, which replays the branch's
  own commits and drops its merge commits, could land a tree the `version` check never judged.
- **Two binaries.** `harbour` on PATH is the Homebrew release; `harbour-dev` is a working
  tree (`scripts/dev-install.sh`). Never `cargo install` the crate into a PATH directory. A
  build from a working tree (`harbour-dev`, CI) prints `harbour 0.0.0` for `--version`: that is
  expected, only a release commit carries a real version.
- **Plan before code, plans in the repo.** Each phase starts from a written plan, re-planned
  after the previous retro, with its "Verified facts" checked against live tools before use.
  Plans are versioned here: write each one to its own file under `plans/` (for example
  `plans/phase-1-<topic>.md`) and commit it with the first ship of the phase, with empty
  `## Outcome` and `## Hurdles` headings at its end that the phase fills before `gh pr ready`.
  Never leave a plan only in an agent's private plan directory or memory. `PLAN.md` stays the
  index of phases and the backlog; `tests/plans.rs` fails while a plan file is not linked from
  it, a phase plan has no `## Outcome`, or a phase has no `plans/metrics.md` row. A plan that
  adds logic over external state (check states, merge states, write results) carries a table of
  the states and what the user sees for each, and that table is reviewed before any code is
  written: in conveyor, three review rounds in a row each found states such a table would have
  listed up front.

## Development

Strict TDD: one failing test, minimal code, refactor. Outside-in: start from the binary, drop
to unit tests only for pure helpers. No code comments; names and tests carry the intent.

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

`tests/toolchain.rs` fails while any of the first four differs, and the `version` check refuses
a channel change that keeps the locked `rust-overlay` revision, so the update must actually move
the `rust-overlay` revision: on a day the overlay has not moved since the lock, the check
refuses the bump until it does, at most about a day since the overlay commits daily. The `tag`
job of `release.yml` stays on the runner image's `stable` on purpose (why: `.github/AGENTS.md`).
The Nix flake reads the same file: `fromRustupToolchainFile ./rust-toolchain.toml` from its
locked `rust-overlay`, so `flake.nix` names no version and a bump needs no hash, but the overlay
only knows the Rust releases up to its locked commit, hence the lock update in every bump.
`tests/toolchain.rs` fails while `flake.nix` stops reading the file or names any Rust version
itself, or the file loses `profile = "minimal"` (the default profile's `rust-docs` doubled
conveyor's macOS `flake` job).

Outside CI (`CI` unset), `gates.sh` leaves out the `scripts` test target (`tests/scripts/`, most
of a warm test run) when `scripts/script-tests-skippable.sh` finds nothing between the merge
base with `origin/main` and the working tree, untracked files included, under `scripts/`,
`tests/scripts/`, `tests/fixtures/`, `.github/`, `changes/`, or in `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md` or `flake.nix`: the paths those tests read from this repository. It then runs
`cargo test --lib --bins` with a `--test` for every other target it finds as `tests/*.rs` or
`tests/*/main.rs`, then `cargo test --doc`, and prints `==> skipping the script tests: <reason>; CI runs them`. No
repository, no `origin/main` or no merge base runs everything. The skip is safe because the
`check` job of `ci.yml` calls `cargo test --locked` itself, not `gates.sh`, and `check` is
required to merge into `main`. A script test that starts reading another path of this
repository puts that path on the list in the script.
`tests/scripts/` runs the scripts themselves in a scratch directory and a scratch git
repository, with `cargo`, `gh` and the scripts they call replaced by symlinks to
`tests/fixtures/record-call.sh`, which appends its name and argv to `$CALLS_LOG` and exits 1 for
a call whose argv starts with `$FAILING_CALL` (how a ship test says the branch has no pull
request yet). Where a script reads what `gh` or `curl` prints (`release-tag.sh`,
`release-unfinished.sh`, `formula-version.sh`, `previous-tag.sh`, `homebrew-formula.sh`), the
program is a symlink to `tests/fixtures/reply-call.sh` instead: it records the same line, exits 1
with `$FAILING_STDERR` on stderr for a call whose argv starts with `$FAILING_CALL`, else prints
every file in `$REPLIES` whose name is part of its argv.

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
`.claude/`, an `AGENTS.md` at any depth (`src/` included), `PLAN.md`, `README.md`, `LICENSE`
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

## Releasing

The pipeline picks the version; a pull request never does. `main` carries the constant
`version = "0.0.0"` in `Cargo.toml` and a `CHANGELOG.md` that holds only its `# Changelog`
heading; a pull request edits neither. The version lives in the `v*` tags, and each release's
`CHANGELOG.md` lives on its release commit, which only its tag reaches, so a build from `main`
prints the heading alone for `harbour changelog` and a release build prints every release.

**The first release starts from zero tags.** No `vX.Y.Z` tag means "never released": the
scripts count from 0.0.0 and every fragment on `main` is pending: a `feat` fragment gives
0.1.0, a patch kind (`fix` or `internal`) gives 0.0.1. The phase 0 fragment is `kind: fix`, so
the first release is v0.0.1. The design and its tests are under "Releasing internals" in
`.github/AGENTS.md`.

**A pull request says what kind of change it is in a fragment**, one added file
`changes/<branch>.md` (a branch name holding `/` uses `-`):

```markdown
kind: fix

- `harbour --help` lists every subcommand.
```

- `kind` is `fix` (patch), `feat` (minor) or `internal` (patch). The bullets are the changelog
  text, written for users, one bullet each; an indented line continues a bullet. `internal` is
  for a change nobody can observe in the binary (a refactor, a dependency bump) and may have no
  bullet; a release made only of those reads `- v<version>: internal changes only.`
- **No majors.** No kind produces one; a breaking change is a decision made by hand later.
- **A fragment is required exactly when the pull request touches what feeds the binary**
  (`scripts/feeds-binary.sh`, the same list `ship.sh` uses for the dev install). `flake.nix` and
  `flake.lock` feed the binary on purpose: they decide what a Nix user runs, and only a release
  moves `stable` to them. Such a pull request is always released, a refactor as an `internal`
  patch. One that touches only tests, docs, plans, scripts, CI, guidelines or agent skills
  (`.claude/`) needs none and releases nothing.
- **`changes/` is add-only.** A fragment is never edited, renamed or deleted once it is on
  `main`: "pending" means "not in the tree of the last tag", so a changed released fragment
  stops every release and a reused name can skip one silently. Put a correction in a new file.
- `tests/scripts/next_version.rs` runs `scripts/next-version.sh` on this repository, so a
  malformed committed fragment fails the gates (it reads `HEAD`, so locally only after the
  commit; CI always sees it).

**The `version` check enforces these rules on every pull request into `main`.** It refuses
one that sets the version in `Cargo.toml` or `Cargo.lock` or leaves either in a shape the
release cannot rewrite, adds or changes `CHANGELOG.md`, edits, deletes or renames a fragment,
adds a malformed or repeated one, changes the Rust channel of `rust-toolchain.toml` without
moving the locked `rust-overlay` of `flake.lock`, or feeds the binary without one, naming the
file and what to do instead. It is required once `main` holds it: rerunning `tui_repo` after the
phase 0 merge adds it to the ruleset (`.github/AGENTS.md`). The full list, how it reads a pull request without running
it, the release run, and what to do when a run fails or a tag can never build are under
"Releasing internals" in [`.github/AGENTS.md`](.github/AGENTS.md).

**A revert keeps the fragment.** A pull request that reverts a merged one keeps that pull
request's fragment (GitHub's Revert button deletes it, which the check refuses, since
`changes/` is add-only) and adds a new fragment for the revert itself.

**After a merge**, watch the run (`gh run list --workflow release.yml --limit 3`, then
`gh run watch <id> --exit-status`). For a pull request with a fragment, check that the release
holds the six assets, that the tap formula names the new version, and that `Cargo.toml` on the
`stable` branch (what the Nix flake installs) names it too:

```bash
gh release view --json tagName,assets --jq '.tagName, .assets[].name'
gh api repos/piacsek/homebrew-tap/contents/Formula/harbour.rb -H 'Accept: application/vnd.github.raw+json'
gh api 'repos/piacsek/harbour/contents/Cargo.toml?ref=stable' -H 'Accept: application/vnd.github.raw+json'
```

Never push or move `stable` by hand: only the `stable` job of the release run moves it, after
`stable-build` built the flake at the tag.

A cancelled `release` run is normal (merges in quick succession queue up); a red one is not: see
"When a run fails or a job is stuck" under "Releasing internals" in `.github/AGENTS.md`.

Never push a `v*` tag by hand: tags come only from the `tag` job of the release run. Nothing
enforces that for the owner; why no tag ruleset does is in `.github/AGENTS.md`.

Never move or delete a tag that has a release: the next run would rebuild and republish it
and rewrite the tap formula for that old version. If a released version is wrong, release a
fix on top.

Semver: `feat` for a new subcommand, flag or behaviour a user sees (minor), `fix` for a fix
(patch), `internal` for the rest (patch); no majors for now.
