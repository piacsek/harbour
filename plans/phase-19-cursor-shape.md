# Phase 19: the pane's cursor takes the shape its program asks for

## Context

Issue #34. The cursor in a pane was always a block: the pane widget (`PseudoTerminal` of
tui-term) painted it as a highlighted cell, and the `ESC [ <n> SP q` a program sends to set the
cursor's shape was dropped by `Replies::unhandled_csi` in `src/session.rs`. Neovim's insert
mode should show a thin bar and its replace mode an underline. Decided in the issue: harbour
places the terminal's real cursor at the pane's cursor cell and sends it the program's shape,
and the widget stops painting a cursor.
Issue #35 (no pane cursor while the sidebar has focus) is built right after this one,
on top of it, so the one place the real cursor is placed stays easy to make conditional.

## Verified facts (2026-11-21, `main` at fb3127a)

- Neovim sends the sequence under the `TERM=xterm-256color` and `COLORTERM=truecolor` harbour
  gives it. The clean fixture (`tests/fixtures/nvim/nvim`, NVIM v0.12.6) in a PTY with a scratch
  `HOME` and nothing else in its environment sent, captured byte for byte: `ESC[1 q`, `ESC[2 q`
  at start, `ESC[6 q` on `i`, `ESC[2 q` on Escape, `ESC[4 q` on `R`, and `ESC[0 q` as it quit.
  The same with `TERM_PROGRAM=tmux` (what a pane inherits under tmux). No TERM or terminfo
  change is needed.
- vt100 0.16.2 hands every CSI with an intermediate it does not know to `unhandled_csi`, with
  `intermediates.first()` as `first`: `ESC [ 6 SP q` arrives as `first = Some(b' ')`,
  `last = 'q'`, parameters `[[6]]`; `ESC [ SP q` arrives with no parameter, which DECSCUSR reads
  as 0.
- tui-term 0.3.4 paints the cursor in `state::handle` only while `screen.hide_cursor()` is false
  and the widget's `Cursor` is shown; `Cursor::default().visibility(false)` turns it off. Its
  `cursor_position` adds `scrollback()` to the row. Harbour's parsers have no scrollback
  (`Parser::new_with_callbacks(rows, cols, 0, ..)`), so a pane never scrolls away from its
  cursor; the cursor can still fall outside the drawn area when the pane is shorter than the
  screen's 2 rows minimum, and then no cursor is placed.
- ratatui 0.30.2 shows the terminal's cursor at the position `Frame::set_cursor_position` names
  after a draw, and hides it when no frame names one. It never sends a shape. `TestBackend`
  reports both (`cursor_visible`, `cursor_position`), so in-process tests can read them.
- tmux 3.7c keeps a pane's cursor shape: `#{cursor_shape}` (`default`, `block`, `underline`,
  `bar`) and `#{cursor_blinking}`, so an end-to-end test reads what harbour sent to its pane.
- `contained` in `src/session.rs` replaces a panicked parser with a fresh one and new
  `Replies`; the shape lives in `Replies`, so it is carried over there like the panic count.

## What the user sees

| State | Cursor |
| --- | --- |
| Program sends `ESC[0 q` or `ESC[ q` | the terminal's default cursor, at the pane's cursor cell, once harbour sent another shape during the run; until then harbour sends nothing, so the pane keeps whatever shape the shell left before harbour started |
| A line that exactly fills the pane (the cursor waits after the last column) | the cursor on the last column, as a terminal shows it |
| `ESC[1 q` / `ESC[2 q` | a blinking / steady block |
| `ESC[3 q` / `ESC[4 q` | a blinking / steady underline |
| `ESC[5 q` / `ESC[6 q` | a blinking / steady bar, between characters |
| `ESC[7 q` or more | the shape it had before (the value is ignored) |
| Program hid its cursor (`ESC[?25l`) | no cursor anywhere; the shape stays the program's, so a Neovim hiding it around each redraw sends nothing |
| Pane shorter than the screen, cursor below the drawn area | no cursor |
| Any frame, the shape changed or not | the cursor hidden before the frame's first cell and shown at its cell after the last, so it never jumps across the screen while the frame draws |
| A panic on another thread (the hook resets the shape and harbour goes on) | the next frame sends the program's shape again |
| Sidebar has focus | the pane's cursor, as before (#35 changes this) |
| Help view shown | no cursor |
| A question at the bottom of the sidebar | the pane's cursor stays in the pane; the question rows never get it |
| Picker popup shown | the picker's own cursor and shape (fzf's prompt), never the pane session's |
| Stopped card or no session selected | no cursor, default shape |
| Another session shown (`j`/`k`, the editor with `e`, C-h/C-l) | that session's cursor and shape; default when it never set one; the cursor is hidden before the new shape is sent, and shown at its new cell only at the end of that frame |
| A failure message drawn over the bottom rows of the narrow pane-only layout | no cursor while it sits on a row the message covers; above it, the cursor as before |
| After a contained vt100 panic | the shape the program set before, on the fresh screen |
| Exited session, dimmed | its last cursor and shape, as the painted one showed |
| Under tmux | harbour's pane takes the shape (`#{cursor_shape}`), and tmux passes it to the outer terminal when that terminal's terminfo has `Ss` |
| A terminal that ignores DECSCUSR | its own cursor, unchanged: harbour sends only a well-formed `CSI Ps SP q`, and only when the shape changes, so nothing at all while every program keeps the default |
| Quit | `ESC[0 q`, the terminal's default shape, only when the last shape harbour sent was another; nothing when harbour never sent one, so a shell's own shape survives; then ratatui shows the cursor |
| Panic (not contained) | the same, from the panic hook through `leave_terminal`, which shares the last shape sent |
| Program sends a full reset (`ESC c`) | left out: harbour keeps the shape it had, since vt100 handles `ESC c` itself without a callback, so harbour never sees it; working around vt100 is not worth it for a sequence programs rarely send |
| Narrowest sidebar (the pane at its widest) | the pane's cursor at its cell; the sidebar never shows it |
| A CJK line under the cursor (a wide cell) | the real cursor sits on the wide character's first column; the character is drawn whole, no longer restyled by a painted cursor |

