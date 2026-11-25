# Phase 47: choose where the editor opens

Issue #228. C-l opens the editor at the repository root of the directory Claude Code was last
seen in. For a repository of separate projects that is the wrong folder: a session working in
`~/acme/notes/rust` gets Neovim at `~/acme/notes`. A setting from the file picks either the
root, as today, or that exact directory.

## Verified facts

- **One function decides the editor's directory today**: `editor::root` (`src/editor.rs`), the
  repository root `repo::root` finds above the directory Claude Code was last seen in, or that
  directory when there is none or it is gone. Its callers are `fresh_root` and `holder` in
  `src/app/handoff.rs` and `Follow::to` in `src/follow.rs`; nothing else calls `repo::root`.
- **Every way an editor opens goes through `fresh_root`**: C-l and swapped C-h (`open_editor`
  from `src/app/key.rs`), side by side and a jump while side by side (`src/app/side.rs`), and
  `harbour edit` (`open_in` with the directory `holder` found).
- **The pane border's editor label is drawn from the editor's directory** (`label` in
  `src/editor.rs`), so it follows whatever directory the editor was started in.
- **`claude_icon` is the model of a file-only enum setting**: `Written`, a `Reader` method that
  refuses any other value naming the line and the key, a `Setting` with its source, a line in
  `harbour config`, a builder on `App` set by `src/tui.rs`, which `--demo` never reaches.

## Decided

- The orchestrator's: `editor_dir`, from the file only, `"root"` (the default) or `"exact"`;
  one place decides for C-l, side by side, the jump that opens an editor, `harbour edit` routing
  and the folder tmux follows; with `"exact"`, `harbour edit <file>` routes a file to a session
  only when it lies inside that exact directory; the pane border's label shows it; `harbour
  config` lists it with its source; any other value is a bad file naming the line and the key;
  `harbour --help` names it on its config lines; `--demo` unchanged; no screenshot change.
- The builder's: `editor::root` becomes `editor::directory`, taking an `EditorDir`, since under
  `"exact"` it is no root; the refusal reads `editor_dir: expected "root" or "exact"`;
  `harbour config` prints `editor_dir` right after `editor`.

## States

| `editor_dir` | Claude Code last seen in | The editor opens in, tmux follows, `harbour edit` takes files under |
| --- | --- | --- |
| absent or `"root"` | a subfolder of a repository | the repository root, as today |
| `"exact"` | a subfolder of a repository | that subfolder |
| `"exact"` | a repository root | that root |
| `"exact"` | a folder outside any repository | that folder, as today |
| `"exact"` | the home directory holding a dotfiles `.git` | the home directory |
| `"exact"` | a directory that is gone | named as today, nothing walks up; tmux goes back where harbour was opened |
| `"exact"` | a symlinked folder | that folder; `harbour edit` compares through symlinks, as today |
| `"exact"`, then Claude Code moves to another subfolder | a second subfolder | a second editor there, as two repositories get today |
| `"Root"`, `"x"`, `1`, `true` | any | harbour refuses to open, in one line naming the file, the line and `editor_dir` |
| `"exact"` | any, under `--demo` | the demo never reads the file: the root, as today |

Text drawn changes only through the directory the label names, which `tabs::strip` fits as
today, so the narrowest sidebar and a CJK directory behave as for any other directory.

## Tasks

1. `editor_dir` read, refused and reported by `src/config.rs` (`tests/config/`).
2. `editor::directory` takes the setting (`tests/editor.rs`), and the app passes it to every
   caller: C-l in a subfolder (`tests/app/directory.rs`), the folder tmux follows
   (`tests/app/moving.rs`), `harbour edit` routing (`tests/app/nearest.rs`), end to end through
   the file (`tests/e2e/config.rs`); `harbour --help`, the `AGENTS.md` lines, the fragment.

## Outcome

`editor_dir` (`"root"` by default, or `"exact"`) is read from the file only, refused naming the
line and the key for any other value, and listed by `harbour config` after `editor`.
`editor::root` became `editor::directory`, which takes the setting and returns the directory
itself under `"exact"`; `fresh_root` and `holder` in `src/app/handoff.rs` and `Follow::to` pass
the app's `editor_dir` (`App::with_editor_dir`, set by `src/tui.rs`), so C-l, side by side, the
jumps, `harbour edit` routing and the folder tmux follows all use the same directory, and the
pane border's label names it. `harbour --help` names the setting on its settings lines, the C-l
line and the `edit <file>` line, and `docs/help.png` was rerendered; `docs/details.png` did not
change. Tests, each red first: the refusals and both `harbour config` listings
(`tests/config/`), three rows of `editor::directory` (`tests/editor.rs`, two of them guards shown
red by breaking the code), C-l in a subfolder (`tests/app/directory.rs`), the folder tmux follows
(`tests/app/moving.rs`), `harbour edit` routing (`tests/app/nearest.rs`), the setting through the
real binary with the border's label (`tests/e2e/config.rs`) and the help text (`tests/cli.rs`).
Shipped in two tasks on draft PR #229. Phase 46's ready, merged and released were read from
GitHub into `plans/metrics.md` (v0.41.0).

Review round 1 found one should and two nits, fixed in one ship. Under `"exact"`, `harbour edit`
could be refused: the ask judged each session by the folder last read, the open read it again
first, so a `cd` of Claude Code inside the repository since the last read made the claim and the
open disagree. The ask now reads each running session's folder again as the open does
(`refresh_live`, red in `tests/app/nearest.rs`), and `docs/known-limits.md` says what that costs
per ask. Two new tests now wait for exactly what they assert, and the `src/AGENTS.md` entry of
`src/app/handoff.rs` says directory where root was no longer right.

## Hurdles

- `tooling` `scripts/screenshots.sh` rewrote five images besides `docs/help.png` whose content did not change (a run renders them differently each time); only the help image was kept → none
- `review-loop` round 1 found the ask and the open of `harbour edit` judging by different reads of where a session works, harmless while both walked up to the same repository root and wrong under `"exact"`; the plan's States table listed what each directory opens, not when it is read → code
