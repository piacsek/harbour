# Phase 22: jump to a card by its number

## Context

Issue #41. Ctrl-1 to Ctrl-8 jump to the card at that position, counted from the top, and Ctrl-9
to the last card, from the sidebar and from inside a pane. A jump selects the card and focuses
its pane; a stopped card resumes as Enter resumes it, with the same refusals. No digit shows on
the cards. The keys reach harbour only as extended keys (tmux `extended-keys on` with harbour's
request, or `always`); without them a terminal sends what it sent before and the session gets
it. Select, focus and resume are one function, `App::jump`, so the pick of #39 can call it.

## Verified facts (2026-11-21, `main` at 825b3a5)

- Bytes tmux 3.7c writes for `send-keys C-1` to `C-9`, captured in scratch servers
  (`-f /dev/null`), the app inside raw and printing every byte:

  | `extended-keys` | format | app asked `\e[>4;2m` | Ctrl-1 to Ctrl-9 |
  | --- | --- | --- | --- |
  | `on` | `xterm` | yes | `\e[27;5;49~` to `\e[27;5;57~` |
  | `on` | `csi-u` | yes | `\e[49;5u` to `\e[57;5u` |
  | `always` | either | yes | the same as `on` |
  | `always` | either | no | Ctrl-1 and Ctrl-9 extended; Ctrl-2 to Ctrl-8 the legacy bytes |
  | `on` | either | no | `1`, `\0`, `\033`, `\034`, `\035`, `\036`, `\037`, `\177`, `9` |
  | `off` | either | either | `1`, `\0`, `\033`, `\034`, `\035`, `\036`, `\037`, `\177`, `9` |

  harbour sends `\e[>4;2m` when it enters the terminal (`src/tui.rs`), so under tmux with
  `extended-keys on` it gets the extended forms.
- `src/input.rs` parses both extended forms to `KeyCode::Char('1'..='9')` with exactly
  `KeyModifiers::CONTROL`. The legacy bytes parse to something else: `1` and `9` to plain
  `Char`, `\0` to C-Space, `\033` to Esc, `\034` to C-\\, `\035` to C-], `\036` to C-^, `\037`
  to C-_, `\177` to Backspace. So the jump never fires on legacy bytes, and the session gets
  them as today.
- Neovim 0.12.6 (`--clean`, scratch `HOME`): with extended keys, Ctrl-6 is `CTRL-^` and
  switches to the alternate file (`:h CTRL-6`); there is no default mapping on `<C-1>` to
  `<C-9>` (`maparg` empty in normal and insert mode). A user's own mapping on them is lost to
  the jump while harbour gets extended keys; Ctrl-6 needs `:e #` or the legacy byte instead.
- Claude Code's keybindings documentation lists no default binding on Ctrl with a digit; a user
  can bind `ctrl+1` in `~/.claude/keybindings.json`, which harbour then takes. Not checked by
  running `claude`.
- macOS can take Ctrl-1 to Ctrl-9 first for "Switch to Desktop N" (Keyboard Shortcuts, Mission
  Control), so the key never reaches the terminal.
- `App::key_handled` (`src/app/key.rs`) sends every key to the picker while it is open, lets the
  help view or a question swallow the next key (`dismiss`), then handles C-h and C-l from a
  focused pane, then forwards the key to the focused session, then runs the sidebar keys. The
  jump sits after `dismiss` and before C-h, so a focused pane never receives it.
- `select` clears a `directory gone` or `open elsewhere` refusal; `resume` reports whether it
  handled a stopped card (a refusal or a resume); Enter runs `focus_or_resume`.
- The cards are `App::sessions` in order, top to bottom; `j`/`k` move along the same index.

## What the user sees

