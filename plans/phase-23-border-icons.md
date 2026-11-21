# Phase 23: icons beside Claude Code and Neovim in the pane's border

## Context

Issue #43. The pane's border names what a session can show as plain text:
`claude | nvim: <directory>`. With one setting turned on in the config file of phase 21, each
name gets a Nerd Font glyph in front of it: `md-creation` (a sparkle) before `claude`,
`custom-neovim` before `nvim`. Another editor (`HARBOUR_EDITOR` or the file's `editor`, not
named `nvim`) gets none. Off by default: harbour cannot tell which font a terminal uses, and
without a Nerd Font the glyphs show as empty boxes.

Out of scope: the card's icons (#44), which come right after with the same setting. This phase
names the setting and puts the glyphs in `src/icons.rs`, so #44 adds its own glyphs there and
reads the same setting from the app.

## Verified facts (2026-11-21, `main` at 9c68b8f)

- The Nerd Fonts glyph list, `glyphnames.json` of `ryanoasis/nerd-fonts` (its `METADATA` says
  version 3.5.1, dated 2026-08-21), gives `md-creation` as `f0674` and `custom-neovim` as `e6ae`.
- The font the screenshots use, `~/Library/Fonts/FiraCodeNerdFont-Regular.ttf`, maps both in its
  `cmap` (format 12 subtable, read with a short Python script, no font library): U+F0674 to glyph
  6727 and U+E6AE to glyph 2192, each with an advance of 1200 units in `hmtx`, the same as `c`.
  The Mono variant gives the same glyphs and advances.
- `text::columns` measures a grapheme with ratatui's `CellWidth`, which is `unicode-width`: both
  code points are private use, East Asian Width "ambiguous", and count one column. ratatui's
  buffer gives each one cell.
- tmux 3.7c, the terminal the screenshots capture, moves the cursor one column for each glyph
  (a scratch server printing the glyph alone, then `#{cursor_x}` = 1, as for `c`). So harbour,
  ratatui and tmux agree on one column. A terminal that draws the glyph two cells wide while
  counting one overlaps the next cell, which is the space that follows each glyph.
- `strip` in `src/tabs.rs` builds both names; `App::tabs` in `src/app/draw.rs` calls it with the
  editor program's `name()` and the editor's directory label, and draws nothing once `claude`
  comes back empty. `Program::is_neovim` is true for a program whose file name is `nvim`.
- `src/config.rs` reads each key as `Option<Spanned<toml::Value>>` with `deny_unknown_fields`
  and checks its type itself; a wrong type is a `Refusal` `<path>:<line>: <key>: <what>`.
  `harbour config` prints one line a setting with its source.
- `--demo` never reads the config file, so the demo's shots keep the plain border. The `accent`
  shot of `scripts/screenshots.sh` runs the interface over a config written into the scratch
  `HOME` and removed after it.

## Decisions

- **The setting is `nerd_font = true`**, not `icons`. It states the one fact harbour cannot find
  out by itself, that the terminal draws with a Nerd Font, which is the reason it is off by
  default; every glyph harbour draws from it, the border's now and the card's of #44, depends on
  exactly that fact. A boolean, default `false`, from the file only (no `HARBOUR_` variable, as
  the issue decides). Any value but `true` or `false` is a bad file:
  `<path>:<line>: nerd_font: expected true or false`. `harbour config` prints
  `nerd_font = false  (default)` or `nerd_font = true  (file)`.
- **The glyphs live in `src/icons.rs`** as `CLAUDE` (U+F0674) and `NEOVIM` (U+E6AE), the only
  two used now; #44 adds its own there and reuses `NEOVIM`.
- **`strip` stays as it is** and serves the plain border; a new `tabs::iconed` serves the
  border with icons, and falls back to `strip` when the names do not fit whole beside their
  glyphs. `tests/tabs.rs`'s table stays unchanged and still covers the plain border.
- **The glyphs go before the editor's name is cut.** With icons on, the order is: the directory
  loses its start, then the directory goes, then both glyphs go together, then the editor's
  name is cut, then everything. So a glyph shows only beside its whole name, which satisfies
  "never without at least one column of its name" with room to spare, and a narrow border keeps
  what the issue says matters most, the names ("glyph and name, not the glyph alone"). The other
  order (cutting the name beside its glyph, then dropping the glyphs) would show `n…` beside a
  glyph at one width and `nvim` without it one column narrower. The fallback is exactly the
  plain border at that width; its directory cannot come back, since the plain border needs more
  room for a directory than the iconed one needs for the editor's whole name.
- **A space follows each glyph** (`<glyph> claude`, `<glyph> nvim: <directory>`), and the
  glyph is counted one column, as harbour, ratatui and tmux all do.
