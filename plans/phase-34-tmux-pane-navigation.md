# Phase 34: C-j and C-k move to the tmux pane below or above harbour

With vim-tmux-navigator in `tmux.conf` and harbour in the pattern of processes it sends C-h,
C-j, C-k and C-l to, C-j and C-k arrive inside harbour. In the sidebar they act as `j` and `k`
(the sidebar matches the letter whatever its modifiers); in Claude Code they are forwarded (C-j
is a newline there). So from the sidebar or Claude Code, C-j never reaches a tmux pane below
harbour, nor C-k one above. Neovim inside harbour already does it: it gets harbour's `TMUX` and
`TMUX_PANE`, and its navigator plugin runs `tmux select-pane -D` at its window edge.

## Verified facts

- **tmux 3.8** (`tmux -V` on the build machine). `display-message -p -t <pane> '#{pane_at_bottom}'`
  prints `0` when a pane lies below, `1` at the bottom edge; `#{pane_at_top}` the same above.
- **`select-pane -D -t <pane>` wraps**: from the bottom pane it selects the top one, so the edge
  check is what keeps C-j from wrapping around.
- **A zoomed window** reports `pane_at_bottom` 1 for its zoomed pane, so nothing moves.
- **A pane id that does not exist**: `display-message -p -t %9` prints an empty line and exits 0,
  so only an exact `0` counts as a pane in that direction.
- **Cost**: ten `display-message` calls took 73 ms on the build machine, about 7 ms each.
- **Keys**: harbour reads C-j as `KeyCode::Char('j')` with `CONTROL`, both from the legacy byte
  0x0a and from the extended forms (`src/input.rs`); C-k from 0x0b the same way.
- **harbour never runs tmux today**; a Claude Code session starts without `TMUX` and
  `TMUX_PANE`, an editor with harbour's (`src/session.rs`).

## Decided

- **A module `src/tmux.rs`**: `Tmux`, made from `TMUX` and `TMUX_PANE` when both are set and not
  empty (`Tmux::from_env`, read by `src/tui.rs`, never by the demo), and `select(direction)`:
  `tmux list-keys -T root C-j` (or `C-k`), so the move happens only while tmux binds that key
  itself (as vim-tmux-navigator does; review round 1: a plain tmux user's C-j newline and C-k
  kill in Claude Code must stay), then `tmux display-message -p -t <pane>
  '#{pane_active}#{window_active}<edge>'`, then, only on an exact `110 <harbour's terminal>`, `tmux select-pane -D`
  or `-U -t <pane>`. The first two share one deadline of 100 ms (as the Neovim calls do); a
  call past it is killed and counts as no move. `select-pane` gets 100 ms of its own and counts
  as a move once started unless it fails (review round 2). The check also compares
  `#{pane_tty}` with harbour's terminal. stdin and stderr are null. `tmux` is looked up on
  `PATH` once at start, relative entries from where harbour was opened, and the client finds
  harbour's server through `TMUX`.
  `Tmux::with_program`, a test seam on the `SEAMS` list, takes a program and its arguments in
  place of `tmux`, so the in-process tests hand it a `/bin/sh -c` script.
- **Where harbour asks**: in `key`, after the jump, zoom, side, swap, width and finder keys and
  before C-h and C-l, for C-j and C-k (Ctrl-Shift too, as for C-h and C-l, so Caps Lock changes
  nothing) whenever no shown editor has focus: the sidebar, Claude Code, an exited session, side
  by side Claude Code's half, zoomed. When it moved, the key is used up; otherwise it goes on as
  today (the sidebar selects the next or previous card, Claude Code gets the key). The usage
  popup, the picker, the changelog view, a question and the help keep the keys as today.
- **Outside tmux** (`TMUX` or `TMUX_PANE` unset or empty) nothing changes and tmux never runs.
- **Log**: a move writes `tmux pane=below` or `tmux pane=above` in place of the key's line.
- **Docs**: `harbour --help` names C-j and C-k in the vim-tmux-navigator line; the README says
  the same where it names C-h and C-l. `KEYS` gets no row: the move matters only with that
  pattern in `tmux.conf`, which `--help` explains, and the help view keeps its 20 rows. No view
  of the interface changes, so `docs/details.png` stays; `docs/help.png` is taken again.
