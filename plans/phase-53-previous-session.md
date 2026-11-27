# Phase 53: the prefix twice switches back to the previous session

Moving back and forth between two sessions takes the prefix and a digit today, and the digit
changes as cards are added or move (#240). Pressing the prefix twice now switches to the session
selected before the current one, as Alt+Tab does, so two presses toggle between the last two.
Today the prefix twice sends the prefix key to the focused program (`src/app/prefix.rs`);
programs still need C-/ (C-_, undo in Claude Code's prompt), so that moves to a new action.

## Verified facts

- **The selected card is what harbour shows.** `App::selected` is an index into `sessions`; the
  pane shows the selected card's session or editor, and every way of moving (`j`/`k`, a jump,
  the finder, `n`, a click on a card, `harbour edit`, a removal) changes that index.
- **An index is not an identity.** Cards move (`J`/`K`, the regroup on every frame), and a
  removal shifts every card after it. A running card's `Entry::id` is unique and never changes;
  a stopped card's is 0 until it resumes, and its `remembered` conversation id names it.
- **One hook follows every event.** `log_changes` runs after each key, paste, mouse event,
  frame and stop, so it sees every selection the user could see.
- **Ctrl held already reaches the prefix as the prefix.** `Keymap::after` returns
  `After::Prefix` for C-/ and C-_ whatever Shift does, so C-/ C-/ with Ctrl held is the prefix
  twice.
- **`p` is free.** No action, and none of the sidebar's own keys, uses it.

## Decided

- The owner's: the prefix twice switches to the previous session; sending the prefix moves to a
  new bindable action, `send_prefix`, default `p`, in `[keys]` like the others.
- The orchestrator's: "previous" is by identity, not by number; a jump, the finder, `n` and
  `j`/`k` all count as a change of selection; an ended or closed previous session does nothing
  and says so on the message line.
- The builder's: **the switch is a jump to that card.** It selects it and focuses its pane
  exactly as the prefix then a digit does (side by side stays side by side, zoomed stays
  zoomed, the card's editor shows when it showed last).
- The builder's: **harbour notes the selection in `log_changes`**, the one place that runs after
  every event: when the selected card is another than the one noted, the noted one becomes the
  previous one, but only while it is still a card. A card that left (closed, tidied, forgotten
  by `x` on an ended card) is not remembered as previous, so closing a card and then pressing the
  prefix twice goes back to the card before it, as Alt+Tab does after a window closes.
- The builder's: **a card is matched by its running id, else by its conversation id**, and the
  note is refreshed at every check, so a stopped card that resumes (its id set) or a session
  whose conversation id changes (`/clear`) is still the same card.
- The builder's: **two messages.** `No previous session` when there is none yet, it was
  removed, or it is the card already selected; `The previous session has ended` when it is an
  exited or a stopped card, which harbour would have to resume rather than show.
- The builder's: the help keeps its 14 rows: the last becomes `C-/ C-/ p`, "the previous
  session / send C-/ to the program", 57 columns wide with the default prefix, as wide as the
  narrowest help allows. With `send_prefix` unbound it reads `C-/ C-/`, "the previous session".

## States

What the user sees after the prefix twice. "Previous" is the card selected before the one
selected now.

| State | What the user sees |
| --- | --- |
| No previous session yet (the first card ever selected, or no card) | Nothing moves; `No previous session` on the message line. |
| Only one session | The same: there was never another card to come from. |
| The previous session is running | Its card is selected and its pane focused, as the prefix then its digit does; the prefix twice again comes back. |
| The previous session exited (an exited card) | Nothing moves; `The previous session has ended`. |
| The previous card is stopped (a remembered conversation) | The same: `The previous session has ended`; Enter or its digit resumes it. |
| The previous card was closed (`X`), ended and removed (`x` twice), or tidied away | It is not remembered: the card before it is the previous one, else `No previous session`. |
| The current card was closed and the selection fell on the previous card | `No previous session`: the previous card is the one shown. |
| A card was moved (`J`/`K`), regrouped or renumbered | The same card is found wherever it now is; its number is never used. |
| The previous card resumed or its conversation changed | Still the same card. |
| Focus in the sidebar | The previous card is selected and its pane focused. |
| Focus in a pane on Claude Code | The same; the pane shows the previous card's Claude Code, or its editor if that showed last. |
| Focus in a pane on an editor | The same; nothing reaches the editor. |
| Side by side | Stays side by side on the previous card, as a jump does there. |
| Zoomed | Stays zoomed, showing the previous card, as a jump does. |
| The help or the changelog view open | The first prefix closes the view and waits; the second switches. |
| The usage popup, the picker, a question or the rename popup open | They take the prefix as any key, as before; nothing switches. |
| Ctrl held for the second press (C-/ C-/, C-_ C-_) | The same as without Ctrl: it switches. |
| The prefix then `p` (or C-p) | The program in the focused pane gets the prefix key, as the prefix twice did before. |
| `send_prefix` set to `none` in `[keys]` | Nothing sends the prefix key; the prefix then `p` does nothing and says so, as any unbound key. |
| The narrowest help and a CJK name | The help's last row fits 57 columns with every prefix; a message line names no card, so a CJK name changes nothing drawn. |

## Tasks

1. `send_prefix`: the prefix then `p` sends the prefix key to the focused program
   (`tests/keymap.rs`, `tests/app/prefix.rs`), `[keys] send_prefix` read like the others.
2. The prefix twice switches to the previous card and back (`tests/app/previous.rs`), with every
   state of the table above that the app owns.
3. The help's last row, the README key table that mirrors it, `harbour --help`, the review
   skill's key list, the known limits that named the prefix twice.
4. The end-to-end test: C-/ C-/ through tmux switches between two sessions and back.
5. Screenshots: `docs/help.png` and `docs/keys.png` rerendered.

## Outcome

- Shipped in #248 as a `feat`: the prefix twice selects the card selected before and focuses it
  as a jump does; the prefix then `p` (`send_prefix`, rebindable) sends the prefix key on.
- The previous card is noted in `log_changes`, matched by its running id, else its conversation
  id; a card that left is never the previous one. `No previous session` and
  `The previous session has ended` go on the message line, cleared by a new selection.
- The help keeps 14 rows: `C-/ C-/ p`, "the previous session / send C-/ to the program", 57
  columns, the widest the narrowest help fits. `docs/keys.png` and `docs/help.png` rerendered.
- Every test of the table's app states is in `tests/app/previous.rs`; two end-to-end tests send
  C-/ C-/ through tmux with extended keys on and off.

## Hurdles

- `test-seam` `picker::a_slow_list_holds_up_neither_harbour_nor_the_picker` failed twice under
  the gates' load (the picker's session started before the wait saw `acme picker read x`) and
  passed alone eight times in a row → backlog
