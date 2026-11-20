# Phase 2: the selected session and the focused side in cyan

## Context

At a glance it is hard to tell which session is selected and whether the sidebar or the pane
has the keys: the selected row is marked by a small `›` and bold text, the focused side by a
thicker border. Issue #3 asks for conveyor's cyan highlight: a cyan `▌` bar on the selected row
in place of `›`, and a cyan border on the focused side. The bar says which session is shown, the
border says where the keys go.

## Verified facts (2026-11-20, `main` at 036e8a5)

- `src/app.rs` draws the sidebar as a `List` with `highlight_symbol("› ")`,
  `HighlightSpacing::Always` and `highlight_style(Modifier::BOLD)`. `border(focused)` returns
  `BorderType::Thick` or `Plain`, and both blocks call it; no colour is set anywhere in `src/`.
- `List::highlight_symbol` in `ratatui-widgets` 0.3.2 takes `Into<Line>`, so the symbol can be a
  styled span; `HighlightSpacing::Always` pads unselected rows by the symbol's width.
- conveyor draws the bar as `Span::styled("▌", fg Cyan)` followed by a plain space, with the
  title bold (`~/projects/conveyor/src/ui/card.rs`). harbour's rows are one line, so one bar cell
  per row.
- An exited selected session dims only the pane's inner area (`set_style(self.pane, DIM)`), not
  its border.
- Below 60 columns only one side is drawn: the sidebar, or the pane while it has focus.
- `tests/app/` reads the `TestBackend` buffer as text only; tests of `ending.rs` and
  `selection.rs` assert the `› ` marker.

## Decisions

- **The bar is `▌` and a space, in place of `› `**, so rows keep their width. Unselected rows
  start with two spaces. The name stays bold.
- **Only the foreground is cyan.** No background, no reverse video, as in conveyor.
- **An exited session keeps the cyan bar, and its dimmed pane keeps the cyan border when it has
  focus.** Selection and focus are facts about harbour, not about the session: the exited row
  is still the one shown, and the keys still go to its pane. "exited" in the row and the dim
  carry the exit; dimming the highlight too would hide where the user is.
- **The narrow layout shows its one side with a cyan thick border**: whichever side is drawn is
  the side with focus. The bar shows in the narrow sidebar as in the wide one.
- **Colour is never the only signal**: the thick border and the bold name stay.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. The selected row starts with a cyan `▌`; unselected rows do not; existing tests move off `›`.
2. The focused side's border is cyan, the other side's is not, for both sides.
3. The fragment, the screenshots rerun, and the docs that mention the marker.

## Outcome

- All 3 tasks shipped: the bar (b1a04fb), the border with the fragment (cedc23a), and the
  screenshots rerun (ad573cb). The selected row starts with a cyan `▌` in place of `›`, and the
  focused side's border is thick and cyan while the other side's is plain and uncoloured.
- The decisions held as planned: an exited session keeps the cyan bar, and its dimmed pane keeps
  the cyan border while it has focus, since the dim covers only the pane's inside; the narrow
  layout draws only the focused side, with the cyan border; colour is never the only signal,
  since the thick border and the bold name stay.
- Not in the plan, added on the way or by review:
  - `scripts/truecolor.py` carries the current colour to the start of each line, since
    `tmux capture-pane -e` does not repeat it, and leaves an empty line bare so freeze does not
    panic on the capture's end. Both have tests in `tests/scripts/truecolor.rs`.
  - `docs/pane.png`, the demo with the pane focused, so the cyan pane border shows in a shot.
  - Two cell-colour tests: an exited session's pane keeps the undimmed cyan border, and the
    narrow layout's one side has it. Both passed at once and were proved by breaking the code.
  - 17 tests no longer depend on the length of the checkout's folder name, which every sidebar
    row carries: 15 `tests/app` tests build a screen sized to it with `Screen::wide`, the narrow
    sidebar test asserts on the row's start, and the session working-directory test sizes the
    session to the path. A run from a 49-character folder name passed the app, session and e2e
    targets.
- Review: 3 rounds, each with both reviewers (Claude and Codex), all unprimed; the last was
  clean. Round 1's only finding was the PR body, which still carried phase 1's text: the edit
  command was blocked by the text guard hook for an em dash, and the next command uploaded a
  stale `body.md` left in the shared scratchpad by phase 1, which nobody fetched back.
- Left as it is: the " sessions " and " End? y/n " titles turn cyan with the focused border,
  since ratatui lays a title over its border's style. Both reviewers saw it and neither called
  it a fault.

## Hurdles

- `test-seam` Ten `tests/app` tests failed in a checkout whose folder name is longer than about 22 characters, since every sidebar row carries that name and the sidebar is a third of the screen → rule in `tests/AGENTS.md`
- `test-seam` The two cell-colour tests passed on their first run because the code already did both; each was proved by breaking the code → none
- `tooling` `tmux capture-pane -e` does not repeat a colour that runs across a line break, so the narrow shot lost the cyan on its left and bottom edges → code
- `tooling` Once the converter carried colour to every line, freeze panicked on colour codes after the capture's last newline → code
- `ci` The `version` check failed on the bar commit, which changed `src/` before the fragment was written → none
- `tooling` A shell command holding an em dash was blocked whole by the text guard hook, so the new PR body file was never written → none
- `process` The builder reported a PR body edit as done without fetching it back, so round 1 was spent on a description that still carried phase 1's text → rule in `AGENTS.md`
- `tooling` The scratchpad is shared between phases of one orchestrator session, so a stale `body.md` from phase 1 was uploaded by mistake → none
- `tooling` The long-folder worktree reused this worktree's target dir, and cargo then kept an e2e binary pointing at the removed worktree's fixtures, so the first ship failed 7 e2e tests until a rebuild was forced → none