Quit resets to the default shape rather than to what the terminal had before harbour: no
terminal answers which shape it shows, and the default is what Neovim itself leaves on exit.

## Tasks

1. This plan and its rows; a session remembers the last shape its program asked for, through a
   contained panic too, and says where its cursor is (`src/session.rs`); the widget stops
   painting one; the app places the terminal's real cursor at the shown pane's cursor cell and
   says which shape it wants (`src/app/draw.rs`): none for the help view, a stopped card or a
   hidden cursor, the picker's own over the popup. One ship, since the session's new functions
   would be dead code until the app calls them.
2. The terminal is sent the shape when it changes and the default when harbour leaves it, on a
   panic too (`src/cursor.rs`, `src/tui.rs`); an end-to-end test under tmux reads
   `#{cursor_shape}`.
3. Screenshots, fragment, outcome.

## Outcome

- The pane's cursor is the terminal's real cursor: the app places it at the shown pane's cursor
  cell (the picker's own over the popup, none for the help view, a stopped card, a hidden cursor
  or one outside the drawn pane) and the widget paints none. `docs/details.png` and the other
  shots of a pane lost their painted block; a screenshot cannot show the real cursor or its shape.
- A session keeps the last shape `ESC [ 0..6 SP q` asked for (7 and up ignored), through a
  contained vt100 panic too. Harbour writes the shape only when it changes, so a terminal that
  ignores DECSCUSR sees nothing while every program keeps the default. The shape is the shown
  program's whether or not its cursor is hidden, so a Neovim hiding it around each redraw sends
  nothing, and it is written inside the frame, after `ESC[?25l`, so the cursor shows at its new
  cell with the new shape only at the end of that frame. Harbour writes `ESC[0 q` when it leaves
  the terminal, at quit and from the panic hook, only when the last shape it sent was another,
  so a shell that sets its own shape keeps it when harbour never sent one (after harbour sent
  the default back, the terminal already has it).
- Every frame starts with `ESC[?25l` and ends, through ratatui, with the cursor shown and moved
  to its cell, so it never jumps while the cells are written. Hiding was chosen over
  synchronized output (`ESC[?2026h`): it is one write every terminal understands, and a terminal
  without synchronized output would still show the cursor jumping.
- A cursor waiting after the last column (a line that exactly fills the pane) shows on that
  column. After the panic hook resets the shape, the next frame sends the program's shape again.
- The pane keeps whatever shape the shell left until a program in harbour asks for another,
  since harbour sends nothing while every program keeps the default.
- A failure message over the bottom rows of the narrow pane-only layout hides the cursor while
  it sits on those rows. A full reset (`ESC c`) inside a session keeps the remembered shape:
  vt100 handles it without a callback (the states table).
- Neovim sends the shapes under the `TERM` harbour gives it, verified byte for byte with the
  clean fixture; no terminfo change was needed. Under tmux, `#{cursor_shape}` of harbour's pane
  follows the shown session (`tests/e2e/cursor.rs`).
- Quit resets to the default shape, not to what the terminal showed before harbour: no terminal
  reports its shape, and Neovim does the same on exit.
- Issue #35, built next on top of this, makes the one place the cursor is set (`placed` for the pane in
  `src/app/draw.rs`) conditional on the pane's focus.

## Hurdles

- `test-seam` The in-process tests of the new session functions and the app's cursor could not be shipped apart: the functions were dead code until the app called them, so plan tasks 1 and 2 became one ship → none
- `test-seam` An end-to-end test sent keys right after `n` while the sidebar still had focus, so the session never got them; it presses Enter first, as the focus test does → none
- `test-seam` A failing test in `tests/tui.rs` trips the counting panic hook of the test beside it and hides its own message, so the red of the cursor reset showed as the hook test failing → none
- `review-loop` Round 1 found the shape flickering under a Neovim that hides its cursor around each redraw, the shape written after the frame showed the cursor, and the reset at quit sent even when harbour never sent a shape; a states table row per frame step (what is written, in which order) would have listed them → none
- `review-loop` Round 2 found the cursor visible while each frame's cells were written, a cursor waiting after the last column hidden, and the panic hook's reset not recorded → none
