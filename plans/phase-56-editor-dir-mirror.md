# Phase 56: the editor follows Claude Code into another checkout

`editor_dir` takes `"root"` (the default: the repository root of the directory Claude Code is
seen in now) or `"exact"` (always the directory the session started in). With `exact`, once
Claude Code moves into another checkout of the same project, a linked git worktree say (it
started in `~/projects/acme/app/rust` and its transcript's `cwd` is now
`~/projects/acme/app-feature`), C-l still opens the old checkout.

A third value, `editor_dir = "mirror"`, opens the same subfolder of the start, in the checkout
Claude Code is in now. `root` and `exact` keep their behaviour; the default stays `root`.

## Verified facts

- **One function decides where the editor opens.** `editor::directory` (`src/editor.rs`) is
  called by C-l, side by side and `e` (`src/app/mod.rs`), `harbour edit`
  (`src/app/handoff.rs`) and the directory harbour moves to (`src/follow.rs`), each with the
  directory Claude Code was last seen in and the directory the session started in.
- **`repo::root` walks up from a directory** to the nearest folder holding a `.git` directory,
  a `.git` file whose `gitdir:` exists (a linked worktree, a submodule) or a bare repository,
  skipping the home directory itself; for a folder that is gone it walks up from the nearest
  folder that is still there.
- **A linked worktree has a root of its own:** its `.git` file makes its folder the root, never
  the main checkout's, so the main checkout and a worktree are two roots.

## Decided

- The owner's: the name `mirror`.
- The owner's: `mirror` resolves as below, in `editor::directory`.

| Seen directory now | Start's checkout root | Seen checkout root | The editor opens in |
| --- | --- | --- | --- |
| gone (not a directory) | any | any | the start directory, as `exact` |
| a directory | none (outside a repository) | any | the start directory |
| a directory | any | none (outside a repository) | the start directory |
| a directory | a root | the same root, by device and inode (a symlinked path too) | the start directory |
| a directory | a root | another root, which holds the start's subfolder | `<seen root>/<subfolder>` |
| a directory | a root | another root, without that subfolder | the seen root |
| a directory | a root, the start itself (no subfolder) | another root | the seen root |

What reads from outside harbour, and what the user sees when it changes:

| Read | Missing | Changed in between | Shared |
| --- | --- | --- | --- |
| the start directory | the start directory is returned, as `exact` does | read again at every call | n/a |
| the seen directory (transcript `cwd`) | the start directory | read again at every call, so C-l and `#{pane_current_path}` follow | n/a |
| the checkouts' git files | no root: the start directory | read again at every call | two harbours read, never write |

The plan draws no text, so no width table is needed.

## Tasks

1. `editor::directory` with `EditorDir::Mirror` (`tests/editor.rs`, every row above), and the
   app opening the editor in the worktree Claude Code moved to (`tests/app/directory.rs`).
2. `editor_dir = "mirror"` read from the file, the refusal naming three values, `harbour config`
   printing it (`tests/config/`); the interface through a scratch config
   (`tests/e2e/config.rs`).
3. `harbour --help` explains `mirror` (`tests/cli.rs`), `src/AGENTS.md` describes it, the
   fragment.

## Outcome

- **`editor_dir = "mirror"` opens the start's subfolder in the checkout Claude Code is in now.**
  C-l, side by side, `e`, `harbour edit` and the directory harbour moves to for tmux all follow
  it, since they share `editor::directory`. `root` and `exact` are unchanged; the default stays
  `root`.
- The name `mirror` (the owner's): the editor mirrors the start's place into the other checkout.
- Claude Code's directory gone gives the start directory: nothing is left to mirror into, and
  `exact`'s answer is the one the user picked.
- Either side outside a repository gives the start directory: without two checkout roots there
  is no subfolder to carry over.
- The same checkout gives the start directory, roots compared by device and inode: inside the
  start's own checkout `mirror` behaves as `exact`, a symlinked path included.
- Another checkout without the subfolder gives its root: the nearest folder of that checkout
  that surely exists.
- `repo::identity` became public so `editor.rs` compares roots the way `repo.rs` compares the
  home directory, rather than holding a second copy.
- After task 1 the orchestrator, at the owner's word, switched this run from `scripts/ship.sh` to plain git: tasks 2 and 3
  ran their related tests, `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and
  `scripts/scrub-check.sh` (both modes) before each commit, without the full gates or the dev
  install.
- `docs/help.png`, the shot of `harbour --help`, was not rerendered on the branch: the help
  text changed, so `scripts/screenshots.sh` is due before `gh pr ready`.

## Hurdles

- `process` The orchestrator, at the owner's word, switched from `scripts/ship.sh` to plain git after the first ship, so the
  second ship's gates were stopped midway and tasks 2 and 3 ran only their related tests → none
