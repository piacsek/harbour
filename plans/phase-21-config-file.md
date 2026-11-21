# Phase 21: harbour's settings in one config file

## Context

Issue #30. Harbour's settings live in three environment variables (`HARBOUR_EDITOR`,
`HARBOUR_DIR_PICKER`, `HARBOUR_SESSION_COMMAND`), each naming one program with no arguments,
and the sidebar's width and colour are fixed. This phase reads one TOML file,
`$XDG_CONFIG_HOME/harbour/config.toml`, else `~/.config/harbour/config.toml`, once at start, with
five keys: `editor`, `picker`, `session_command` (each a program and its arguments, no shell),
`sidebar_width` and `accent`. A variable still wins over the file, the file over the default.
`harbour config` prints the path and every effective setting with its source. Harbour never
writes the file; without one nothing changes.

Out of scope (the issue's): retention values, key bindings, writing or migrating the file.
Doors kept open: #37 (Alt-minus and Alt-equals change the width at runtime, starting from
`sidebar_width`) reads the width from the app, where this phase puts it; #43 and #44 (an icons
setting such as `nerd_font = true`) add one key to the same table, its line to `harbour config`
and one more "wrong type" refusal.

## Verified facts (2026-11-21, `main` at d9bcb34)

- `toml = "1"` is already a dev-dependency (resolved to 1.2.1 in `Cargo.lock`, default
  features), and `Cargo.lock` lists `toml` among harbour's own dependencies already (the lock
  does not tell dev from normal). Moving it to `[dependencies]` with the same requirement adds
  no crate and leaves `Cargo.lock` as it is. `serde` with `derive` is a dependency already.
- Probed with toml 1.2.1 offline: a syntax error, a wrong type and an unknown key each come back
  as `toml::de::Error` with a byte `span()` and a one-line `message()`; for an unknown key the
  span covers the key (so its text is the key), for a wrong type it covers the value only, with
  no key in the message. `Display` is a multi-line snippet, unfit for one line. So harbour
  takes the line from the span itself, and checks each value's type itself
  (`Option<Spanned<toml::Value>>` per key, `deny_unknown_fields`), naming the key it was reading.
- The flake builds with `cargoLock.lockFile = ./Cargo.lock`, crates.io sources only, so no
  output hash is involved; with `Cargo.lock` unchanged the vendored set is the same.
- The `audit` job runs a plain `cargo audit` over `Cargo.lock`; an unchanged lock audits the
  same.
- The `version` check needs a fragment whenever the branch feeds the binary (`Cargo.toml`,
  `src/`); `changes/phase-21-config-file.md` with `kind: feat` covers the dependency move too.
- `tests/e2e/main.rs` runs harbour with `env -u XDG_CONFIG_HOME ... HOME=<scratch>`, and
  `scripts/screenshots.sh` with `env -u XDG_CONFIG_HOME ... HOME=<scratch>`, so both already
  keep the real file out. `tests/cli.rs`'s `launch` sets nothing, `launch_at_home` sets `HOME`
  but not `XDG_CONFIG_HOME`; `tests/status.rs`, `tests/summary.rs`, `tests/edit.rs`,
  `tests/e2e/edit.rs` and `tests/e2e/log.rs` launch the binary with neither.
- `src/app/split.rs` splits a third and two thirds from 60 columns, one side at a time below.
  Card and title widths are `width - 2` with `saturating_sub`, so a sidebar of 1 or 2 columns
  draws nothing inside instead of panicking.
