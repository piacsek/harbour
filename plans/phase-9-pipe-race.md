# Phase 9: no script dies of SIGPIPE when its reader stops early

## Context

Issue #21. The `check` job failed on `main` for the merge of #19 (run 44695247823) on
`version_check_cargo::the_lock_of_this_repository_passes` with exit status 141, a process killed
by `SIGPIPE`, while the same tree passed in the pull request's run. `scripts/version-check.sh`
runs under `set -euo pipefail` and pipes the whole `Cargo.lock` into `awk` programs that `exit`
as soon as they have their answer; when `printf` is still writing, it gets `SIGPIPE` and
`pipefail` fails the script. This phase fixes the class in every script and adds a guard.

## Verified facts (2026-11-20, `main` at 147e272)

- The race loses every time once what is left to write after the reader stops is larger than a
  pipe buffer. A scratch script under `set -euo pipefail`:

  ```bash
  big="$(printf 'line %s\n' $(seq 1 100000))"
  first="$(printf '%s\n' "$big" | awk '{ print; exit }')"
  echo "reached: $first"
  ```

  never reaches the `echo` and exits 141 on macOS (bash 5.3.9, BSD awk) and in `ubuntu:24.04`
  under Docker (bash 5, `awk` is `mawk`), the runner's image. Below the buffer it is a race:
  bash's `printf` writes in chunks, so `awk` can leave between two of them. This repository's
  `Cargo.lock` is about 50 KB with the `harbour` entry in the middle, which is why CI loses it
  only now and then.
- The real script: `checked_lock` (`tests/scripts/version_check_cargo.rs`) with a lock whose
  `harbour` entry comes first, followed by 3000 entries (about 600 KB), exits 141 on macOS
  every run, where `main` expects 0; a hand-edited line 2 in the same lock exits 141 where the
  check should refuse it naming the line.
- The `version` workflow runs on `pull_request_target`: it checks out `main` and runs `main`'s
  `scripts/version-check.sh` against the pull request's head. So this pull request is judged by
  the unfixed script and can still lose the race in its own `version` run; the fix judges pull
  requests only once it is on `main`.
- A here-string (`<<<`) is written by bash before the reader starts, so it never gets
  `SIGPIPE`; a reader that reads a file (`awk '...' "$errors"`) has no writer to kill.

## Pipelines whose reader can stop before its writer is done

Every pipeline in `scripts/*.sh` and in the inline shell of `.github/workflows/*.yml` was read.
These are the ones whose reader can stop early; the rest read to the end, or read a file or a
here-string.

| Where | Reader | Fed by | Under `set -e` today | Fix |
| --- | --- | --- | --- | --- |
| `version-check.sh` `lock_version` | `awk` `print; exit` | `printf "$lockfile"` (twice) | dies with 141 (the CI failure) | reads to the end, prints once |
| `version-check.sh` `shape` | `awk` `done = 1; exit` (3 rules) | `printf "$lockfile"` | dies with 141 | `done { next }` first, `next` for `exit` |
| `version-check.sh` cargo error | `awk` `found = 1; exit` | the file `"$errors"` | safe (a file) | `!found &&` instead of `exit`, so the guard has no exception |
| `version-check.sh` `refuse_pending_repeats` | `awk` `exit 1` | `fragment_bullets \| joined_bullets` | in `\|\|`, same outcome | `refused { next }`, `END { exit refused }` |
| `fragments.sh` `refuse_strays` | `awk` `print; exit` | `printf "$listing"` | in `\|\|`, same outcome | `!found &&`, prints once |
| `fragments.sh` `refuse_repeats` | `awk` `exit 1` | `fragment_bullets \| joined_bullets` | in `\|\|`, same outcome | `refused { next }`, `END { exit refused }` |
| `release-rewrite.sh` `root_package` | `awk` `bad = 1; exit` (4 places) | a file in `version-check.sh`; `cargo metadata \|` in `release-commit.sh` | `\|\| package=""`, same outcome | `bad { next }` first, `next` for `exit` |
| `previous-tag.sh` | `awk` `print prev; exit` | `{ grep; echo } \| sort -u` | dies with 141 | `!found &&`, prints once |
| `formula-version.sh` | `head -n 1` | `sed <<<"$text"` | dies with 141 | `sed -n 1p` |
| `releases.sh` `release_state` | `head -n 1` | `grep <<<"$published"` | `\|\| true` hides it | `sed -n 1p` |
| `screenshots.sh` `render` | `head -1` (twice per line) | `grep -o "$svg"` | dies with 141 (local tool) | `sed -n 1p` |

