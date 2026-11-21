# Phase 17: a still sidebar, with no activity line on the selected card

## Context

Issue #31. Line 2 of a working session's card showed its last tool call, which changes every few
seconds: on the selected card beside a pane that already shows it, and on the others while the
user reads somewhere else. The selected card now never shows line 2, and every other working or
idle card shows the session's last prompt, dimmed. The last tool call is no longer read. Phase 15
is in flight in another worktree and moves key handling and the split of the terminal, so this
phase touches neither.

## Verified facts (2026-11-21, `main` at 41b0896)

- `Card::new` in `src/card.rs` picks line 2 in its `doing` match: `Exited` and `Stopped` from
  `ended`, `Blocked` from the registry's `waitingFor`, `Working` from `facts.activity`, `Idle`
  from `facts.prompt`, dimmed; an empty or missing fact gives no line.
- `Sidebar::draw` already gets the selected index and draws every card as `HEIGHT` (3) lines with
  a blank line between cards, `lines` always returning three: hiding line 2 there needs no new
  argument from the app, so `src/app/` is not touched, and a card keeps its height by
  construction.
- `facts.activity` is read only by `Card::new`. In `src/transcript.rs` it is the only user of the
  `assistant` arm of `learn`, of the `user` arm that clears it, and of `prompted`,
  `last_tool_call` and `described`; `src/app/logging.rs` and `src/log.rs` never read `Facts`.
  `tests/dead_code.rs` has none of these on `SEAMS`.
- `Facts` is a public struct with public fields, so dropping `activity` breaks every test that
  names it: `tests/card.rs`, `tests/transcript.rs`, `tests/app/transcripts.rs`.
- The in-process card tests of `tests/app/transcripts.rs` start one session, which is selected,
  so they now need a second session for the card under test to show line 2.
- The demo starts with its working session selected (`App::demo` sets `selected = 0`), so it
  cannot show a working unselected card without changing `src/app/mod.rs` or adding a session,
  and a seventh card does not fit the 26 rows of the shots. The `details` shot of
  `scripts/screenshots.sh` presses `j` instead: the blocked card is selected and hides its line,
  and the working card, unselected, shows its last prompt dimmed. The working session's demo
  transcript gets a `last-prompt` line.
- `tests/e2e/main.rs` (`the_demo_shows_a_made_up_session_in_each_state`) asserts
  `Run the retry tests`, and `tests/e2e/cards.rs` asserts `Run the login tests`, both tool calls.

## What line 2 shows

| Session state | Selected card | Any other card |
| --- | --- | --- |
| Working | nothing | last prompt, dimmed |
| Idle | nothing | last prompt, dimmed (as before) |
| Blocked | nothing | what it waits for (as before) |
| Exited | nothing | `exited` and its note (as before) |
| Stopped (remembered) | nothing | why it cannot resume, if anything (as before) |
| No prompt read yet | nothing | nothing |
| Narrowest sidebar (20 columns), working | nothing | the prompt cut with `…` to the room left after the indent |
| A CJK prompt (`東京の テストを 直して`), working | nothing | the prompt cut by display width, a wide character never split |

The row a card starts on depends only on its position, never on whether a card above is
selected.

## Tasks

1. This plan and its rows; a selected card draws no line 2, and an unselected working card shows
   its last prompt dimmed (`src/card.rs`).
2. The last tool call removed from `src/transcript.rs` with everything only it used, and its
   tests.
3. The demo's working session gets a last prompt, the `details` shot selects the blocked card,
   the screenshots are shot again; the fragment.

## Outcome

## Hurdles