- The accent today is `Color::Cyan` in two places: `bordered(true)` in `src/app/draw.rs` (the
  focused side, and the directory picker's popup) and the selected card's `▌` in `src/card.rs`.
- `HARBOUR_DIR_PICKER` gives `Picker::Command(vec![program])`; the default is `Picker::Fuzzy`
  (fzf over harbour's directory and zoxide's list). `HARBOUR_EDITOR` gives `Program::named`,
  Neovim when its file name is `nvim`. `HARBOUR_SESSION_COMMAND` replaces `claude` and turns off
  `--session-id`; set but empty, it runs an empty program today.
- `ratatui::style::Color::from_str` accepts the 16 names in several spellings (`lightred`,
  `light-red`, `light_red`, `LightRed`, `grey`), `#rrggbb`, and also indexed numbers and
  `reset`.

## Decisions

Each is the smallest option that meets the issue.

- **Errors name file, line and key on one line**: `harbour: <path>:<line>: <key>: <what>`.
  A syntax error names the key of its line when the line has one (`<key> =`), else none. An
  unreadable file or a directory in its place is `harbour: <path>: <reason>`: refused like a bad
  file, since its settings would otherwise be dropped silently.
- **A relative program path is refused when the file is read**, as a bad value on its line;
  `~` and `~/...` are expanded from `HOME` first; a bare name is searched on `PATH` when it
  starts, as today. Only the program (the first item) is expanded or checked, never its
  arguments. The variables keep today's handling: a relative `HARBOUR_DIR_PICKER` is still read
  from where harbour was opened.
- **An empty variable counts as unset for all three** (picker and editor did so already), so
  `HARBOUR_SESSION_COMMAND=` falls through to the file, then to `claude`, instead of running an
  empty program.
- **`picker` from the file runs fzf over the list when its program's file name is `fzf`**
  (with the arguments given), as the default does, so writing the default `["fzf"]` into the
  file changes nothing; any other program runs alone, as `HARBOUR_DIR_PICKER` does. The
  variable keeps today's meaning (`HARBOUR_DIR_PICKER=fzf` runs fzf alone).
- **`--session-id` is passed to a `session_command` from the file whose program's file name is
  `claude`, after its own arguments**, as with the default, so writing the default into the file
  changes nothing (decided in review round 1); any other program from the file, and the
  variable as today, runs as given, its transcript found through the registry. A resume runs
  the configured program and its arguments, then `--resume <id>`.
- **`editor` from the file is Neovim when its program's file name is `nvim`**, as
  `HARBOUR_EDITOR` is.
- **The accent takes every place cyan marks focus**: the focused side's border, the selected
  card's bar, and the directory picker's popup border (drawn as a focused side). Accepted values:
  what `Color::from_str` reads as one of the 16 named colours, or `#rrggbb`; indexed numbers and
  `reset` are refused.
- **`sidebar_width` is any whole number above 0**, cut to half the window (the pane keeps at
  least half), kept on every resize. A number beyond 65535 counts as 65535, cut to half anyway.
- **`harbour config` prints TOML-shaped lines**, one per setting, with its source:
  `editor = ["nvim", "--clean"]  (file)`, `(variable HARBOUR_EDITOR)`, `(default)`; the default
  width prints `sidebar_width = a third  (default)`. Its first line is the path:
  `file: <path>`, with `(none there)` when absent; `file: none (HOME is not set)` when no path
  can be made.
- **Only the interface and `harbour config` read the file.** `--demo`, `status`, `edit`,
  `log`, `changelog`, `--help` and `--version` never do, so a bad file never stops them.

## What the user sees: reading outside harbour

| Thing read | State | What the user sees |
| --- | --- | --- |
| `XDG_CONFIG_HOME` | unset | the path is `$HOME/.config/harbour/config.toml` |
| `XDG_CONFIG_HOME` | set, absolute | the path is `$XDG_CONFIG_HOME/harbour/config.toml` |
| `XDG_CONFIG_HOME` | set but empty | as unset (XDG says so) |
| `XDG_CONFIG_HOME` | relative | as unset (XDG says to ignore it) |
| `HOME` | unset, empty or relative, with no usable `XDG_CONFIG_HOME` | no path: today's defaults; `harbour config` says `file: none (HOME is not set)` |
| config path | missing | today's defaults, nothing printed; `harbour config` says `(none there)` |
| config path | empty file | today's defaults |
| config path | unreadable (permissions) | `harbour: <path>: Permission denied (os error 13)`, exit 1, no interface |
| config path | a directory in its place | `harbour: <path>: Is a directory (os error 21)`, exit 1 |
| config path | not UTF-8 | `harbour: <path>: stream did not contain valid UTF-8`, exit 1 |
| config path | a symlink to a file | read through it |
| config path | changed after start | ignored until the next start; `harbour config` reads it again each run |
| config path | syntax error (a repeated key included) | `harbour: <path>:<line>: <key>: <toml's message>`, exit 1; a control character in what is printed is escaped |
| config path | wrong type (`editor = "nvim"`, `picker = ["fzf", 3]`, `sidebar_width = "40"`, `accent = 3`) | `harbour: <path>:<line>: <key>: expected ...` (for a program: `expected a list of strings: the program, then its arguments`), exit 1 |
| config path | unknown key, a table such as `[keys]` included | `harbour: <path>:<line>: <key>: unknown key`, exit 1 |
| config path | `sidebar_width` 0, negative or `1.5` | `harbour: <path>:<line>: sidebar_width: expected a whole number of columns above 0`, exit 1 |
| config path | `accent = "pink"`, `"#12345"`, `"12"` | `harbour: <path>:<line>: accent: expected one of the 16 colour names or #rrggbb`, exit 1 |
| config path | a program list empty (`editor = []`) | `harbour: <path>:<line>: editor: expected a list of strings: the program, then its arguments`, exit 1 |
| config path | a program `""` | the same refusal as an empty list |
| config path | a program relative (`bin/vim`) | `harbour: <path>:<line>: editor: "bin/vim" is a relative path; give an absolute path, ~/... or a bare name`, exit 1 |
| config path | a program `~/bin/vim` | run as `$HOME/bin/vim` |
| config path | a program that does not exist | read fine; when it starts, today's message for that program (the picker's "could not start", the editor's failure, the session's failure in the pane) |
| `HARBOUR_EDITOR`, `HARBOUR_DIR_PICKER`, `HARBOUR_SESSION_COMMAND` | unset or empty | the file's value, else the default |
| the same | set | wins over the file; one program, no arguments, as today |
| the same | relative path | as today (the picker's from harbour's directory, the others as `exec` finds them) |
| the same | a program that does not exist | as today |
| the same | changed after start | ignored until the next start, as today |
| `harbour status`, `harbour edit`, `--demo` | any bad file | run as if there were none |