| State | Ctrl-N (extended keys) |
| --- | --- |
| A card at position N | that card is selected and its pane focused |
| No card at position N (N from 1 to 8) | nothing happens |
| Ctrl-9 with at least one card | the last card is selected and its pane focused |
| Ctrl-9 with no card at all | nothing happens |
| The card is already selected | its pane is focused |
| The pane already focused on another session | the card at N is selected and its pane focused; the session that had the pane receives nothing |
| An editor focused, one that asked for extended keys | the jump happens; the editor receives nothing |
| A stopped card that resumes | the card resumes as Enter resumes it, its pane focused |
| A stopped card that cannot resume | the card is selected and says why (`directory gone`, `open elsewhere`), as Enter would; the sidebar keeps focus |
| An exited card | selected and its pane focused, as Enter does |
| The selected card was removed between two keys (`x`, or its session exited and was settled) | the position counts the cards shown at that key, so N names whatever card is at N now |
| A question shown | the key answers it as any key but `y` does (no), and no jump |
| The help view shown | the key hides it, as any key does, and no jump |
| The picker shown | the picker gets the key, as today, and no jump |
| A session started but not yet in the sidebar | only through the picker, which is still shown: the picker gets the key |
| Narrow layout (under 60 columns), sidebar shown | the card is selected and its pane shown alone, as Enter shows it |
| Narrow layout, pane shown | the card at N is selected and its pane shown alone |
| Extended keys unavailable (`extended-keys off`, or a terminal without them) | nothing changes from today: the session gets the legacy bytes (`1`, `\0`, Esc, ..., `9`) and the sidebar treats them as it did |
| macOS "Switch to Desktop N" on | macOS takes the key, harbour never sees it (`harbour --help` says so) |
| The log on | the key is logged as `key name="C-N"` |

Zoom is not built yet, so it has no row.

## Decisions

- Ctrl-N is harbour's whenever it arrives as Ctrl with a digit, even from an editor that asked
  for extended keys: the issue wants the keys to work from inside any pane, and Neovim's only
  default on them (Ctrl-6 as `CTRL-^`) has `:e #`.
- Two end-to-end tests in `tests/e2e/keys.rs` sent Ctrl-3 to a session: one under
  `extended-keys on` without an ask now jumps instead, so it runs under `extended-keys off`
  (where Ctrl-3 still reaches the session as Esc, the legacy-bytes check); the one sending
  Ctrl-3 to an editor that asked for level 2 now sends Ctrl-0, which harbour does not take, so
  the encoding it pins keeps a test.
- A jump does not count as acting on the session for `harbour edit`'s "last used", as Enter
  does not.
- The help view and a question keep their rule: the next key closes them and does nothing else.
- The Neovim end-to-end test that typed Ctrl-3 in insert mode under `extended-keys on` now
  types Ctrl-0: Ctrl-3 jumps there, and Ctrl-0 still pins that an extended key never leaves
  insert mode.
- The help row reads `focus or resume that card; C-9 the last` (39 columns): at 59 and 60
  columns the help already used every row, so a row that wraps there pushes `harbour --help has
  the rest.` off. For the same reason C-h now reads `from an editor` instead of `from the
  editor`, one column shorter, so it stops wrapping at 59 columns and frees the row the new key
  takes. The narrow layout on 20 rows has no spare row left; the wide one keeps four.

## Tasks

1. This plan, the `PLAN.md` and `plans/metrics.md` rows, phase 20's times from GitHub.
2. The jump (`src/app/jump.rs`): from the sidebar, from a pane, from an editor that asked for
   extended keys, a stopped card, Ctrl-9, out of range, help and question and picker shown.
3. The legacy bytes still reach the session: in process and end to end under
   `extended-keys off`; the two conflicting end-to-end tests changed.
4. The log names C-1 to C-9.
5. The help row, the README row, the spare-rows test; the macOS note in `harbour --help`.
6. Screenshots, fragment, outcome.

## Outcome

- Ctrl-1 to Ctrl-8 select the card at that position and focus its pane; Ctrl-9 the last card.
  A stopped card resumes as Enter does, with its refusals. Out of range does nothing; the
  help, a question and the picker take the key first.
- A session never gets them when they arrive extended (`\e[27;5;49~` to `\e[27;5;57~`, or
  `\e[49;5u` to `\e[57;5u`); with extended keys off tmux sends the legacy bytes
  (`1`, NUL, ESC, `\034` to `\037`, DEL, `9`), which still reach the session.
- `App::jump(index)` is the one "select, focus, resume" a later pick can call.
- The log names them `C-1` to `C-9`; `harbour --help` says tmux needs `extended-keys on` and
  that macOS's Switch to Desktop N shortcuts can take them first.

## Hurdles

- `api-shape` Three end-to-end tests (two in `tests/e2e/keys.rs`, one in `tests/e2e/neovim.rs`) used Ctrl-3 as the sample extended key and broke once it jumped → code
- `design` The help on 59 and 60 columns had no spare row, so the new key needed a row freed by shortening the C-h text → code
- `test-seam` A key sent out of range from a pane looked passed because Ctrl-5's byte (0x1d) draws nothing; the test now checks the pty echo `^]` → code
