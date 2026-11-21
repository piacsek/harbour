# Phase 25: zoom, the selected session on the whole screen

Issue #36. Alt-z hides the sidebar and gives the selected session the whole screen; the pane's
top border then carries what the sidebar showed about it (its dot, title, `repository · branch`
and time) and the counts. Phase 25 is a multiple of 5, so it opens with the sweep of phases 20
to 24 (`### Sweep` under `## Outcome`).

## Verified facts

- **Alt-z bytes**, captured in a scratch tmux 3.7c server (`tmux -L <scratch> -f /dev/null`,
  `TMUX_TMPDIR` a short `mktemp -d /tmp/...` folder, since the scratchpad path makes the socket
  path too long), by a program that turns on modifyOtherKeys 2 (`\e[>4;2m`, what harbour
  writes) and prints every read in hex:

  | tmux setup | `send-keys M-z` | `send-keys Escape z` | `send-keys -H 1b 7a` |
  | --- | --- | --- | --- |
  | `extended-keys on`, `extended-keys-format xterm` | `1b 5b 32 37 3b 33 3b 31 32 32 7e` (`CSI 27;3;122~`) | `1b 7a` | `1b 7a` |
  | `extended-keys on`, `extended-keys-format csi-u` | `1b 5b 31 32 32 3b 33 75` (`CSI 122;3u`) | `1b 7a` | `1b 7a` |
  | `extended-keys always`, xterm | `CSI 27;3;122~` | `1b 7a` | `1b 7a` |
  | `extended-keys off` (tmux's default) | `1b 7a` | `1b 7a` | `1b 7a` |
  | `extended-keys on`, the program did not ask | `1b 7a` | `1b 7a` | `1b 7a` |

- **Esc then z typed into tmux**: bytes fed to a tmux client the way a terminal sends them (by
  an outer tmux, and by a Python pty client after a 4 s settle). With `extended-keys on`, ESC
  then `z` sent 0, 2, 5, 50 and 300 ms apart reached the pane as the extended Alt-z, in both
  formats, with `escape-time` 10 and 500 alike; a lone ESC reached the pane alone only after
  0.3 to 1.3 s. With `extended-keys off` every one arrived as one read of `1b 7a`. So inside
  tmux, tmux itself turns a fast Esc then z into Alt-z before harbour reads a byte, and no rule
  over bytes can tell the two apart there. Outside tmux a terminal sends ESC and `z` as two
  keys; harbour's reader takes `ESC z` in one read as the legacy Alt-z.
- **Alt-z with Caps Lock on**: a terminal sends Alt with `Z` and no Shift (xterm's
  `CSI 27;3;90~`). Fed by an outer tmux to an inner one with `extended-keys on`, the pane read
  `CSI 27;3;90~` (format xterm) or `CSI 90;3u` (csi-u); `CSI 90;3u` from outside read the same,
  and kitty's Caps Lock bit (`CSI 122;67u`) reached the pane as plain Alt-z (tmux drops the bit).
  Caps Lock itself cannot be pressed in the scratch server; `send-keys M-Z` gives the same bytes.
- **Harbour's reader** (`src/input.rs`) reads `CSI 27;3;122~`, `CSI 122;3u` and the legacy
  `ESC z` as the same `KeyEvent(Char('z'), ALT)` today.
- **Claude Code** binds no Alt-z by default: its keybindings page (code.claude.com/docs/en/keybindings,
  read 2026-11-21) lists Meta+P, Meta+O, Meta+T, Meta+Up and Meta+Down, and Alt+V on Windows and
  WSL, as its only Alt bindings; a user may still bind `meta+z` in `keybindings.json`. Not run.
- **Neovim 0.12.6** reads an unmapped Alt key as Esc then the key in every mode: in a clean
  headless Neovim, `<M-j>` fed in normal, insert and visual mode moved the cursor one line down
  and left Neovim in normal mode each time (`:h i_ALT`, `v_ALT`, `c_ALT` document it for insert,
  visual and cmdline mode). So `Esc zz` typed fast in a Neovim pane inside tmux with extended
  keys reaches Neovim today as `<M-z>z`, which Neovim runs as Esc, `zz`.

## Decided

- **Harbour takes Alt-z only in an extended form** (`CSI 27;3;122~` or `CSI 122;3u`), and never
  from an editor: the legacy `ESC z` and any Alt-z while the editor has focus reach the session
  or the editor as today. The reader marks a key it read after an ESC prefix (a key state bit
  of its own, `input::ESCAPED`), so the app can tell the two apart.
  - Why: outside tmux, Esc then z typed fast arrives as `ESC z`, which never zooms. Inside tmux
    no byte rule can tell them apart (above), and Neovim runs Alt-z as Esc then z in every mode,
    so taking it from Neovim at all would steal `Esc zz`. A mode check would not help, normal
    mode included.
  - Cost: without extended keys (tmux's default `extended-keys off`, a terminal that ignores
    modifyOtherKeys) Alt-z never zooms; from the editor it never zooms, so leave a zoomed editor
    with C-h first (it shows Claude Code, still zoomed); in a Claude Code pane inside tmux with
    extended keys, Esc then z typed within tmux's wait (seen up to 300 ms) zooms, and Claude Code
    gets neither key. `harbour --help`, the PR body and the fragment say so.
- Zoom is a flag of the app, `zoomed`, that holds only while the pane has focus and the selected
  session's Claude Code runs. Each frame drops it otherwise, before the layout: C-h from Claude
  Code (focus to the sidebar), the session exiting, a refused resume. It is never saved.
- Alt-z from the sidebar on a running session zooms and focuses the pane; from the Claude Code
  pane it toggles; with no session, on a stopped card or an exited one it does nothing. A
  session never receives it (the editor rule aside).
- Leaving zoom keeps focus on the pane; the sidebar comes back at its configured width.
- Ctrl-1 to Ctrl-9 while zoomed jump and stay zoomed (the jump focuses the pane); `harbour edit`
  showing another session keeps the zoom the same way. A jump to a stopped card zooms the
  resumed session; a refused resume ends the zoom.
- The zoomed border, in a module of its own (`src/zoom.rs`), on the pane's top border:
  - left: the dot, the title, `repository · branch` and the time in its state, then the names
    (`claude | <editor>: <directory>`, with icons under `nerd_font`) as today;
  - right: the counts, by the sidebar title's rule (`state::title`, counts above zero);
  - fitted by priority: the counts first; then the session info is guaranteed its dot, up to
    16 columns of title and the time; the names (by the rules of `src/tabs.rs`, the directory
    losing its start first) fit in what remains; whatever the names leave goes back to the
    session info (more title, then the place). The time stays while the title has a column, as
    on the card; the title and place are cut by `text::fit`; the dot alone needs 3 columns
    (` ● `), under that the info goes.
- The help gets one row (`M-z`), the README key table with it, and the headroom test drops to 3.
- The log names the key `M-z` when harbour acts on it; one forwarded to the session or editor
  is logged as input, as today.

## States

| State | What shows |
| --- | --- |
| Zoomed, a session running | the pane alone on the whole screen, its top border: dot, title, `repository · branch`, time, the names, the counts at the right |
| Zoomed, terminal too narrow for the whole border | the counts stay; the session info keeps its dot, up to 16 columns of title and the time; the names shorten by `tabs.rs`'s rules into the rest (the directory loses its start first); what they leave goes back to the info (more title, then the place) |
| Zoomed at 80 columns, an editor directory of about 45 columns | `┏ ● Fix the login  acme-api…  3m  claude \| acme-edit: …/acme-api  sessions ● 1 ┓`: the dot and the title stay, the directory loses its start |
| Zoomed from the sidebar while a failure message shows at its bottom | the message is cleared, as a key over the pane clears it, so it never covers the zoomed pane's bottom rows |
| Zoomed, the narrowest width the border still fits | 11 columns: `┏ ● …  ●1 ┓`, the short counts and the dot with one column of title; under that the info goes, then the names. The 11 columns hold for a single kind of count: with several kinds the short counts alone can take the whole room, and the dot and title go |
| Zoomed, a CJK title and branch | cut by display width (`text::fit`), never a half character, never wider than the border |
| Zoomed, terminal under 60 columns | today's narrow mode with the pane shown, plus the zoomed border |
| Alt-z with no session or on a stopped card | nothing happens |
| Alt-z on an exited session | nothing happens |
| Alt-z while the editor has focus | the editor gets it, as today; zoom unchanged |
| Alt-z from the sidebar on a card showing its editor (the sidebar reached by a click) | zooms the session's Claude Code, as if C-h had been pressed in the editor first |
| C-l while zoomed | the editor shows, still zoomed; Alt-z there reaches the editor and does nothing else; C-h returns to Claude Code, still zoomed, where Alt-z works. From Neovim in insert mode: Esc, then C-h (from the leftmost window), then Alt-z or C-h; `harbour --help` says "from Neovim, Esc then C-h" |
| Alt-z with Caps Lock on | the terminal sends Alt with `Z` (tmux passes `CSI 27;3;90~` or `CSI 90;3u`); it zooms like Alt-z |
| Zoomed, a session with no title | the place alone after the dot, no gap |
| Leaving zoom with more cards than fit | the sidebar scrolled as it was before the zoom (the cards are neither drawn nor scrolled while their area has no height; the narrow layout showing the pane takes the same path) |
| Alt-z sent as legacy `ESC z` | the session gets it, as today |
| C-h from a zoomed Claude Code pane | zoom ends, the sidebar is back and has focus |
| C-h from a zoomed editor | Claude Code shows, still zoomed (or Neovim takes the key, by today's rule) |
| The zoomed session exits or is removed | zoom ends, the sidebar is back |
| Question, help view or picker while zoomed | shown as today; zoom kept underneath (each opens only from the sidebar, so only a zoom of a session that keeps the pane can sit under them) |
| `harbour edit` opens a file in another session while zoomed | that session is shown, still zoomed |
| Ctrl-1 to Ctrl-9 while zoomed | the jumped-to session, still zoomed |
| Zoom ends | the sidebar at its configured `sidebar_width`, or a third |
| Quit and start again | not zoomed |

What harbour reads from outside: only the key bytes above. A terminal or tmux without extended
keys sends the legacy form, so Alt-z goes to the session; nothing else is read.

## What a frame writes when zoom toggles

1. `tui::frame` writes `ESC[?25l` (cursor hidden) first, as every frame.
2. `compose` drops a zoom that no longer holds, then lays out: zoomed, the pane takes the whole
   screen (no sidebar); otherwise the split as today.
3. Every session is resized to the new pane (its program gets SIGWINCH and repaints later; until
   then the frame shows vt100's screen cut or padded to the new size, as any width change does).
4. The pane's block with the zoomed or plain top border, then the session's screen, then the
   overlays (question, failure, help, picker).
5. The cursor position is taken from the session at the new pane's origin; the shape follows.
6. `tui::frame` sends the shape, then `MoveTo` and `Show` when the pane has focus.

## Tasks

1. The sweep, this plan, the `PLAN.md` and `plans/metrics.md` rows, phase 22's after-merge cells.
2. The reader marks a key read after an ESC prefix (`input::ESCAPED`).
3. Alt-z zooms: the flag, the layout without the sidebar, the keys (sidebar, Claude Code pane,
   editor, legacy), the frame's settle (C-h, exit), the log name.
4. The zoomed border (`src/zoom.rs`): info, names, counts, fitted.
5. Help row, README key table, `harbour --help` (the rule and its cost), the e2e through tmux
   with `extended-keys on`, the `zoom` shot, the fragment, the Layout of `AGENTS.md`.
6. Outcome and Hurdles.

## Outcome

### Sweep

Range: phases 20 to 24. `scripts/sweep-report.sh` starts after phase 20, the last `### Sweep`,
so it reports phases 21 to 24; phase 20's own `## Outcome` and `## Hurdles` were read by hand.
Phase 24 merged while this phase was being built and was taken in with a merge of `main`. The
report counted 9 hurdles for phases 21 to 23 (`test-seam` 5, `api-shape`, `design`, `process`
and `review-loop` 1 each), phase 24 added 3 (`test-seam`, `process`, `review-loop`) and phase
20 added 1 (`test-seam`); 6 of the 13 went `→ none`. Ready, merged and released of phase 24
were read from GitHub (phase 22's were written by phase 24). The owner decided in advance that every
finding becomes a proposed issue or is declined, no rule and no backlog line is applied, and
nobody is asked; the orchestrator files the issues.

- A test that passed before the change existed, in all five phases: phase 20's three cursor
  tests passed whatever their rule did (the sidebar already hid the cursor), phase 21's width
  test asked for a third, the default, phase 22's out-of-range key drew nothing either way, phase
  23's off path did not change, phase 24's cut, CJK and ended-card tests passed against the finished join, and this phase's
  leaving-zoom and `ESC z` tests were green first too. Each is a test of an absence or of a default; two were caught by
  review, not by the red run → issue #96.
- A helper shipped before its first caller and stopped by the dead-code gate: phase 23 (`→ none`)
  and, in the last range, phases 17 and 19. Twice `→ none` across two sweeps → issue #97.
- The help view has no spare row in the narrow layout: phase 22 freed one by shortening C-h's
  text, and this phase meets the same wall for `M-z` → issue #98.
- No "Sweep due before phase 25" issue was opened: phase 25 started while phase 24 was unmerged
  → declined: the class of #88.
- Phase 22's end-to-end tests used Ctrl-3 as a sample key the phase then claimed → declined:
  one-off, fixed in code; this phase checked `M-z` in the tests before taking it.
- Phase 21's file-name rule applied to one side of each program → declined: one-off, fixed in
  code in the round that found it.
- Phase 22's round-1 test that did not wait between two sessions → declined: fixed in code, the
  wait every other test already makes; load-sensitive tests are #87.
- Phase 21's tmux colour form (`38;5;N`) → declined: a fact of tmux's capture, fixed where it
  arose.
- Phase 24's edit of `scripts/screenshots.sh` while a ship ran, which `git add -A` would have
  taken into that commit (`→ none`, once) → declined: one-off, kept out by hand that time.
- Phase 24's cut that measured a part's first grapheme apart from the joined line, found in
  review → declined: one-off, fixed in code in that round.

Metrics (from `plans/metrics.md`): review rounds 1, 2, 2, 1, 1 for phases 20 to 24; start to
release 30 min 47 s, 1 h 29 min 41 s, 1 h 34 min 18 s, 42 min 7 s and 47 min 41 s. The Claude
reviewer per round runs from 103532 to 169182: phase 21's two rounds (157086, 169182) sit above
the last sweep's top of 149335, phases 22 to 24 are back at 115281 to 124912; one phase, no
trend. Builder notices: 1, 2, 2, 1 and 1 per phase, each between 90614 and 164528, the last ship
of each unknown. The orchestrator's context grew from 521726 (phase 20's start) to 670656
(phase 24's end), in one session shared by every phase of the range and a fix branch, so no two
rows compare. Process drift: none beyond the missing sweep issue; every phase had a worktree, a
review and a metrics row.

### Facts and decisions

- Alt-z reaches harbour as `CSI 27;3;122~` (tmux `extended-keys on`, format xterm) or
  `CSI 122;3u` (format csi-u), and as `ESC z` with `extended-keys off`, captured in tmux 3.7c
  (Verified facts). `tests/e2e/keys.rs` pins all three through a scratch tmux server: both
  extended forms zoom and the session reads only the next key; `ESC z` reaches the session as
  `^z` after an escape and zooms nothing.
- The reader marks a key read after an ESC prefix with `input::ESCAPED`, a key state bit of its
  own (`0x80`), so the app takes only the extended form.
- The zoomed border lives in `src/zoom.rs` (`info`, the session's part) and `src/app/zoom.rs`
  (the counts, the names and the headline from the app's facts). The place is cut first, then
  dropped once fewer than 2 columns are left after the title, then the title is cut; the time
  stays while the title keeps a column. The narrowest border measured is 11 columns,
  `┏ ● …  ●1 ┓`, the counts in their short form.
- The help gained the `M-z` row and lost a line in the narrow layout; the `q` row was shortened
  to "quit, after a yes while anything runs" (it said "while a session or editor runs" and
  wrapped at 59 to 60 columns), the cheapest row to shorten without losing what it says. The
  headroom test now asserts 3.
- Review round 1: leaving zoom scrolled a long sidebar to the selected card, since the cards
  were drawn into an area of no height and the scroll for height 0 forced the offset; the
  sidebar now neither draws nor scrolls its cards while it has no height, which also keeps the
  narrow layout's scroll across a visit to the pane. Alt-z from the sidebar on a card that showed
  its editor now zooms Claude Code; Alt-Z (Caps Lock) zooms; a missing title shows the place
  alone.
- Review round 2: Alt-z from the sidebar clears a failure message shown at its bottom, which
  otherwise moved onto the zoomed pane's bottom rows; the session info is guaranteed its dot,
  up to 16 columns of title and the time before the names fit, since a long editor directory
  left it nothing at 80 columns. With several kinds of count the short counts alone can exceed
  the narrowest border, which drops the dot and title; not acted on. 2 review rounds.
- `scripts/screenshots.sh` turns on `extended-keys` for the `zoom` shot alone, so `send-keys M-z`
  arrives as the extended Alt-z, and turns it off after it, so the other shots are as before.

## Hurdles

- `test-seam` The leaving-zoom border test and the `ESC z` end-to-end test passed on their first run, being tests of an absence; each was shown red by breaking the code → none
- `tooling` The tmux socket path under the scratchpad was too long for a Unix socket, so the byte capture ran with `TMUX_TMPDIR` a short folder under `/tmp` → none
- `process` Phase 24 merged while this phase was being built and was taken in with a merge of `main`, its sweep and metrics rows included → none
- `review-loop` Round 1 found the long sidebar scrolled to the selected card after a zoom (the cards drawn into an area of no height), a zoom from the sidebar that focused an editor Alt-z cannot leave, Caps Lock's `Z` and a gap before the place without a title → code