## What the user sees: the "Done when" states

| State | What the user sees |
| --- | --- |
| No file | everything as on `main`: fzf over zoxide, nvim, claude with `--session-id`, a third, cyan |
| `editor = ["nvim", "--clean"]` | C-l runs `nvim --clean --listen <socket> -- <file>` |
| `picker = ["/opt/acme/pick", "--dirs"]` | `n` runs it alone and starts in the directory it prints |
| `picker = ["fzf", "--height=40%"]` | `n` runs fzf with that option over the list, as the default does |
| `session_command = ["claude", "--model", "acme"]` | a new session runs it with `--session-id <id>` after its arguments; a resume adds `--resume <id>` after them |
| a variable and the file both set | the variable's program, alone |
| `accent = "magenta"` or `"#d7875f"` | the focused border, the selected bar and the picker's border in that colour; dots unchanged |
| `sidebar_width = 40`, window 160 | sidebar 40 columns, pane 120 |
| `sidebar_width = 100`, window 160 | sidebar 80 (half), pane 80 |
| `sidebar_width = 40`, resized to 100, then 59 | 40 then one side at a time, as today |
| `harbour config` | the path, then one line per setting with `(file)`, `(variable <NAME>)` or `(default)`; exit 0; a bad file prints the same one line as at start and exits 1 |

## What the user sees: a configured width that draws text

| State | What the user sees |
| --- | --- |
| narrowest sidebar: `sidebar_width = 1` at 60 columns | a 1-column sidebar, its borders cut, no card text, no panic; the pane 59 columns |
| `sidebar_width = 12` at 60 columns | cards and title counts cut to 10 columns by `src/text.rs`, the bar on the selected card |
| a CJK name (`阿克姆-接口`) in a 12-column sidebar | cut at a whole character, never wider than 10 columns |
| more cards than rows, configured width | the scroll keeps the selected card whole, as with a third |
| `sidebar_width = 40` at 80 columns, the window made 160 | still 40 |

