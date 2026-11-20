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

## Hurdles
