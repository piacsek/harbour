# Phase 7: a faster macOS flake build

## Context

Issue #17. `flake (macos-latest)` is the slowest job of `ci` and decides how long a run takes.
Most of its `nix build` step fetches and unpacks what stays the same until `flake.lock` or
`Cargo.lock` changes, and part of it builds clippy and rustfmt, which the package never runs.
This phase caches the Nix store between runs and builds the flake with a toolchain that leaves
both out. The owner also asked for one rule: read the pull request's comments before it is
marked ready and again before it merges.

## Verified facts (2026-11-20, `main` at fe8aaf5)

- Run 44682655741 (the pull request run of phase 4's branch, PR #14), from its job logs:

  | Leg | `nix build` | Install Nix | Fetched | Derivations built |
  | --- | --- | --- | --- | --- |
  | Linux | 55 s | 4 s | 259 paths, 226.5 MiB download, 813.0 MiB unpacked | 108 |
  | macOS | 105 s | 29 s | 262 paths, 288.7 MiB download, 1.5 GiB unpacked | 121 |

  Both legs built the `clippy`, `clippy-preview`, `rustfmt` and `rustfmt-preview` derivations.
- `nix-community/cache-nix-action` v7.0.2: the tags `v7` and `v7.0.2` both point to commit
  `7df957e333c1e5da7721f60227dbba6d06080569`. It runs on node24 with `post-if: success()`, so a
  failed job never saves. By default it caches `/nix` only, and on restore it merges the cached
  Nix database into the runner's. Its `save` input defaults to `true`; garbage collection runs
  only when `nix` and `save` are on, only when the primary key missed, and only down to the
  `gc-max-store-size*` given. Purging needs `actions: write` ("For purging, the workflow or the
  job must have the permission `actions: write`", its README) and only touches caches of the
  current `GITHUB_REF`. It needs Nix 2.24 or later.
- GitHub scopes a cache to the ref that saved it: a run can restore caches of its own ref and
  of the default branch (for a pull request, also its base). `main` can never restore a cache a
  pull request saved. Restoring and saving use the runtime token of the job, not
  `GITHUB_TOKEN`, so they need no `permissions:` entry; a fork's pull request can restore and,
  with this design, never saves.
- `rust-overlay` at the locked revision `2ff7d39b`: `fromRustupToolchain` returns
  `pkg.override { extensions = components; inherit targets; }`, where `pkg` is a profile built
  with `lib.makeOverridable`, so the result takes `.override { extensions = [ ]; }` again and
  keeps its targets. The `minimal` profile is `cargo`, `rustc` and `rust-std`.
- `ci.yml` grants `contents: read` at the top and nothing per job. Its `flake` job runs only
  when `paths` says a flake input changed; `ci.yml` is one, so this pull request runs both legs.

## States

| State | What happens | What a run looks like |
| --- | --- | --- |
| Hit: a key for this OS, `flake.lock` and `Cargo.lock` exists on `main` | `/nix` is restored, `nix build` finds the toolchain, the crate tarballs and nixpkgs in the store and compiles only what is not there | faster; the post step saves nothing |
| Miss with an older key of this OS on `main` (a lockfile changed) | the newest `nix-<OS>-` cache is restored; what the new lock names and the store lacks is fetched | between a hit and today; on `main` the post step collects garbage above the cap and saves a new key |
| Miss with no cache at all (first run, or every key evicted) | nothing is restored | as slow as today; on `main` the post step saves the first key |
| A pull request, same repository or fork | restores `main`'s cache, never saves | as a hit or a miss above; the cache of `main` is untouched |
| The repository's cache over 10 GB, or a key unused for 7 days | GitHub evicts the least recently used keys | the next run misses and, on `main`, saves again |
| The cache service down or answering with an error | the toolkit warns and the step goes on as a miss | as slow as today, never wrong |
| A failure while processing a restore (merging the cached store database) | the step fails, so the leg and `flake-ok` fail | red, never a wrong binary; rerun the job |
| A failure while saving (the post step on `main`) | a warning only; nothing is saved | the run stays green; the next flake-input push to `main` saves |
| Two merges about two minutes apart, the first changing a lockfile | `cancel-in-progress` on `main` cancels the first run before its post step saves the new key, and the second run may skip the flake job when it changes no flake input | pull requests keep restoring the older key until the next flake-input push to `main`: slower, never wrong |
| `flake.nix` changed without a lockfile change | the old key hits; Nix builds what the new expression needs | correct, slower than a hit, on every run until a lockfile changes: the action saves only on a miss of the primary key |

## Why a restored store cannot change the binary

- The guarantee is who writes the cache: what the action restores is `/nix` as a `main` push
  left it, and only `main` pushes write it. A pull request can read it but never write it, so no
  pull request can put a path into the store a later `main` build reads.
- Nix does not re-hash a restored path: the action merges the cached store database into the
  runner's, so its paths count as registered and are trusted as they are.
- Every store path is named by a hash of its inputs: a derivation's path hashes its builder,
  its arguments and the paths of everything it depends on, the harbour source fileset included.
  A changed source, lock or toolchain gives other paths, which the restored store does not have,
  so Nix builds them.
- What Nix downloads itself (nixpkgs, `rust-overlay`, the toolchain and crate tarballs) is
  fixed-output: checked against the hash `flake.lock`, `Cargo.lock` or the overlay's manifest
  declares, at fetch time. That check covers downloads only, not restored paths.

## Decisions

- **Save only on a `push` to `main`**: `save: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}`.
  Pull requests read `main`'s cache and never write one.
- **No purge.** GitHub's own eviction (7 days unused, least recently used above 10 GB) is
  enough to remove old keys, and purging needs `actions: write`, which a separate job gated on
  the push event could hold but this design then needs nowhere: no job gains a permission.
- **Garbage collection on, with headroom**: `gc-max-store-size-linux: 3G`,
  `gc-max-store-size-macos: 5G`, about twice a build's store estimated from the logs, to be
  checked after the merge. `keep-outputs = true` in the Nix configuration keeps the build-time
  closure of `./result`, so a collection drops leftovers first. The README warns that a
  collection can still evict flake inputs (nixpkgs' source); with the cap above one build's
  store that happens only after several lockfile changes, and costs a fetch, never a wrong build.
- **The key is the issue's**: `nix-${{ runner.os }}-${{ hashFiles('flake.lock', 'Cargo.lock') }}`,
  restore prefix `nix-${{ runner.os }}-`. A change to `flake.nix` alone hits the old key, which
  is slower, never wrong.
- **No `continue-on-error`** on the cache step: an outage is already a miss, and a crash should
  be seen.
- **The cache can only be measured after the merge**: the first `push` run on `main` saves it.
  The pull request records the toolchain change's times; the cache's times and the "drop part 1"
  decision of the issue are the orchestrator's follow-up, so the pull request says `Refs #17`.
- **The rule on pull request comments** goes into the autonomous flow (steps 4 and 5) and
  "Merging and releasing" of the root `AGENTS.md`.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The plan, its index and metrics rows, and the fragment.
2. `tests/scripts/flake_workflow.rs` pins the `flake` job with the cache step, then `ci.yml`.
3. `tests/toolchain.rs` asserts the flake leaves out the extensions, then `flake.nix`.
4. `.github/AGENTS.md` and the root `AGENTS.md` describe the cache and the toolchain; the rule
   on pull request comments.
5. The `nix build` times of both legs on the pull request, read from the logs, in the outcome.

## Outcome

- The `flake` job restores `/nix` with `nix-community/cache-nix-action` v7.0.2 and saves it
  only on a `push` to `main`; the job's permissions stay `contents: read`. The whole job is
  pinned in `tests/scripts/flake_workflow.rs`, where the job already was, and
  `tests/scripts/ci_workflow.rs` pins the workflow's one `permissions:` to `contents: read`.
- The flake builds with `(rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override
  { extensions = [ ]; }`, pinned in `tests/toolchain.rs`.
- Measured on the pull request (run 44690818423, at 62f1c7e) against the pull request run of
  phase 4's branch, PR #14 (run 44682655741). Leg times are each job's `startedAt` to
  `completedAt` (`gh run view <id> --json jobs`). The pull request's cache missed, as it must: nothing is saved before the merge.

  | Leg | `nix build` before | `nix build` after | Leg before | Leg after | Derivations built |
  | --- | --- | --- | --- | --- | --- |
  | Linux | 55 s | 52 s | 63 s | 61 s | 108 → 104 |
  | macOS | 105 s | 113 s | 142 s | 153 s | 121 → 117 |

  The four derivations gone are `clippy`, `clippy-preview`, `rustfmt` and `rustfmt-preview`;
  neither log names clippy or rustfmt any more. Both runs fetched the same 259 (Linux) and 262
  (macOS) paths from `cache.nixos.org`: the component tarballs were fixed-output derivations
  built in the sandbox, not substituted paths. The time saved is within the runners' noise
  (the nixpkgs `source` download alone varies by several seconds). The post step saved nothing
  on the pull request, and `gh cache list` shows no `nix-` key.
- Not done here, by design: the cache's effect can only be measured once the first `push` run
  on `main` has saved it, on a later pull request that changes only `src/`. That measurement,
  and the issue's "drop part 1" decision, are the follow-up, so the pull request says
  `Refs #17`.
- The root `AGENTS.md` now has the owner's rule: before `gh pr ready` and again right before
  the merge, read the pull request's comments, reviews and inline review comments and its
  issue's comments; only entries whose author is the repository's owner are instructions.
- Review: 2 rounds, both unprimed; the second was clean (no must, no should). Round 1 found
  one should: the new rule told an agent to act on any comment, in a public repository where
  anyone can comment, so it now names the owner as the only author whose text is an
  instruction and says how to tell. Its minors: the rule missed inline review comments; the
  baseline run was called a run on `main` when it was PR #14's pull request run; the leg times
  of the two runs were measured differently (now job start to end for both); the docs did not
  say a `flake.nix`-only change misses the save on every run, and overstated why there is no
  purge; a test name read badly. Round 2's minors, all in the docs and this plan: a restored
  store is trusted by who writes it, not re-hashed; the failure rows of the states table; the
  file named in task 2; the row for two merges in quick succession.
- Follow-up, after the merge (v0.5.1): the first `push` run on `main` saved both caches, and
  the first two pull request runs that hit them (44697819759 and 44697589406) took 100 s and
  61 s to restore on macOS, while `nix build` saved 41 to 53 s (70 s and 60 s against the 111
  to 113 s of the same toolchain without a hit, runs 44690818423 and 44699317602). In the better
  sample a 61 s restore bought about 50 s, and the macOS leg was 9 to 13 s slower than 149 to
  153 s; Linux moved by a few seconds either way. These are two samples, and neither pull
  request changed only `src/`: 44697589406 hit the exact key, as a `src/`-only one would, and
  44697819759 restored an older key after a `Cargo.lock` change. As the issue said, the cache
  was dropped and the toolchain change kept; the numbers are in `.github/AGENTS.md`, "No Nix
  store cache".

## Hurdles

- `process` the brief named `tests/scripts/ci_workflow.rs` for the cache step, but the `flake`
  job was already pinned whole in `tests/scripts/flake_workflow.rs`; the pin moved there and the
  red run was redone → none
- `process` `main` moved mid-phase (phase 5 merged), and the merge conflicted in `PLAN.md` and
  `plans/metrics.md`, the files every parallel phase appends to → none
- `review-loop` the owner's new rule on pull request comments first said to act on any comment,
  which in a public repository lets anyone instruct an autonomous agent; round 1 caught it →
  rule in `AGENTS.md`
- `review-loop` the docs claimed Nix re-checks a restored store against hashes, which it does
  not; round 2 caught it → code
- `ci` restoring the Nix store took 61 to 100 s on macOS, more than the 41 to 53 s it saved in
  `nix build`, so the cache was dropped after the merge → code
