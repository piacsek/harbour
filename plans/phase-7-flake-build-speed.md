# Phase 7: a faster macOS flake build

## Context

Issue #17. `flake (macos-latest)` is the slowest job of `ci` and decides how long a run takes.
Most of its `nix build` step fetches and unpacks what stays the same until `flake.lock` or
`Cargo.lock` changes, and part of it builds clippy and rustfmt, which the package never runs.
This phase caches the Nix store between runs and builds the flake with a toolchain that leaves
both out. The owner also asked for one rule: read the pull request's comments before it is
marked ready and again before it merges.

## Verified facts (2026-11-20, `main` at fe8aaf5)

- Run 44682655741 (on `main`), from its job logs:

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
| The cache service down or the restore failing | the toolkit warns and the step goes on as a miss | as slow as today, never wrong |
| The action itself crashing | the step fails, so the leg and `flake-ok` fail | red, never a wrong binary; rerun the job |
| `flake.nix` changed without a lockfile change | the old key hits; Nix builds what the new expression needs | correct, slower than a hit; the key is not refreshed until a lockfile changes |

## Why a restored store cannot change the binary

- Every store path is named by a hash of its inputs: a derivation's path hashes its builder,
  its arguments and the paths of everything it depends on, the harbour source fileset included.
  A changed source, lock or toolchain gives other paths, which the restored store does not have,
  so Nix builds them.
- What Nix downloads (nixpkgs, `rust-overlay`, the toolchain and crate tarballs) is
  fixed-output: checked against the hash `flake.lock`, `Cargo.lock` or the overlay's manifest
  declares, at fetch time.
- What the action restores is `/nix` as a `main` push left it, and only `main` pushes write
  it. A pull request can read it but never write it, so no pull request can put a path into the
  store a later `main` build reads.

## Decisions

- **Save only on a `push` to `main`**: `save: ${{ github.event_name == 'push' && github.ref == 'refs/heads/main' }}`.
  Pull requests read `main`'s cache and never write one.
- **No purge.** It needs `actions: write`, and job permissions cannot be granted for one event
  only, so every same-repository pull request run would hold it too. Old keys are left to
  GitHub's eviction (7 days unused, least recently used above 10 GB). The job keeps
  `contents: read` and gains no permission.
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
2. `tests/scripts/ci_workflow.rs` pins the `flake` job with the cache step, then `ci.yml`.
3. `tests/toolchain.rs` asserts the flake leaves out the extensions, then `flake.nix`.
4. `.github/AGENTS.md` and the root `AGENTS.md` describe the cache and the toolchain; the rule
   on pull request comments.
5. The `nix build` times of both legs on the pull request, read from the logs, in the outcome.

## Outcome

## Hurdles
