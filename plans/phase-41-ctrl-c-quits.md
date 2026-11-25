# Phase 41: Ctrl+C quits harbour from the sidebar

Issue #187. Quitting takes the prefix then `q` today, and Ctrl+C in the sidebar does nothing.
With the sidebar focused, Ctrl+C starts the quit as `C-/ q` does, and Ctrl+C to the quit
question counts as yes, so Ctrl+C twice always quits.

## Verified facts

- **The sidebar's keys** are the last match in `key_handled` (`src/app/key.rs`), reached only
  when no popup, picker, view, question or prefix took the key and `forward` sent nothing to a
  focused pane. `C-/ q` calls `quit()`, which asks `Quit? y/n` while a session runs, else quits.
- **Every question is answered in one place**: `dismiss(is_yes(key))`, before the prefix and the
  sidebar; the help and the changelog view close on any key there too.
- **`ctrl_shift_as_ctrl`** lowers only `h`, `j`, `k` and `l`, so Ctrl-Shift-c stays a key of its
  own and does nothing in the sidebar.
- **The log names a sidebar key** with `named` in `src/app/logging.rs` (`key name="C-h"`); any
  other control key logs `key name=other`.
- **The help fills a 20-row terminal with no row to spare** (phase 40), so a row for Ctrl+C does
  not fit; `harbour --help` carries the notes.

## Decided

- The orchestrator's: Ctrl+C acts only in the sidebar; a focused pane gets it as before. The
  usage popup, the pickers, the help and the changelog view take it as they take any key.
- Ctrl+C to the quit question is a yes only when nothing is unsaved (round 1 of review): to
  `Unsaved changes. Quit? y/n` and `May have unsaved changes. Quit? y/n` it is a no, so a
  double tap never loses an editor's edits. To `End? y/n` and `End and remove? y/n` it stays a
  no. Two taps in the sidebar ending every session otherwise is the owner's ask, kept in
  `docs/known-limits.md`.
- Ctrl-Shift-c gets no handling of its own.
- The log names it `C-c`, as it names `C-h` and `C-l`.
- `harbour --help` names the key; the help view keeps its 14 rows (no room), so no screenshot.

## States

| Before | Key | What the user sees |
| --- | --- | --- |
| Sidebar focused, a session running | Ctrl+C | `Quit? y/n` |
| Sidebar focused, nothing running | Ctrl+C | harbour quits |
| `Quit? y/n` | Ctrl+C | harbour quits |
| `Unsaved changes. Quit? y/n` or `May have unsaved changes. Quit? y/n` | Ctrl+C | the question closes, harbour runs on |
| `End? y/n` or `End and remove? y/n` | Ctrl+C | the question closes, the session runs on |
| Pane focused | Ctrl+C | the program gets it |

## Tasks

1. Ctrl+C in the sidebar quits as `C-/ q` does (`tests/app/ending.rs`).
2. Ctrl+C answers the quit question yes, the end questions no (`tests/app/ending.rs`); no to
   the quit question with unsaved changes (`tests/app/neovim.rs`, `tests/app/editor_ending.rs`).
3. A focused pane still gets Ctrl+C (`tests/app/ending.rs`).
4. The log names it (`tests/app/logging.rs`), `--help` names it (`tests/cli.rs`), one
   end-to-end test through tmux (`tests/e2e/main.rs`), the fragment.

## Outcome

Ctrl+C in the sidebar calls `quit()`, as `C-/ q` does, from the last match of `key_handled`,
in #188. The answer to a question is a yes for Ctrl+C only while that question is
`Quit? y/n` with nothing unsaved; `End? y/n` and `End and remove? y/n` still close on it as a no. A focused pane, the
usage popup, the pickers, the help and the changelog view take it as before, since they all come
first. The log names it `key name="C-c"`, and `harbour --help` says what it does. The help view
and the README are unchanged (no row to spare), so no screenshot. Tasks 2 and 3 shipped
together. Phase 40's ready, merged and released were read from GitHub into `plans/metrics.md`.

Round 1 of review made Ctrl+C a no to the quit question while an editor may have unsaved
changes. Round 2 found no must; its findings (held-key autorepeat, Ctrl+C from a quit question
opened in a pane, the Ctrl-Shift-c claim, a pre-existing modifier gap) were left as issue #199,
and the loop ended at the owner's 2-round limit.

## Hurdles

- `review-loop` round 1 found that Ctrl+C twice quit past `Unsaved changes. Quit? y/n`, losing an editor's edits; Ctrl+C is now a yes only when nothing is unsaved, and two taps ending every session otherwise went to `docs/known-limits.md` → code
- `review-loop` round 2 found smaller gaps around the new key (held-key autorepeat, a quit question opened from a pane, the Ctrl-Shift-c claim, a pre-existing modifier gap); the owner's 2-round limit sent them to issue #199 → backlog