- **Known limit**: the tmux calls run on the key thread, at most 200 ms a key.

## States

| State | C-j (C-k the same, above) |
| --- | --- |
| Not under tmux | as today |
| Under tmux, C-j (C-k) not bound in tmux's root table | as today; only `list-keys` runs |
| Harbour's pane or window not the active one (`synchronize-panes`, `send-keys`) | as today |
| `TMUX` and `TMUX_PANE` inherited from another tmux (a terminal started from a tmux shell) | as today: the pane's `#{pane_tty}` is not harbour's terminal |
| `select-pane` slower than its own 100 ms | the key is taken, never also forwarded |
| Under tmux, a pane below, sidebar focused | tmux focus moves to the pane below; the selection stays |
| Under tmux, a pane below, Claude Code focused (zoomed, side by side on its half, exited too) | tmux focus moves; Claude Code gets nothing |
| Under tmux, nothing below (bottom edge, zoomed tmux window) | as today: the sidebar selects the next card, Claude Code gets the key |
| A shown editor has focus (side by side its half too) | the editor gets the key; tmux is not asked |
| The usage popup, the picker, the changelog view, the help or a question | as today; tmux is not asked |
| `tmux` not on `PATH`, or exits non-zero | as today |
| tmux answers nothing within 100 ms (a stalled server) | killed; as today, the key late by at most 100 ms (200 ms when the move stalls) |
| `TMUX_PANE` names a pane that is gone | `display-message` prints no `0`; as today |
| The layout changes between the check and the move | the move follows tmux's layout at that instant (may wrap), accepted |

## Tasks

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 32's after-merge cells.
2. `src/tmux.rs`: `from_env`'s variables, `select` running the check and the move, the deadline
   (`tests/tmux.rs`).
3. C-j and C-k in the sidebar and in Claude Code, falling back when tmux says no; an editor
   keeps them (`tests/app/navigating.rs`); the log line.
4. The real tmux: `tests/e2e/navigating.rs` with a pane below and above harbour.
5. `harbour --help`, the README, the Layout of `AGENTS.md`, the Known limit, the fragment,
   `docs/help.png`.
6. Outcome, Hurdles, metrics.

## Outcome

- Shipped: under tmux, C-j and C-k from the sidebar or Claude Code move to the tmux pane below or
  above harbour, but only while tmux binds the key in its root table (vim-tmux-navigator does),
  harbour's pane and window are the active ones, the pane is the terminal harbour draws on, and a
  pane lies that way. Otherwise, and over a shown editor, the keys act as before.
- Review round 1 found that a plain tmux user with a split lost C-j (newline) and C-k (kill to
  end of line) in Claude Code. The binding check is the smallest fix that needs no setting; an
  opt-in setting can replace it later without anyone losing anything.
- Round 2 found a key that could be both a move and input when tmux was slow, and `TMUX`
  inherited from another tmux moving that tmux. The move now has its own deadline and takes the
  key once started; the check compares `#{pane_tty}` with harbour's terminal. `tmux` is found on
  `PATH` once at start, relative entries from where harbour was opened.
- Round 3 was clean. Left as minor: Ctrl-Shift-J and K count as C-j and C-k even when tmux's
  extended keys deliver them past the navigator's C-j binding; `list-keys` with a key argument
  needs tmux 3.1 or later, so an older tmux never moves; a deleted launch directory turns the
  feature off; an EINTR in the poll counts as no answer; the tests compare the calls log as one
  string.
- Numbered 34, since #124 claimed 33; the branch keeps its name.

## Hurdles

- `process` The phase number collided with an open pull request that claimed 33 first, so the plan was renamed mid-phase → none
- `process` Tasks 2 and 3 landed in one commit, since the seam and its first caller were written together → none
- `tooling` `scripts/screenshots.sh` re-renders every image, so each run needed the untouched ones restored by hand → backlog
- `design` The first design took C-j and C-k from Claude Code whenever a pane lay that way, which the plan's state table missed: whether tmux binds the key at all → code
