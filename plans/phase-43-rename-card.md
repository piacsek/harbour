# Phase 43: rename a card

Issue #189. A card's title is whatever Claude Code named the conversation, often long, generic or
the same across one repository's sessions. The prefix then `r` lets the user type a title of
their own, which the card shows in place of Claude Code's.

## Verified facts

- **Every title but the card's goes through `headline_of`** (`src/app/zoom.rs`): the zoomed top
  row, the narrow side-by-side border and the session finder's lines. The card picks its own:
  the transcript's title, else the entry's name (`Card::new`).
- **The question and the failure message draw at the bottom of `below_rows`** (the sidebar's
  inside), or of the pane while the sidebar is hidden (zoomed, side by side, the narrow layout
  showing the pane), through `draw_bottom` in `src/app/draw.rs`.
- **A remembered file is written only for a running session whose transcript was found**
  (`wanted` in `src/app/remember.rs`); a stopped card has a file and no `kept`, an exited one a
  file and its last `kept`. `Memory::place` rewrites an existing file's position alone.
- **`Saved` ignores unknown fields** (no `deny_unknown_fields`), so an older harbour reads a file
  holding the new field, and drops it the next time it writes that file.
- **`log_key` writes `key name=j` (and `k`, `y`, the session keys) for a key nothing forwarded**,
  so a key typed into the input line must be marked handled, or one letter of the title reaches
  the log.
- **`text::clean` trims and folds spaces**, so it runs on the saved title only, never on each key,
  or a space between two words could never be typed.

## Decided

- The orchestrator's decisions are in issue #189 and the build brief; these are the builder's.
- The input line reads `Title: <text>`, the label bold, then the text's end that fits with one
  column left for the cursor, `…` at its start when cut. Under 17 columns the label goes and the
  text has the whole line.
- With the sidebar hidden, the input line takes the bottom row of the pane, where the question
  and the failure message already go: the smallest change, and it stays visible in every layout.
- Enter on the prefilled title unchanged changes nothing, so a card keeps following Claude Code's
  title until the user types one.
- The input line takes Backspace and printable keys; a paste is cleaned and added; every other
  key, the prefix included, and a mouse press do nothing while it is open.
- Saving a title writes it into the remembered file at once when the card has one (stopped,
  exited or running), as `place` writes a position; a running session whose file does not exist
  yet writes it with the rest once its transcript is found.
- A stopped card merged into the session that resumed its conversation gives that session its
  title when the session has none.
- The help row of `find` shares with `rename`: `C-/ s/r  jump to a session by name / rename the
  card`, 54 columns.

## States

| What | State | What the user sees |
| --- | --- | --- |
| card | running, exited or stopped | the prefix then `r` opens the input line, prefilled |
| card | none (no session) | nothing |
| input | Enter, text changed and not empty | the card, the headline, the finder show it |
| input | Enter, empty after cleaning | Claude Code's title (or the name) again |
| input | Enter, unchanged | nothing changes |
| input | Esc | closed, nothing changes |
| input | a paste holding a newline or a control character | cleaned, one line |
| sidebar | narrowest (15 columns, 13 inside) | no label, the text's end and the cursor |
| sidebar | hidden (zoomed, side by side, narrow layout on the pane) | the line at the pane's bottom |
| title | CJK (`東京の会議`, 2 columns each) | cut by display width, never half a character |
| remembered file | none yet (running, no transcript) | written with the title once the transcript shows |
| remembered file | exists (stopped, exited, running) | rewritten with the title, other fields read fresh |
| remembered file | written before this phase (no field) | read, no given title |
| remembered file | from a newer harbour | never overwritten (existing rule); the title lasts until quit |
| remembered file | gone or unwritable | the title lasts until quit |
| remembered file | read by an older harbour | read; that harbour drops the title when it next writes |
| `harbour edit` | while the input line is open | refused as for a question |
| log | any key typed into the input line | nothing; only `prefix key=rename` |

## Tasks

1. The remembered file keeps an optional given title (`tests/memory/`).
2. A card shows a given title over Claude Code's (`tests/card/`).
3. The input line's text for a width (`src/rename.rs`, `tests/rename.rs`).
4. The `rename` action: the input line opens prefilled, takes keys and pastes, Esc cancels, Enter
   saves, empty goes back (`tests/app/renaming.rs`); `harbour config`, the help row, the README.
5. The given title in the zoomed headline, the side-by-side border and the finder.
6. Saved to the remembered file and recalled; carried over a merge.
7. The end-to-end test, `harbour --help`, the guides, the fragment, the screenshot.

## Outcome

The prefix then `r` (the `rename` action, configurable under `[keys]`) opens one line at the
bottom of the sidebar, or of the pane while the sidebar is hidden, prefilled with the card's
title and the cursor after it. Enter saves the cleaned text as the card's given title, which
wins on the card, the zoomed headline, the side-by-side border and the session finder (all but
the card read `title_of`); empty goes back to Claude Code's title, unchanged changes nothing, Esc
cancels. The title is written into the remembered file at once (`Memory::title`) and with every
later write as the optional `givenTitle`, recalled with the stopped card, and handed to the
session a stopped card is merged into. While the line is open it takes every key, paste and mouse
press, `harbour edit` is refused as for a question, and only `prefix key=rename` is logged. The
help row of `find` shares with `rename`; `harbour --help` and the README say it; a `rename` shot
shows the line. Phase 42's ready, merged and released were read from GitHub into
`plans/metrics.md`.

Review round 1 found six cheap gaps, fixed in one ship: Shift with a letter typed the small
letter under extended keys; a config already binding `r` refused to start, so `rename`'s default
now yields as `close`'s does; renaming a card open elsewhere wrote a file another harbour owns,
now refused as Enter is; the cursor of a line with no row; the text past 512 characters; and the
demo's stopped card now has a given title, so `docs/details.png` shows one. A merge where both
sides have a given title keeps the session's, a known limit.

## Hurdles

- `tooling` the first `ship.sh` ran in the background while the next task was being written; its `git add -A` took the half-done card change into the first commit, so that commit alone fails `tests/dead_code.rs`, which the next commit fixed → none
- `tooling` waiting on a ship with `pgrep -f "<its message>"` matched the waiting command itself and never ended; wait on the ship's pid or its log instead → none
- `test-seam` several tests were green on their first run (Esc, an empty title, Enter unchanged, the merge, the end-to-end test); each was shown red by breaking the code it covers, and the merge test had to wait for the stopped card to go before it could fail → code
- `review-loop` round 1 found states the plan's table missed: an upgrade whose config already binds `r`, a card open elsewhere, Shift under extended keys and a merge where both sides have a title → code
- `tooling` a metrics-only ship failed twice on `openpty` errors in `tests/session/` while other worktrees ran their gates; it passed once the load dropped → none