`release-commit.sh` runs an `awk` with only a `BEGIN` block, but nothing pipes into it. The
workflows' inline pipes (`tee`, `cut`, `base64 | tr`, `od | tr`, an `awk` without `exit`, a
`grep -m1` on a file into `cut`) all read to the end.

## Decisions

- Readers read to the end. Each fixed program prints exactly what it printed before and exits
  with the same status, so every message stays byte for byte and the existing script tests pass
  unchanged.
- The guard is a test, `tests/scripts/pipe_readers.rs`, not only a rule: it reads every
  `scripts/*.sh` and the `run:` steps of `.github/workflows/*.yml` as shell and fails naming
  the file and line of a pipe into `head`, `read` or a `grep` that stops at a match, and of an
  `exit` outside `BEGIN` and `END` in a single-quoted `awk` program. It is red on `main` naming
  the 19 places above. It flags an `exit` even where the program reads a file: the rule is
  simpler to keep than the exception. `tests/AGENTS.md` carries the rule, its reason and the
  exact list of what the scan cannot see.
- No fragment: nothing here feeds the binary (`scripts/feeds-binary.sh` prints nothing for the
  changed paths).

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The plan, its index and metrics rows; `version_check_cargo.rs` runs the check on a lock whose
   `harbour` entry is far above its end (red: 141), then every reader in `version-check.sh`
   reads to the end.
2. `tests/scripts/pipe_readers.rs` scans the scripts and workflows (red: the remaining places),
   then every other reader in the table reads to the end; the rule in `tests/AGENTS.md`.

## Outcome

- Cause, proven: an `awk` that calls `exit` while `printf` is still writing kills the writer with
  `SIGPIPE`, and `set -o pipefail` turns that into exit 141. A lock with about 600 KB after the
  `harbour` entry makes `scripts/version-check.sh` exit 141 on every run on `main`, on macOS
  (BSD awk) and on ubuntu (`mawk`); this repository's 50 KB lock loses it only now and then,
  as in run 44695247823.
- Fixed: the 19 places in the table, in eight `awk` programs and three `head` pipelines. Every
  message and exit status is unchanged: the existing script tests pass untouched, and both
  reviewers ran the old and the new form of each `awk` program on the same inputs (BSD awk,
  `mawk` and `gawk`) and found the same output and status.
- Guard: `tests/scripts/pipe_readers.rs` reads the scripts and the workflows' `run:` steps as
  shell, with quotes, comments, here-documents and line continuations as text, and never
  panics on what it cannot parse. It finds a pipe into `head`, `read` or a `grep` that stops at a
  match (short and long options, through `command`, on the line after a trailing `|`), and an
  `exit` outside `BEGIN` and `END` in a single-quoted `awk`, `gawk`, `mawk` or `nawk` program.
  Against `origin/main` it names exactly the 19 places; against the branch, none. What it
  cannot see (a program in a variable, a file, double quotes or a here-document; a reader behind
  `xargs`, `env`, `sh -c` or `eval`; `sed q`; a loop that `break`s) is listed under the rule in
  `tests/AGENTS.md`.
- Review: one round, unprimed, clean for both reviewers (no must, no should). Its minor findings
  on the guard (a panic on an unbalanced apostrophe after `awk` in a comment, spellings it
  missed, a `|` in quotes read as a pipe, the overstated coverage sentence) were fixed in step 4
  without touching `scripts/` or `.github/`.
- The `version` workflow judges a pull request with `main`'s copy of the script, so this pull
  request was judged by the unfixed one; the fix protects pull requests once merged.
- No fragment: nothing here feeds the binary, so the merge releases nothing.

## Hurdles

- `ci` the `check` job failed on `main` itself for the merge of #19, on a lock that was right:
  `version-check.sh` died of `SIGPIPE` when `awk` left before `printf` finished writing → code
- `test-seam` the first guard parsed only from the word `awk` onwards and treated every `|` as
  a pipe, so a comment or a quoted string could make it panic or report text; it became a small
  shell lexer → code