- **The app takes it as `App::with_nerd_font`**, set by `tui::run` from the config; the demo
  never sets it.

## What the user sees: reading outside harbour

The setting is the one new thing read from outside: a key of the config file.

| State | What the user sees |
| --- | --- |
| no file, or no `nerd_font` key | the plain border, byte for byte as on `main`; `harbour config`: `nerd_font = false  (default)` |
| `nerd_font = true` | the border with glyphs; `harbour config`: `nerd_font = true  (file)` |
| `nerd_font = false` | the plain border; `harbour config`: `nerd_font = false  (file)` |
| `nerd_font = "yes"`, `1`, or any non-boolean | the interface refuses to open: `harbour: <path>:<line>: nerd_font: expected true or false`, exit 1; `harbour config` prints the same and exits 1 |
| the file changed while harbour runs | nothing until the next start (read once, as every setting) |
| `nerd_font = true` on a terminal without a Nerd Font | an empty box or a missing-glyph mark before each name; the names still read |
| `--demo` with `nerd_font = true` | the plain border: the demo never reads the file |

## What the user sees: the border with icons, by width

`G` stands for a glyph, one column. `width` is the border's inner width, the pane's width minus
2. The editor is `nvim` unless the row says otherwise.

| State | Border reads |
| --- | --- |
| room for all, directory `~/work/acme/api` | `G claude \| G nvim: ~/work/acme/api` |
| another editor, `acme-edit`, no directory | `G claude \| acme-edit` |
| the directory does not fit whole (width 31) | `G claude \| G nvim: …/acme/api` |
| a CJK directory `~/acme/日本語` (width 26) | `G claude \| G nvim: …本語`, cut at a whole character |
| not one column of the directory fits (width 22) | `G claude \| G nvim` |
| the editor's whole name no longer fits beside the glyphs (width 18) | `claude \| nvim`, the plain border |
| narrower (width 13) | `claude \| n…`, as the plain border |
| the narrowest pane (width 6) | nothing, as the plain border |
| icons off, any width | exactly what `strip` gives on `main` |

## Tasks

1. This plan, its rows, phase 21's times from GitHub; the `nerd_font` setting in
   `src/config.rs`, its refusal and its `harbour config` line.
2. `tabs::iconed` and `src/icons.rs`, the table of how the iconed border fits.
3. The app draws the iconed border with the setting on, the plain one byte for byte with it off;
   `tui::run` hands the app the setting.
4. The icons shot, `--help`, `AGENTS.md`, the fragment, the outcome.

## Outcome

- The setting is `nerd_font` (`true` or `false`, from the file only, `false` by default). It names the one fact harbour cannot detect, that the terminal draws with a Nerd Font, which is why it is off by default, and it reads the same for the card icons of #44, which only add their glyphs to `src/icons.rs` and draw them where `nerd_font` is on.
- Code points checked two ways: Nerd Fonts' `glyphnames.json` (v3.5.1) gives `md-creation` as `f0674` and `custom-neovim` as `e6ae`; the cmap of FiraCode Nerd Font maps U+F0674 and U+E6AE to glyphs with the advance of `c`.
- Width: unicode-width (ratatui's measure) counts both glyphs as one column, and tmux 3.7c moves the cursor one column for each, so the border's fit needed no special case.
- `tabs::iconed` fits the names with the glyphs: the directory's start goes first, then the directory, then both glyphs together, then the rest exactly as `strip` fits it. A glyph never shows without its whole name, and once the glyphs go the directory does not come back.
- With `nerd_font` off the app still calls `strip`: the `tests/tabs.rs` table is unchanged, and an app test pins the drawn top border character for character.
- `docs/icons.png` is the new shot (a scratch config with `nerd_font = true` and an editor script named `nvim`); `docs/help.png` changed because `--help` names the setting; every other shot is byte for byte the same.
- 1 review round, with no must or should finding. On the reviewer's notes, `--help` now names which settings a variable can outrank (editor, picker, session_command) and which are read from the file only, and the one-column count of the glyphs is a known limit.
- Decided, not changed: the icons show in `docs/icons.png` and not in `docs/details.png`, because the issue says the demo keeps the plain border and adds one shot.
- Task 2 could not ship alone: the dead-code gate refused `iconed` before the app called it, so tasks 2 and 3 shipped as one commit.

## Hurdles

- `process` The task split put `tabs::iconed` in a ship of its own, before anything in `src/` called it, and `tests/dead_code.rs` refused the ship; tasks 2 and 3 shipped together → none
- `test-seam` The fallback tests of `iconed` and the icons-off border test passed on their first run (the stub already fell back to `strip`, and the off path did not change); each was shown red by breaking the code it covers, then restored → none
