# Phase 27: Alt-minus and Alt-equals change the sidebar's width

Issue #37. Alt-minus makes the sidebar 2 columns narrower and Alt-equals 2 columns wider, from
the sidebar and from the Claude Code pane, until harbour quits. A held key repeats. A session
never receives either key.

## Verified facts

- **The bytes**, captured in a scratch tmux 3.7c server (`tmux -L <scratch> -f /dev/null`,
  `TMUX_TMPDIR` a short `mktemp -d /tmp/...` folder) by a program that turns on modifyOtherKeys
  2 (`\e[>4;2m`, what harbour writes) and prints every read in hex:

  | tmux setup | `send-keys M--` | `send-keys M-=` | `send-keys M-+` |
  | --- | --- | --- | --- |
  | `extended-keys on`, `extended-keys-format xterm` | `1b 5b 32 37 3b 33 3b 34 35 7e` (`CSI 27;3;45~`) | `CSI 27;3;61~` | `CSI 27;3;43~` |
  | `extended-keys on`, `extended-keys-format csi-u` | `1b 5b 34 35 3b 33 75` (`CSI 45;3u`) | `CSI 61;3u` | `CSI 43;3u` |
  | `extended-keys always`, xterm | `CSI 27;3;45~` | `CSI 27;3;61~` | `CSI 27;3;43~` |
  | `extended-keys off` (tmux's default) | `1b 2d` | `1b 3d` | `1b 2b` |
  | `extended-keys on`, the program did not ask | `1b 2d` | `1b 3d` | `1b 2b` |

  In every setup `send-keys Escape -` and `send-keys -H 1b 2d` arrived as `1b 2d`, and the same
  for `=` (`1b 3d`).
- **Esc then `-` typed into tmux**: bytes fed to an inner tmux client by an outer tmux, with
  `extended-keys on`. Sent in one write, Esc and `-` reached the pane as the extended Alt-minus
  in both formats. With tmux 3.7c's default `escape-time` 10, Esc and `-` sent 5, 50 and 300 ms
  apart reached it as a lone `1b`, then `2d`; with `escape-time` 500 they reached it as
  `CSI 27;3;45~`. So inside tmux, Esc then `-` (or `=`) within tmux's escape-time becomes the
  extended Alt key before harbour reads a byte, and no rule over bytes can tell the two apart.
- **No Repeat event**: harbour asks for modifyOtherKeys 2 and never for kitty's event types, so
  a held key arrives as one press after another, each in the forms above.
- **Neovim 0.12.6**, clean and headless: `i<M-->` on line 3 left insert mode and went to line 2
  (Esc, then `-`); `i<M-=>k` reindented line 1 from `  a` to `a` (Esc, then `=k`). Neovim reads
  both keys as Esc then the character in insert mode, so taking either from the editor would
  steal `Esc -` and `Esc =`.
- **Claude Code** binds neither by default: its keybindings page
  (code.claude.com/docs/en/keybindings, read 2026-11-21) lists Meta+P, Meta+O, Meta+T, Meta+Up
  and Meta+Down as its Meta bindings; `ctrl+shift+-` is undo and `ctrl+x =`, `ctrl+x -`,
  `ctrl+x ctrl+=`, `ctrl+x ctrl+-` are chords of other keys. A user may still bind `meta+-` or
  `meta+=` in `keybindings.json`. Not run.
- **Harbour's reader** (`src/input.rs`) reads `CSI 27;3;45~`, `CSI 45;3u` and `ESC -` as
  `KeyEvent(Char('-'), ALT)`, the last with `input::ESCAPED` set (phase 25).
- **Every frame resizes every session to the pane** (`compose` in `src/app/draw.rs`), and
  `Session::resize` resizes the PTY only when the size changed, so a session gets one SIGWINCH
  at the first frame after the width changes.

## Decided

- **The key rule is Alt-z's** (phase 25): harbour takes Alt-minus and Alt-equals only in an
  extended form (`CSI 27;3;45~`, `CSI 45;3u`, `CSI 27;3;61~`, `CSI 61;3u`: Alt, a press, not
  marked `ESCAPED`), and never while the editor has focus; the legacy `ESC -` and `ESC =`, and
  either key while the editor has focus, reach the session or the editor as today. My rule does
  not differ from Alt-z's: Neovim reads both keys as Esc then the character (above) exactly as
  it reads Alt-z, so the smallest rule that never steals a Neovim key is the same one.
  - The issue asked for the keys "from inside a pane (Claude Code or the editor)"; from the
    editor they are not taken, for the reason above. C-h first, as for Alt-z.
  - Cost: without extended keys (tmux's default `extended-keys off`, a terminal that ignores
    modifyOtherKeys) nothing happens and the session gets `ESC -` or `ESC =`; from the editor
    nothing happens; in a Claude Code pane inside tmux with extended keys, Esc then `-` or `=`
    typed within tmux's escape-time changes the width, and Claude Code gets neither key.
    `harbour --help`, the PR body and the fragment say so.
- **Only `=`, not `+`**: Alt-equals is the key the issue names; `+` needs Shift on most layouts
  and arrives as its own code, so it stays the session's.
- **A taken key is consumed even when nothing happens** (at a limit, under 60 columns, zoomed),
  so a session never receives it. A key held at a limit does nothing, press after press.
- **The narrowest sidebar is 15 columns.** The sidebar's title inside its border must hold the
  short counts (`state::title` falls back to ` ●1 ◉1 ○1 ·1 `), 13 columns for all four kinds
  with one-digit counts, so 15 with the border. A card is the sidebar's inner width less 2
  (the bar and its gap), and its first line keeps 2 more for the dot: 15 leaves 9 columns, so a
  3-column age (`<1m`), its gap and 5 columns of title (2 CJK characters and `…`). The card
  alone would need 12. Two-digit counts in all four kinds overflow 15 as they overflow any
  narrow `sidebar_width` today; not acted on.
- **The widest sidebar is half the terminal** (`area.width / 2`, rounded down, the limit
  `sidebar_width` already has), so the pane keeps at least half.
- **The width starts from `sidebar_width`, else a third of the terminal**, and from the first
  key on the app keeps the new number of columns in place of the configured one (in memory
  only, never written anywhere). Each key moves the width as drawn: narrower is `max(drawn - 2, 15)` unless drawn is
  already 15 or less; wider is `min(drawn + 2, half)` unless drawn is already at half. A
  configured width under 15 stays as configured; Alt-minus does nothing there and Alt-equals
  adds 2.
- **A terminal resize keeps the number** and the layout cuts it to half the new width at each
  frame, so a terminal shrunk and grown back gets it back. A key after a cut starts from the
  cut width, as drawn.
- **Zoom leaves the number alone**: the keys do nothing while zoomed, and the sidebar is back at
  the adjusted width when the zoom ends.
- **The help gets one row for both keys** (`M--/M-=`), short enough for one row at 59 to 60
  columns; the README key table follows; the headroom test drops from 3 to 2.
- **The log names the keys** `M--` and `M-=` when harbour takes them; a key forwarded to the
  session or editor is logged as input, as today.

## States

| State | Alt-minus / Alt-equals |
| --- | --- |
| Sidebar and pane both shown, from the sidebar | the border moves 2 columns; every session gets its new size at the next frame |
| Sidebar and pane both shown, from the Claude Code pane | the same; focus stays on the pane |
| From the editor | the editor gets the key, as today; the width is unchanged |
| Sent as legacy `ESC -` or `ESC =` | the session gets it, as today; the width is unchanged |
| At the narrowest, 15 columns (or a configured width under 15) | Alt-minus: nothing happens; Alt-equals: 2 columns wider |
| At the widest, half the terminal | Alt-equals: nothing happens; Alt-minus: 2 columns narrower |
| One column from a limit (odd half, or a configured odd width) | the border stops at the limit |
| A key held down | one press after another, 2 columns each, until a limit |
| Terminal under 60 columns (one side shown) | nothing happens; the number kept is unchanged |
| Zoomed | nothing happens; the width from before the zoom is back when it ends |
| A question, the help view or the picker shown | nothing happens (the question and help view are dismissed by any key; the picker takes every key) |
| The usage popup (#99) shown | nothing happens: it is dismissed by any key like the help view; checked once #99 is merged |
| No session | the border moves; the empty pane takes the rest |
| The terminal narrowed below the adjusted width's half | the sidebar is cut to half the new width; the number is kept |
| The terminal widened again | the sidebar is back at the adjusted width, or half the new width when smaller |
| A CJK card title at the narrowest sidebar | `▌ · 東京の倉…` in 9 columns without a time, `東京…` in 5 beside a 3-column time: cut by display width, never a half character, the border kept |
| What each session is told, and when | at the first frame after the key, every session (shown or not) is resized to the new pane: its PTY gets the new columns, its program SIGWINCH, once; the rows never change |
| Quit and start again | the width starts from `sidebar_width`, else a third |

What harbour reads from outside: only the key bytes above. A terminal or tmux without extended
keys sends the legacy form, so the session gets it; nothing else is read.

## Tasks

Each ship is whole: the new items ship with their caller.

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 25's after-merge cells, the
   release links of phases 24 and 25.
2. The keys move the border (`tests/app/width.rs`): Alt-minus and Alt-equals from the sidebar
   and from the Claude Code pane, the session never reading the key, every session told its new
   size, the log names. Red: today the key goes to the session and the border never moves.
   Then each guard, test first against the unguarded code, so each absence is seen red before
   its guard exists:
   - the limits (15, half): red while the move is unclamped;
   - under 60 columns, zoomed (and the width back after it), help and question shown: red
     while the move acts whatever is shown;
   - the editor and the legacy `ESC -`: red while every Alt-minus is taken;
   - a resize below and above the adjusted width: red while the number is cut when stored
     instead of when drawn.
   A test that still passes first is shown red by breaking the code it covers, and restored.
3. Through tmux (`tests/e2e/keys.rs`): both extended forms move the border and the session
   reads only the next key; `ESC -` with `extended-keys off` reaches the session. They test
   code from task 2, so each is shown red by breaking that code, and restored.
4. Help row (and the row shortened to keep the narrow help whole), the README key table, the
   help tests and the headroom number, `harbour --help`, the fragment, the Layout of
   `AGENTS.md`, `docs/keys.png` and `docs/help.png` shot again. The new help row's test is red
   until the row exists; the headroom test is red at 3 once it does, then set to 2.
5. Outcome and Hurdles.

## Outcome

## Hurdles
