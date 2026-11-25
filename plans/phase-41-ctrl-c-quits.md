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
- Ctrl+C to the quit question is a yes; to `End? y/n` and `End and remove? y/n` it stays a no.
- Ctrl-Shift-c gets no handling of its own.
- The log names it `C-c`, as it names `C-h` and `C-l`.
- `harbour --help` names the key; the help view keeps its 14 rows (no room), so no screenshot.

## States

| Before | Key | What the user sees |
| --- | --- | --- |
| Sidebar focused, a session running | Ctrl+C | `Quit? y/n` |
| Sidebar focused, nothing running | Ctrl+C | harbour quits |
| `Quit? y/n` | Ctrl+C | harbour quits |
| `End? y/n` or `End and remove? y/n` | Ctrl+C | the question closes, the session runs on |
| Pane focused | Ctrl+C | the program gets it |

## Tasks

1. Ctrl+C in the sidebar quits as `C-/ q` does (`tests/app/ending.rs`).
2. Ctrl+C answers the quit question yes, the end questions no (`tests/app/ending.rs`).
3. A focused pane still gets Ctrl+C (`tests/app/ending.rs`).
4. The log names it (`tests/app/logging.rs`), `--help` names it (`tests/cli.rs`), the fragment.

## Outcome

## Hurdles
