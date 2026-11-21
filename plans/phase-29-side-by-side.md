# Phase 29: Claude Code and its editor side by side

Issue #42. Alt-\ hides the sidebar and splits the terminal evenly: the selected session's Claude
Code on the left, its editor on the right. C-l and C-h move the focus between the halves. The
mode needs 120 columns, and only the focused half shows a cursor.

## Verified facts

- **The bytes**, captured in a scratch tmux 3.7c server (`tmux -L <scratch> -f <scratch conf>`,
  `TMUX_TMPDIR` a short `mktemp -d /tmp/...` folder) by a program that turns on modifyOtherKeys
  2 (`\e[>4;2m`, what harbour writes) and prints every read in hex:

  | tmux setup | `send-keys 'M-\'` | `send-keys Escape '\'` |
  | --- | --- | --- |
  | `extended-keys on`, `extended-keys-format xterm` | `1b 5b 32 37 3b 33 3b 39 32 7e` (`CSI 27;3;92~`) | `1b 5c` |
  | `extended-keys on`, `extended-keys-format csi-u` | `1b 5b 39 32 3b 33 75` (`CSI 92;3u`) | `1b 5c` |
  | `extended-keys always`, xterm | `CSI 27;3;92~` | `1b 5c` |
  | `extended-keys off` (tmux's default) | `1b 5c` | `1b 5c` |

- **Esc then `\` typed into tmux**: bytes written by an outer scratch tmux into an inner tmux
  client (`extended-keys on`, xterm). `\e\` in one write reached the pane as `CSI 27;3;92~`, the
  extended Alt-\; Esc and `\` 800 ms apart reached it as `1b`, then `5c`. As for Alt-z (phase 25),
  tmux turns a fast Esc then `\` into Alt-\ before harbour reads a byte, and no rule over bytes
  can tell them apart.
- **Harbour's reader** (`src/input.rs`) reads `CSI 27;3;92~` and `CSI 92;3u` as
  `KeyEvent(Char('\\'), ALT)`, and `ESC \` (which is also the string terminator) as the same
  key with `input::ESCAPED` set: `[0x1b, rest]` parses `\` as a plain character and marks it.
  C-\ is `0x1c`, read as `Char('\\')` with CONTROL, a different key.
- **Neovim 0.12.6** maps no `<M-\>` or `<A-\>`: neither appears in its runtime docs
  (`$VIMRUNTIME/doc/*.txt`). Like every unmapped Alt key it reads Alt-\ as Esc then `\` in
  every mode (phase 25), so taking it from the editor would steal `Esc \`. Its terminal mode
  leaves with C-\ C-n, not Alt-\.
- **Claude Code** binds no Alt-\ by default: its keybindings page
  (code.claude.com/docs/en/keybindings, read 2026-11-21 for phase 25) lists Meta+P, Meta+O,
  Meta+T, Meta+Up and Meta+Down as its Meta bindings. A user may still bind `meta+\`. Not run.
- **Redraw at half width**: the clean Neovim (`nvim --clean`, what `tests/fixtures/nvim/nvim`
  runs) in a scratch tmux pane, 120 then 58 then 120 columns: at 58 it rewrapped the long line
  and moved its ruler to the new right edge at once; back at 120 it drew the line whole again.
  The agent fixtures (`tests/fixtures/agent.sh` and the `/bin/sh` scripts of `tests/app/`) never
  redraw: their printed lines stay as vt100 keeps them, cut at the new width, and a line cut at
  58 columns stays cut after the mode is left. Claude Code redraws on SIGWINCH like Neovim.
- **Every frame resizes every session to the pane** (`compose` in `src/app/draw.rs`);
  `Session::resize` acts only when the size changed and never on an exited session.
- **The terminal write order of a frame** (`tui::frame`): `ESC[?25l` hides the cursor, ratatui
  writes the diff of the whole buffer, the cursor shape is sent, then one `MoveTo` and `Show`
  for the one cursor position the app gives, if any.

## Decided

- **The key rule is Alt-z's** (phase 25): harbour takes Alt-\ only in an extended form
  (`CSI 27;3;92~` or `CSI 92;3u`: Alt, a press, not marked `ESCAPED`), and never while an editor
  has focus. The legacy `ESC \` and any Alt-\ while the editor has focus reach the program as
  today. The capture shows nothing that would call for another rule: Neovim reads Alt-\ as Esc
  then `\`, as it reads Alt-z.
  - The issue says the key works "from anywhere"; from the editor half it does not, for the
    reason above.
  - Cost: without extended keys nothing happens and Claude Code gets `ESC \`; inside tmux with
    extended keys, Esc then `\` typed fast in Claude Code toggles the mode.
- **Leaving the mode**:
  - from the Claude Code half: Alt-\ (the sidebar comes back, Claude Code keeps the focus) or C-h
    (the sidebar comes back focused, as from a zoom);
  - from the editor half: C-h first (Neovim: in normal mode, from its leftmost window), which
    moves the focus to Claude Code, then Alt-\ or C-h;
  - from Neovim in insert mode: Esc, then C-h, then Alt-\.
  `harbour --help` says it in one paragraph, like M-z's.
- **Entering**: from the sidebar or the Claude Code pane of a running session on a terminal of
  120 columns or more. The focus goes to the Claude Code half (from the sidebar, which hides, the
  pane is the only place left). Its editor for Claude Code's directory is started as C-l would
  start it, or the one already running is shown on the right. When it cannot start, the mode is
  not entered and C-l's message shows.
- **One flag, `side_by_side`**, and the focused half is the entry's `on_editor`: `true` is the
  editor half. `zoomed` and `side_by_side` are never both set: Alt-z from the Claude Code half
  turns side by side into a zoom of Claude Code, Alt-\ from a zoom turns it into side by side.
- **The halves**: `Layout::horizontal([Ratio(1, 2), Ratio(1, 2)])` over the whole terminal, each
  half a block of its own. The focused half has the thick accent border, the other the plain one.
- **The border**: the left half's top line holds the zoom's headline (dot, title, place, age)
  then `claude`; the right half's holds the editor's name and directory, the counts at its right
  end. Names are bold over the focused half and dim over the other, with their Nerd Font icons
  when `nerd_font` is on. `tabs::editor_named` fits the editor's name alone; the left name is
  `claude` as `tabs::strip` and `tabs::iconed` give it.
- **Sizes**: in the split every Claude Code gets the left half's inner size and every editor the
  right half's, selected or not, so a jump between sessions sends no new size. New sessions
  start at the left size, new editors at the right one. Outside the split both get the pane, as
  today.
- **Mouse**: a click focuses the half it lands in. A gesture belongs to the half where its
  button went down and to the program shown there then (`Gesture::Pane(id)` already names it);
  a wheel or a move goes to the half under the pointer and moves no focus.
- **Log**: `key name="M-\\"` when harbour acts on the key, nothing named when it reaches a
  program.

## States

| State | What the user sees |
| --- | --- |
| Alt-\ from the sidebar or Claude Code pane, a running session with an editor, 120 columns or more | sidebar hidden; Claude Code left, its editor right, each sized to its half; the Claude Code half focused, the only cursor there |
| The same, no editor running yet | the editor starts at the right half's size, as C-l would start it; until it draws, the right half is empty; focus on Claude Code |
| The same, the editor cannot start (program missing, directory gone) | the mode is not entered; C-l's message (`Could not start <editor>: …`) shows where it does today |
| Alt-\ under 120 columns | nothing happens; the key reaches no program |
| The terminal shrinks under 120 columns while side by side | the focused half alone on the whole screen with the zoom border, as Alt-z shows it, both programs at that size; at 120 or more again, side by side is back |
| Alt-\ with no session, or on a stopped card | nothing happens |
| Alt-\ on an exited session | nothing happens |
| Alt-\ again from the Claude Code half | the sidebar comes back with the configured `sidebar_width`, Claude Code focused, its editor still running |
| C-l from the Claude Code half | the focus moves to the editor half; the editor gets the next key |
| C-h from the editor half | the editor's C-h rule (`wants_ctrl_h`): Neovim in normal mode from its leftmost window gives the focus to Claude Code; otherwise the editor gets C-h |
| C-h from the Claude Code half | the mode ends, the sidebar comes back focused |
| Alt-\ while the editor has focus | the editor gets it (Esc then `\` for Neovim); the mode is unchanged |
| An editor that asked for extended keys has focus | it gets Alt-\ in its own encoding, as any key; the mode is unchanged |
| Alt-\ sent as legacy `ESC \` | Claude Code gets `ESC \`; nothing else happens |
| Alt-z from the Claude Code half | Claude Code alone, zoomed; Alt-z again brings the sidebar back |
| Alt-\ from a zoom on Claude Code | side by side |
| The width keys (M-- / M-=) from the Claude Code half | nothing happens, as while zoomed; from the editor half the editor gets them |
| C-1 to C-9, or a pick by name (phase 28's Alt-s), to a running session | that session side by side; its editor started as C-l would, or the running one shown; the half that session last focused keeps the focus |
| The same, its editor cannot start | that session's Claude Code alone, as Alt-z shows it, with the message at the pane's bottom |
| The same to a stopped card that resumes | the resumed session side by side, its editor started |
| The same to a stopped card that cannot resume, or an exited session | the mode ends, the sidebar comes back with the message, as from a zoom |
| `harbour edit` opens a file in a session | that session side by side, the file in its editor on the right, the editor half focused |
| The usage popup (u) or phase 28's session picker over the mode | u is a sidebar key, so it opens only after leaving; a picker opened from the mode draws over both halves, its keys go to it, and closing it leaves the mode as it was |
| The editor exits | Claude Code alone, as Alt-z shows it; Alt-z brings the sidebar back |
| The session exits while side by side | the mode ends, the sidebar is back, the pane keeps the focus, the exited screen dimmed and never resized again |
| An exited session's screen in the split (a session that exited before another was shown side by side) | never resized: it keeps the size it had |
| `nerd_font` on | the names keep their icons:  claude on the left,  nvim: <dir> on the right |
| `sidebar_width` set in the config | ignored while side by side; the sidebar comes back at it |
| `accent` set in the config | the focused half's border takes it |
| The demo (`harbour --demo`) | side by side with the demo's made-up editor on the right |
| Quit and start again | not side by side |

### Mouse

| Gesture | What happens |
| --- | --- |
| Click in the left half | focus on Claude Code; Claude Code gets the press if it asked for the mouse |
| Click in the right half | focus on the editor; the editor gets the press if it asked |
| Click on the border between the halves (either half's border column) | nothing: no focus change, no program gets it |
| Drag that starts in the left half and crosses the middle | Claude Code gets every drag event and the release, clamped to its half; the editor gets nothing |
| Drag that starts on the border | no program gets it |
| Wheel over the unfocused half | the program under the pointer gets it, if it asked; the focus stays |
| Move with no button (any-motion mode) | the program under the pointer, if it asked |
| Another session shown mid-gesture (C-1) | the gesture ends, as today |

### Text

| Width | What the border shows |
| --- | --- |
| 120 (the narrowest split), halves of 60 | left: dot, the title cut to fit, `claude`; right: `acme-edit: ~/acme-api` and the counts |
| A CJK title (`支払いの修正`) | cut by display columns with `…`, never wider than the half |
| A CJK directory (`~/コード/acme-api`) | the directory loses its start first (`…/acme-api`), by display columns |

## Per frame

When the mode toggles, the focus moves, or the terminal crosses 120 columns, one frame:

1. `settle_side` drops the mode when the pane lost focus or the session ended, and turns it into
   a zoom when the session has no editor.
2. The sizes: every Claude Code resized to the left half's inner size, every editor to the right
   one's (or both to the pane outside the split). A program whose size changed gets one SIGWINCH
   now, before anything is drawn.
3. Both halves drawn into ratatui's buffer from each program's current screen: a program that has
   not yet redrawn at its new size shows its old screen as vt100 resized it (cut or padded).
4. Both borders with their titles.
5. ratatui writes the buffer's diff, with the cursor hidden since the frame's first write.
6. The shape of the focused program's cursor, then one `MoveTo` and `Show` inside the focused
   half only; the unfocused half never shows one.

Between two writes the user sees the old frame with no cursor. A program redraws at its new size
one or more frames later (Neovim: at once; Claude Code: at its next paint), so for a few frames
after a toggle a half can show the old screen cut at the half's width.

## Tasks

Each ship is whole: a new pub item ships with its first caller. Every test of an absence or a
default is shown red by breaking the code it covers, then restored, and said so.

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 27's after-merge cells.
2. The mode (`src/app/side.rs`, `tests/app/side.rs`): Alt-\ from the sidebar and from Claude
   Code shows two halves, Claude Code focused, keys reaching it; the editor started at the right
   size; each program sized to its half. Red: today the key reaches Claude Code and nothing
   splits. Then each guard red first: no session, stopped, exited, under 120, legacy `ESC \`,
   the editor focused, an editor that cannot start, Alt-\ again, Alt-z both ways, the width keys.
3. Focus and settle: C-l and C-h between the halves, C-h from Claude Code leaving, the editor
   exiting, the session exiting, shrinking under 120 and back, the cursor only in the focused
   half, jump and `harbour edit` keeping the mode.
4. The border (`tabs::editor_named` with its first caller): headline and names in each half,
   counts at the right, CJK, `nerd_font`, accent.
5. The mouse: focus by click, the drag across the middle, the wheel, the border.
6. Through tmux (`tests/e2e/keys.rs`): both extended forms split and the session reads only the
   next key; `ESC \` with `extended-keys off` reaches it.
7. The help row, the README key table, the help headroom test renamed, `harbour --help`, the
   log name, the fragment, the Layout of `AGENTS.md`, the `side-by-side` shot and
   `docs/keys.png`, `docs/help.png` shot again.
8. Merge `origin/main` once phase 28 lands, its metrics cells, Outcome and Hurdles.

## Outcome

- Alt-\ hides the sidebar and splits the terminal in two even halves: Claude Code on the left,
  its editor (opened as C-l opens it, sized to its half) on the right, Claude Code focused. Alt-\
  again, or C-h from the Claude Code half, brings the sidebar back.
- The key rule is Alt-z's: harbour takes Alt-\ only as an extended key (`CSI 27;3;92~` or
  `CSI 92;3u`), never the legacy `ESC \`, and never while the editor half has focus. From the
  editor half: C-h first (Neovim in normal mode, leftmost window); from Neovim insert mode, Esc,
  then C-h, then Alt-\.
- C-l and C-h move the focus between the halves; a click focuses the half it lands in and a
  gesture stays with the half it began in; only the focused half draws a cursor.
- Under 120 columns Alt-\ does nothing; a terminal that shrinks below 120 while side by side
  shows the focused half alone with the zoom border, and the split comes back when it widens.
- Each frame drops the mode once the pane loses focus, the session ends or no session is
  selected; with no editor left it turns into a zoom. Ctrl-1 to Ctrl-9 keep the mode for the
  session they show.
- The borders: the headline and `claude` over the left half, the editor's name and directory and
  the counts over the right, the focused half's names bold. The mode lives in `src/app/side.rs`;
  `src/tabs.rs` gained `claude` and `editor_named`.
- The help gained one row (`M-\`), so the C-l/Enter row was shortened to "focus or resume; from
  Claude Code, its editor", the wording phase 28 uses, and the help again fills all 20 rows of a
  20-row terminal; issue #98 tracks the lasting fix. The log names the key `M-\`.
- `docs/side-by-side.png` is the new `side-by-side` shot; `docs/keys.png` and `docs/help.png`
  changed with the help.
- Not covered by a test: `harbour edit` opening a file while side by side keeps the mode (read
  from the code: the holder's editor is shown and the selection kept, and the settle keeps the
  mode while that editor runs).

## Hurdles

- `design` the new help row overflowed the 20-row help beside the narrowest sidebar, so an
  existing row had to be shortened → code
- `process` phase 28, built in parallel, edits the same help rows, README table, screenshots and
  AGENTS.md lines, so this branch waits for its merge to resolve them → none
- `tooling` foreground waits are blocked or moved to the background, so a ship is waited on with
  an until-loop, with no edits while `ship.sh` stages → none