## Tasks

1. This plan, its rows, phase 20's times from GitHub; `src/config.rs`: the path, the reading,
   and every refusal (in process, `tests/config.rs`).
2. `harbour config`, the refusal at start, `--demo`, `status` and `edit` ignoring a bad file;
   a test that fails while a launcher of the binary under `tests/` leaves `XDG_CONFIG_HOME` or
   `HOME` as the user's.
3. The three programs from the file, with arguments, under the variables.
4. The accent.
5. The sidebar width.
6. The accent shot, the docs (usage, `--help`, README usage line, `AGENTS.md`), the fragment,
   the outcome.

## Outcome

- Shipped on PR #90 in six ships, one per task. Every decision above held as written; none
  needed the owner.
- `toml` moved from dev-dependencies to `[dependencies]` as `"1.2.1"`, the version the lock
  already held, with its default features. `Cargo.lock` did not change, so the flake's
  `cargoLock` and the `audit` job saw the same crates as `main`: `audit`, both `flake` legs and
  `version` passed on every push.
- The variables keep today's meaning and outrank the file; the file is still parsed when a
  variable is set, so a bad file is refused whatever the environment holds.
- The accent replaces `Color::Cyan` in the three places it marked focus: the focused border
  (`bordered` in `src/app/draw.rs`, the picker popup's border included) and the selected card's
  bar (`Sidebar::draw` takes it). The state dots keep their colours.
- `sidebar_width` is a `Constraint::Length` of the width cut to half the window, the pane a
  `Fill`; below 60 columns one side shows at a time, as before.
- `tests/real_config.rs` scans `tests/` for every launch of the binary and fails while its
  function neither sets `XDG_CONFIG_HOME` nor removes it with a scratch `HOME`; it named 13
  launchers when written, all fixed in the same ship. It checks the launch line of
  `scripts/screenshots.sh` too.
- `docs/accent.png` is a new shot: the interface without `--demo` over a config file written into
  the screenshot script's scratch `HOME` (accent `#d7875f`, width 48, `pwd` as the picker, a
  `sh` session command), removed after the shot. `docs/help.png` changed with `--help`, which now
  names the file and `harbour config`; `docs/details.png` did not change.
- Review round 1 changed three things: a `picker` from the file named `fzf` that cannot start
  is refused with "could not start" (only the default fzf, missing, still starts a session at
  once); a `session_command` from the file named `claude` gets `--session-id`; `harbour config`
  quotes each word itself, so every setting stays on one line, and says when a `HARBOUR_`
  variable was ignored for not being UTF-8. A `~` program with no `HOME` is refused as such, and
  the selected card's bar is left out when the sidebar has no inner column. Its premise that a
  resume did not use the configured arguments was false: it already ran them.
- Red runs, each seen before the code: the config reading and refusals (task 1), the 13
  launchers and the startup refusals (task 2), the precedence and the three programs (task 3),
  the accent in process and through tmux's colour capture (task 4), the width in process and in
  the e2e (task 5), the README usage line (task 6). Tests green on their first run (empty
  variables, a variable over a bad file, the dots under an accent, one side below 60 columns)
  were shown red by breaking the code they cover, then restored.

## Hurdles

- `test-seam` A first width test asked for 40 columns of a 120-column window, a third, so it passed before the code existed; changed to 30 → none
- `review-loop` Round 1 found the file's `fzf` and `claude` names treated unlike the default's (a missing configured fzf skipped silently, a configured claude lost `--session-id`): the file-name rule was applied to one side of each and not the other → code
- `test-seam` The tmux colour capture writes an indexed colour as `38;5;N`, not `3N`; the first accent e2e assertion used the wrong form and was corrected, then shown red again without the wiring → none
