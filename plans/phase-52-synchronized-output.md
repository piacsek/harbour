# Phase 52: Claude Code's repaints show as whole frames

A Claude Code session flickers on every repaint: a half-drawn screen shows before the whole one
(#238). Claude Code wraps each repaint in synchronized output (`ESC [ ? 2026 h` to
`ESC [ ? 2026 l`); the PTY reader of `src/session.rs` feeds each read into `vt100`, which
ignores mode 2026, and the draw thread draws whatever the screen holds between two reads.

## Verified facts

- **vt100 hands mode 2026 to the callbacks.** `Screen::decset` and `decrst` (vt100 0.16.2) call
  `unhandled_csi` with `Some(b'?')`, the whole parameter list and `h` or `l` for a mode they do
  not know, so `Replies` sees the markers. The parser keeps its state between two `process`
  calls, so a marker split across two reads still arrives whole, with no scan of our own.
- **`vt100::Screen` is `Clone`.** A copy holds both grids, the cursor and the modes.
- **Neovim asks before it uses the mode.** `nvim --clean` (v0.12.6) under `script` writes
  `ESC [ ? 2026 $ p` (DECRQM) once and, with no answer, never wraps its output. vt100 hands the
  query to `unhandled_csi` with `Some(b'?')`, `Some(b'$')` and `p`.
- **Every pane goes through one reader.** Claude Code sessions, the editor, the picker and the
  usage popup are all `Session`s, drawn by `Session::draw` with the frame's `now`.
- **The interface draws a frame at least every 30 ms** (`src/tui.rs`), so a timeout shows on
  the next frame without any output from the program.

## Decided

- The owner's: keep showing the last whole frame while an update is open, never a blank; give
  up after 100 ms, as Ghostty and kitty do; no setting to turn it off.
- The builder's: **a copy of the screen at the begin marker.** `Replies` keeps the copy and the
  time it was taken; `Session::draw` draws the copy, and takes the cursor from it, while it is
  younger than 100 ms, else the live screen. The end marker drops it. Withholding the draw
  instead would need the draw thread to keep each pane's last cells per area, which a resize or
  a layout change invalidates; a copy at the begin needs no area. A copy costs one screen clone
  per repaint, taken under the parser lock the reader already holds.
- The builder's: **the editor gets the same treatment by answering Neovim's query.** A DECRQM
  for mode 2026 is answered `ESC [ ? 2026 ; 2 $ y` (known, reset), `; 1` while an update is
  open, for every role, so Neovim starts wrapping its repaints and the same copy covers it.

## States

What harbour reads is the program's output, through its PTY.

| State | What the user sees |
| --- | --- |
| Begin and end in one read | The new frame, whole; the copy is taken and dropped under one lock, never drawn. |
| Begin and end split across reads | The frame before the repaint until the end arrives, then the new one whole. |
| A marker itself split across two reads | The same: vt100 keeps the half sequence and reports it whole on the next read. |
| A begin while an update is open (nested or repeated) | The first copy stays, and so does its 100 ms; the second begin is not a new frame. |
| A begin after an open update ran out | A fresh copy: the screen as it is now, the last thing the user saw. |
| An end with no begin | Nothing changes; the live screen is drawn. |
| A begin with no end | The frame before it for 100 ms, then the live screen, redrawn on the next frame with no output needed. |
| The program exits mid-update | The frame before the repaint for at most 100 ms, then the screen it left, dimmed as an exited session is. |
| A resize mid-update | The copy keeps its old size, clipped or padded in the new pane, for at most 100 ms, until the end or the timeout; the program's repaint after the resize then shows whole. |
| `ESC [ ? 1049 ; 2026 h` in one DECSET | vt100 applies 1049 first, so the copy is the cleared alternate screen and a blank can show for up to 100 ms; accepted as rare. |
| The alternate screen switched mid-update | The copy holds the grid that was shown at the begin; the new grid shows at the end. |
| A terminal reset (`ESC c`) mid-update | Still the copy until the end or the timeout; the reset changes the live screen only. |
| vt100 panics mid-update | The session's screen is rebuilt with no copy, as before: the blank screen until the program repaints. |
| Neovim asks for mode 2026 | It gets the answer and wraps its repaints; the editor pane shows whole frames too. |
| A query while an update is open | Set while the copy is younger than 100 ms, reset after, as the pane is drawn. |
| The frame before hid its cursor | No cursor while the copy shows, never one at a spot of the half-drawn screen. |
| A program that never asks and never sends the markers | Nothing changes. |

## Tasks

1. A session whose program opened an update draws the frame before it, and its cursor, until
   the update ends or 100 ms pass (`tests/session/synchronized.rs`).
2. A begin after an open update ran out takes a fresh copy.
3. A DECRQM for mode 2026 is answered, set while an update is open.
4. In the app, a repaint that never ends still shows once the wait is over
   (`tests/app/frames.rs`): the wiring through `App::draw`, and the failure that would hurt
   most, a frozen pane. Whether the frame before shows can be checked only against the
   100 ms, which no in-process or tmux test can hold without a seam, so the session tests,
   which hand `draw` the instant, own it.

## Outcome

- Shipped in #244 as a `fix`: a session draws a copy of its screen taken at `ESC [ ? 2026 h`,
  and its cursor (none when the copy's is hidden), until `ESC [ ? 2026 l` or 100 ms; a
  DECRQM for mode 2026 is answered set while that copy is fresh, else reset.
- vt100 already reports the markers to `unhandled_csi` and holds a sequence split across reads,
  so no scan of the reader's chunks was needed.
- Neovim 0.12.6 wraps its repaints only once its DECRQM is answered (0 begins unanswered, 5
  answered, by hand in a PTY), so answering it gives the editor the same treatment.
- No tmux end-to-end test: it cannot hold the 100 ms window. The session tests hand `draw` the
  instant; `tests/app/frames.rs` keeps the guard that an update never ended still shows.
- Review: one round of `/code-review high`, no must or should; its nits (the query's state after
  the timeout, no lingering cursor over the copy, the 1049 row above, a fixture's text, a guide
  line) were fixed in the closing ship.

## Hurdles

- `test-seam` Two tests were green on their first run: the timeout ones, which pass without the
  feature, and the fresh-copy one, whose expired copy fell through to a live screen holding the
  same text; the first were broken and restored, the second got a negative assertion → code
- `test-seam` The previous frame showing can only be checked against the 100 ms window, which no
  tmux or in-process app test can hold without a seam, so the session tests own it → none
