# Phase 42: C-; as a prefix setting

Issue #186. `prefix = "C-;"` is refused today with `expected C-<letter>, C-Space, C-/, C-\,
C-], C-^ or C-_`. Accept it, in the file and in `HARBOUR_PREFIX`, so Ctrl+semicolon opens the
prefix where the terminal reports it as an extended key. The default stays `C-/`.

## Verified facts

- **Harbour asks for extended keys** (`EXTENDED_KEYS_ON` in `src/tui.rs`, `modifyOtherKeys` 2),
  and `src/input.rs` already reads `ESC [ 27 ; 5 ; 59 ~` and `ESC [ 59 ; 5 u` as `;` with Ctrl:
  both go through `extended`, which keeps any other codepoint as its character.
- **`Keymap::is_prefix`** compares the typed character with the prefix (`same_ctrl`), so a
  keymap built with `;` already matched Ctrl+`;`; only `parse_prefix` refused it.
- **`src/keys.rs` already encodes Ctrl+`;`** as xterm does: a plain `;` in the legacy encoding
  (no control byte), `ESC [ 27 ; 5 ; 59 ~` for `modifyOtherKeys` 1 and 2, `ESC [ 59 ; 5 u` for
  kitty (`tests/keys.rs` pins all four), so the prefix pressed twice needs nothing new.
- **`C-;` is as wide as `C-/`**, so the help rows, the bottom border and the unknown-key message
  fit wherever the default does.

## Decided

- The orchestrator's: the default stays `C-/`; `C-;` is only a value the setting accepts.
- The refusal lists it last: `expected C-<letter>, C-Space, C-/, C-\, C-], C-^, C-_ or C-;`.
- `harbour --help` gets a note under the tmux `extkeys` line: a `C-;` prefix needs a terminal
  that reports extended keys, else Ctrl+`;` arrives as a plain `;`; under tmux,
  `extended-keys on`, and `extkeys` for a terminal tmux does not know.
- No screenshot: nothing changes with the default settings.

## Tasks

1. Prove `src/input.rs` reads both extended forms of Ctrl+`;` (`tests/input.rs`).
2. `parse_prefix` accepts `C-;` from the file and the variable, named `C-;`, and the refusal
   lists it (`tests/config/keys.rs`).
3. The prefix set to `;` pressed twice sends the program a `;` (`tests/app/bindings.rs`).
4. `harbour --help` says what it needs (`tests/cli.rs`), one end-to-end test sending the xterm
   form through tmux (`tests/e2e/config.rs`), the fragment.

## Outcome

`parse_prefix` takes `C-;`, from the file and from `HARBOUR_PREFIX`, and the refusal names it.
Nothing else needed code: `src/input.rs` already read both extended forms of Ctrl+`;`,
`is_prefix` already matched it, and `src/keys.rs` already sent the prefix pressed twice as `;`
to a program that asked for nothing and in the extended form to one that did. Tasks 1 and 3
and the end-to-end test were green on their first run, so each was shown red by breaking the
code it covers, then restored. `harbour --help` says what `C-;` needs. Review round 1 found no must; its five cheap findings
(the variable shown unquoted in the fragment, the end-to-end test pinning a literal, the help
note too terse, the `src/help.rs` entry missing it, an app test repeating another) were fixed
in one ship. Phase
41's ready, merged and released were read from GitHub into `plans/metrics.md`.

## Hurdles

- `test-seam` the prefix-twice test first read one byte, which Ctrl+`;` alone also sends, so it passed with the prefix broken; it now reads the next key too → code
- `review-loop` round 1 found five cheap gaps, among them `HARBOUR_PREFIX=C-;` shown unquoted, which a shell cuts at `;`, and an end-to-end test pinning a literal; all fixed in one ship → code
