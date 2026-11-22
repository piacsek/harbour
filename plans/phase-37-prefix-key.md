# Phase 37: one prefix key for harbour's actions, from anywhere

Today a harbour action works from some places and not others. The sidebar's letters (`e`, `u`,
`c`, `?`, `q`) work only while the sidebar has focus. The Alt keys (`M-z`, `M-\`, `M-{`/`M-}`,
`M-s`, `M--`/`M-=`) work from the sidebar and Claude Code, never from an editor, and only in
their extended form. `C-1` to `C-9` arrive only as extended keys. The owner wants a tmux-like
prefix instead: one key, then a plain key, that acts the same wherever the focus is. Both
the prefix and the key of each action are settings.

This phase adds the prefix and removes the Alt keys, `C-1` to `C-9` and the sidebar letters
other than `n` and `x`, without keeping the old keys: harbour has one user, the owner, who asked
for the clean break.

## Verified facts

- **The key dispatch is one function**: `key_handled` in `src/app/key.rs` hands a key to the
  usage popup, the picker, the changelog view and a question or the help first, then tries
  `jump_by_key`, `zoom_by_key`, `side_by_key`, `swap_by_key`, `resize_by_key`, `find_by_key` and
  `navigate_by_key`, then C-h and C-l, then the focused program, and only then the sidebar's
  letters. Each action already has a handler the prefix can call.
- **The Alt keys skip editors on purpose**: `zoom_by_key` returns false while an editor is shown
  with focus, and `is_zoom` refuses a key carrying `ESCAPED`, since a legacy `ESC z` is also Esc
  then `z` typed fast (`harbour --help` says so for `M-z` and `M-s`).
- **`C-/` reaches harbour as one byte or as an extended key, never as something else** (tmux
  3.4 in a scratch server, `send-keys C-/` and `send-keys C-_` into a program reading raw bytes):
  without the program asking for extended keys both arrive as `0x1f`; once it asks for
  `modifyOtherKeys` 2, as harbour does on entering, `C-/` arrives as `ESC [ 47 ; 5 u` and `C-_`
  still as `0x1f`, with `extended-keys` on and off alike. `src/input.rs` reads `0x1f` as Ctrl
  with `_` and the csi-u and xterm forms as Ctrl with `/`. What a real terminal sends under tmux
  goes through tmux's own input parser, not `send-keys`, and is checked in task 1.
- **Sending `C-/` on**: `src/keys.rs` encodes Ctrl with `/` as `0x1f` in the legacy form,
  `ESC [ 27 ; 5 ; 47 ~` at `modifyOtherKeys` 2 and `ESC [ 47 ; 5 u` in kitty's (`tests/keys.rs`).
  Issue #75 lists Ctrl-/ arriving as one byte and going to a kitty program as Ctrl-_, which the
  prefix pressed twice inherits; it stays #75's.
- **Settings follow one pattern**: `src/config.rs` reads each setting from its `HARBOUR_`
  variable, then the file, then the default, and refuses a bad value naming the file, the line
  and the key; `harbour config` reports each with its source.
- **The help view is full on a narrow terminal** (issue #98): at 59 to 60 columns and 20 rows,
  the 14 rows of `KEYS` (`src/help.rs`) fill it. The README key table mirrors `KEYS`
  (`tests/readme.rs`).
- **Tests that press the keys this phase removes**: `tests/app/` `zoom.rs`, `zoom_border.rs`,
  `side.rs`, `side_border.rs`, `side_jump.rs`, `side_mouse.rs`, `side_states.rs`,
  `side_swap.rs`, `finding.rs`, `finding_states.rs`, `width_keys.rs`, `jumping.rs`,
  `changelog.rs`, `usage.rs`, `cursor.rs`, `ending.rs`, `logging.rs`, `neovim.rs`;
  `tests/e2e/` `keys.rs`, `finding.rs`, `editor.rs`. `scripts/screenshots.sh` presses `M-z`,
  `M-\`, `M-}`, `M-s` (with tmux's `extended-keys` on for those shots), `?`, `q`, `u` and `c`.
- **Claude Code 2.1.330 reads `0x1f` as undo** (checked in task 1 from its bundled default
  keybindings, since its prompt cannot run under a scratch configuration without signing in):
  `ctrl+_`, `ctrl+-`, `ctrl+shift+-` and `ctrl+shift+_` all map to `chat:undo`, with the tip "To
  undo typing, press ctrl+_"; nothing binds `ctrl+/`. Claude Code always gets the legacy byte, so
  `C-/ C-/` reaches its prompt as undo.
- **A real terminal's bytes under tmux 3.8** (checked in task 1 by writing raw bytes into an
  attached client's PTY, `TERM=xterm-256color`, `terminal-features xterm*:extkeys`): with the
  program not asking for extended keys, `0x1f`, `ESC [ 47 ; 5 u` and `ESC [ 27 ; 5 ; 47 ~` all
  arrive as `0x1f`, with `extended-keys` off, on or always; with `extended-keys` off it is `0x1f`
  even when the program asks; with it on or always and the program at `modifyOtherKeys` 2, `0x1f`
  arrives as `ESC [ 27 ; 5 ; 95 ~` (Ctrl with `_`) and both extended forms of `C-/` as
  `ESC [ 27 ; 5 ; 47 ~`. So the prefix matches Ctrl with `/` or `_` in every form. Which
  terminals send nothing for `C-/` was not checked on a real one; the `--help` note names `C-_`
  and the `prefix` setting for them.

## Decided

- **The prefix is `C-/`**, matched as Ctrl with `/` or `_` in every form `src/input.rs` reads
  (`0x1f`, csi-u, xterm), Shift ignored, so it needs no extended keys. Not `C-Space` (the owner's
  tmux prefix), not `C-Backspace` (`0x08`, the same byte as C-h without extended keys, and
  delete-word in Claude Code's prompt) and not `C-Enter` (`CR`, plain Enter without extended
  keys, and Claude Code's newline in harbour).
- **`prefix` is a setting**: `HARBOUR_PREFIX`, then `prefix` in `config.toml`, then `C-/`. It
  takes `C-<letter>`, `C-Space`, `C-/`, `C-\`, `C-]`, `C-^` and `C-_`, the keys with a legacy
  byte of their own. It refuses `C-h`, `C-j`, `C-k`, `C-l`, `C-m` and `C-i` (harbour's own keys,
  Enter and Tab) and anything else, naming what it takes. Keys that need extended keys
  (`C-Backspace`, `C-Enter`) are left for a later phase.
- **One key after the prefix**, then the prefix is spent. Each action has a name, which the
  `[keys]` table of `config.toml` binds to another key; the defaults:

  | Action | Default key | Does |
  | --- | --- | --- |
  | (fixed) | `1`..`9` | focus or resume that card, `9` the last (`jump` as today) |
  | `zoom` | `z` | zoom the session, or back |
  | `side` | `\` | side by side, or back (120+ columns) |
  | `swap_left` / `swap_right` | `{` / `}` | Claude Code in the left / right half, only side by side |
  | `find` | `s` | the session finder |
  | `new` | `n` | pick a directory and start a session |
  | `end` | `x` | end the session and its editors, or forget an ended one |
  | `editor` | `e` | the next editor |
  | `sidebar` | `h` | focus the sidebar, from anywhere, without asking Neovim |
  | `pane` | `l` | focus the pane, as Enter from the sidebar |
  | `narrower` / `wider` | `-` / `=` | the sidebar 2 columns narrower / wider; the same key again within 500 ms repeats without the prefix |
  | `usage` | `u` | the usage popup |
  | `changelog` | `c` | the changelog view |
  | `help` | `?` | the help view |
  | `quit` | `q` | quit, after a yes while anything runs |
  | (fixed) | the prefix | the prefix itself to the focused program, in the form it asked for |

  Every action works from the sidebar, Claude Code, an editor, zoomed, and both halves of side by
  side, editors included. A letter's Shift form does the same (`Z` as `z`), so Caps Lock changes
  nothing.
- **The bindings are settings**, from the file only, as `nerd_font` is (no `HARBOUR_` variable
  per action):

  ```toml
  prefix = "C-]"

  [keys]
  zoom = "f"
  usage = "none"
  ```

  - A value is one printable character, matched as typed with no modifier but Shift; a letter
    stands for both its cases, so `z` and `Z` are one key. `"none"` unbinds the action.
  - An action left out keeps its default.
  - Refused, naming the file, the line and the key, as every bad setting: an unknown action
    name, a value that is not one character or `"none"`, a digit (`1`..`9` are the jumps), a
    space, and two actions on one key, whether both were written or one kept its default (the
    refusal names both, so moving `zoom` to `s` means binding `find` elsewhere too).
  - `new` and `end` also bind the sidebar's plain keys: `new = "a"` makes `a` start a session
    from the sidebar and after the prefix, and `n` then does nothing in the sidebar. A sidebar key
    that would shadow `j` or `k` is refused.
  - `harbour config` reports the prefix and each binding with its source (`default` or the
    file), one line each.
  - The help view, the hint and the unknown-key message draw the bindings in force; a pair row
    with one side unbound shows the other alone, a row with neither is left out.
- **An unknown key after the prefix is swallowed** and the message line says
  `C-/ <key> does nothing; C-/ ? lists the keys`, cleared as other messages are.
- **Esc after the prefix cancels it** silently. A paste or a mouse press cancels it and is then
  handled as today.
- **No timeout**, as tmux's default.
- **The sidebar keeps**: `j`/`k`, Enter, `C-h`/`C-l`, `C-j`/`C-k` (tmux panes), `n` and `x`.
  Any other key there does nothing. The sidebar stays a focus target.
- **Removed**: `M-z`, `M-\`, `M-{`, `M-}`, `M-s`, `M--`, `M-=` and their Shift forms; `C-1` to
  `C-9`; `e`, `u`, `c`, `?` and `q` in the sidebar. They now reach the focused program as any
  other key does; in the sidebar they do nothing.
- **What stays as is**: C-h and C-l with their Neovim rules, Ctrl-Shift-h and Ctrl-Shift-l,
  C-j and C-k under tmux, every key inside the usage popup, the changelog view, the picker, the
  help and a question. While any of those is open the prefix is not read: fzf and the popup get
  it as they get every key, and the help or a question takes it as any other key.
- **The extended-keys request stays**: harbour still asks for `modifyOtherKeys` 2 so an editor or
  picker that asked for extended keys gets them, but no harbour action depends on them any more.
  `harbour --help` loses the notes on the Alt keys' rule and on Esc then `z`/`s` typed fast; it
  keeps the newline and `extkeys` notes, which are about the programs.
- **Waiting for the second key shows**: `C-/` in the accent, bold, right-aligned on the pane's
  bottom border; zoomed, on the top row just before the counts. It goes once the key comes.
- **A hint shows the way in**: `C-/ ? help`, dimmed, right-aligned on the sidebar's bottom
  border, the configured prefix's name in place of `C-/`, left out when the border has no room
  for it whole; with no session, on the empty pane's message too.
- **The help view keeps 14 rows**, so the narrow terminal of #98 still fits it: the keys column
  names the prefix in each row, and pairs share a row:

  | Keys | Text |
  | --- | --- |
  | `j/k` | select the next / previous session |
  | `Enter/C-l` | focus or resume; C-l in Claude Code: editor |
  | `C-h` | focus the sidebar; from an editor, Claude Code |
  | `n/x` | start a session / end or forget it, also after C-/ |
  | `C-/ 1..9` | focus or resume that card; 9 the last |
  | `C-/ z` | zoom the session to the whole screen, or back |
  | `C-/ \ {}` | side by side (120+ columns); swap the halves |
  | `C-/ s` | jump to a session by part of its name |
  | `C-/ e` | show the next editor, one per directory |
  | `C-/ h/l` | focus the sidebar / the pane |
  | `C-/ -/=` | narrow / widen the sidebar by 2 columns |
  | `C-/ u/c` | Claude Code's usage / the changelog |
  | `C-/ ?/q` | this help / quit, after a yes while anything runs |
  | `C-/ C-/` | send C-/ to the program |

  `KEYS` stays a constant of the defaults, naming `C-/`; the help view draws its rows from the
  prefix and the bindings in force. The README mirrors the constant.
- **The log**: `prefix` when it is pressed, `prefix key=<name>` for an action (the action's own
  line follows as today), `prefix key=other` for an unknown key, `prefix cancelled`; the prefix
  sent on as `input to=<where> bytes=<n>`, never its bytes.
- **The fragment is `kind: feat`**: the bullets say the actions moved behind `C-/`, which keys
  are gone, and the `prefix` setting and the `[keys]` table.

## States

| State | What the user sees |
| --- | --- |
| Sidebar, `C-/ z` on a running session | zoomed on its Claude Code, as `M-z` did |
| Neovim focused, `C-/ z` | zoomed on that editor (today: nothing) |
| Neovim focused, `C-/ \` | side by side, the editor half focused |
| Side by side, editor half, `C-/ }` | swapped, as `M-}` from Claude Code did |
| Not side by side, `C-/ }` | nothing, no message (as `M-}` today) |
| Under 120 columns, `C-/ \` | nothing, as `M-\` today |
| Ended session, `C-/ z` | nothing, as today |
| No sessions, `C-/ 3` | nothing |
| `C-/ 9` with 4 cards | the 4th card |
| Zoomed, `C-/ 2` | card 2, zoom dropped as a jump drops it today |
| Side by side, `C-/ 2` | card 2 side by side, as Ctrl-2 today |
| `C-/ s` with an editor focused | the session finder (today: nothing) |
| `C-/ s` with a picker that is not fzf | the message `Could not start the session finder: it needs fzf, not <program>`, as today |
| `C-/ n` from Claude Code | the directory picker |
| `C-/ x` from Neovim with unsaved changes | the question, saying it may lose changes |
| `C-/ h` with Neovim in insert mode in the leftmost window | the sidebar focused (C-h would stay in Neovim) |
| `C-/ h` while zoomed or side by side | the sidebar focused, which drops the mode as today |
| `C-/ l` from the sidebar on a stopped card | resumed, as Enter |
| `C-/ -`, then `-` within 500 ms | 4 columns narrower |
| `C-/ -`, then `-` after 500 ms | 2 columns narrower, then `-` reaches the program |
| `C-/ =` zoomed or under 60 columns | nothing, as today |
| `C-/ u` from Neovim | the usage popup; Esc closes it, the focus back on Neovim |
| `C-/ c` from Claude Code | the changelog view; any other key closes it, the focus as it was |
| `C-/ ?` from a pane | the help; any key closes it |
| `C-/ q` from Claude Code | the quit question |
| `C-/ C-/` to Claude Code | Claude Code gets `0x1f` |
| `C-/ C-/` to an editor at `modifyOtherKeys` 2 | it gets `ESC [ 27 ; 5 ; 47 ~` |
| `C-/ w` | swallowed; the message `C-/ w does nothing; C-/ ? lists the keys` |
| `C-/ Esc` | nothing, the prefix gone |
| `C-/`, then a paste | the prefix gone, the paste to the focused program |
| `C-/`, then a click on another card | the prefix gone, the click as today |
| `C-/`, then the session exits | the prefix still waits; the next key acts on what is selected then |
| `C-/` while the picker is open | fzf gets it |
| `C-/` while the usage popup is open | its report gets it |
| `C-/` while a question shows | the answer no, as any key but `y` today |
| `C-/` while the help or the changelog view shows | it closes, as any key does today |
| `C-_` (Ctrl-Shift-minus) | the prefix, the same byte |
| `C-/` from a terminal that sends nothing for it | nothing; `harbour --help` names `C-_` and `prefix` |
| `M-z`, `M-s`, `M-\` anywhere | reach the focused program; nothing in the sidebar |
| `Ctrl-3` with extended keys | reaches the focused program; nothing in the sidebar |
| `e`, `u`, `c`, `?`, `q` in the sidebar | nothing |
| `n`, `x` in the sidebar | as today |
| `C-/` waiting, not zoomed | `C-/` in the accent on the pane's bottom border |
| `C-/` waiting, zoomed | `C-/` on the top row before the counts |
| `C-/` waiting, side by side | `C-/` on the focused half's bottom border |
| `[keys] zoom = "f"`, `C-/ f` | zoomed |
| `[keys] zoom = "f"`, `C-/ z` | swallowed, the unknown-key message |
| `[keys] zoom = "F"`, `C-/ f` | zoomed: one key for both cases |
| `[keys] zoom = "s"` alone | harbour refuses to open: `zoom` and `find` both on `s` |
| `[keys] zoom = "s"`, `find = "/"` | `C-/ s` zooms, `C-/ /` finds |
| `[keys] usage = "none"` | `C-/ u` swallowed; the help row reads `C-/ c` alone |
| `[keys] usage = "none"`, `changelog = "none"` | no usage-and-changelog row in the help |
| `[keys] new = "a"` | `a` and `C-/ a` start a session; `n` in the sidebar does nothing |
| `[keys] new = "j"` | refused: `j` moves the selection in the sidebar |
| `[keys] zoom = "3"` | refused: digits are the jumps |
| `[keys] zoom = "zz"` or `""` | refused: one character or `"none"` |
| `[keys] zoomed = "z"` | refused: no action named `zoomed`, the names listed |
| `[keys]` not a table, or a value not a string | refused, naming the line |
| `prefix = "C-h"` | refused: C-h is harbour's own key |

### What it reads from outside

| Read | Missing | Changed meanwhile | Shared |
| --- | --- | --- | --- |
| `HARBOUR_PREFIX` | the file's `prefix`, else `C-/` | read once at start | per process |
| `prefix` in `config.toml` | `C-/` | read once at start; a bad value refuses to open, as other settings | the user's |
| `[keys]` in `config.toml` | every default | read once at start; a bad binding refuses to open | the user's |
| the terminal's bytes for `C-/` | no prefix; `C-_` still works | n/a | n/a |
| tmux's `extended-keys` | no change: the prefix needs none | n/a | the tmux server's |

### What it draws

| Case | What shows |
| --- | --- |
| Sidebar 15 columns (the narrowest) | `C-/ ? help` (10 columns) on its bottom border, the corners kept |
| Sidebar too narrow for the hint whole | no hint |
| A configured prefix `C-]` | `C-] ? help`, and `C-]` in every help row |
| Narrow layout under 60 columns, the pane shown | no sidebar, no hint; the waiting `C-/` on the pane's border |
| A CJK card title | cards drawn as today; the hint and the waiting mark are ASCII |
| Help view at 60 by 20 | the 14 rows fit, as today |

## Tasks

1. The prefix and its table, the old keys still working: `src/app/prefix.rs` with the waiting
   state, the dispatch at the top of `key_handled` after the overlays, every action of the table,
   the repeat of `-` and `=`, the message for an unknown key, the log lines. Check first what
   Claude Code does with `C-/` and `C-_`, and what a real terminal sends under tmux; write both
   into this plan.
2. The settings: `prefix` (`HARBOUR_PREFIX`, the file) and the `[keys]` table (the file), every
   refusal of the States table, the sidebar's `new` and `end` keys following their bindings,
   `harbour config` reporting each with its source.
3. The old keys removed: the Alt keys, `C-1` to `C-9`, and `e`, `u`, `c`, `?`, `q` in the
   sidebar; their tests moved to prefix sequences, the e2e tests through tmux without
   `extended-keys`.
4. What it draws: the waiting mark, the hint, the help rows, the README key table.
5. `harbour --help` notes, the `AGENTS.md` layout lines (`key.rs`, `zoom.rs`, `side.rs`,
   `find.rs`, `split.rs`, `jump.rs`, `changelog.rs`, `usage.rs`, `help.rs`, `config.rs`, the new
   `prefix.rs`), `scripts/screenshots.sh` on prefix sequences without `extended-keys` (and a
   `prefix` shot of the waiting mark), the fragment `changes/<branch>.md` (`kind: feat`).

## Outcome

## Hurdles
