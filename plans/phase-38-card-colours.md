# Phase 38: colour the card's facts, dim the age and the cards that ended

Issue #136. Most of a card is drawn in the terminal's default foreground: only the state dot, the
red line of what a blocked session waits for, the dimmed prompt and the selected card's accent bar
have colour, so the sidebar reads as one white block. This phase colours the third line by kind,
dims the age, colours `exited <code>`, dims the cards that ended, draws the unknown dot in the
default foreground, and adds a `[colors]` table to `config.toml` for the three fact colours.

## Verified facts

- **A card is three lines built in `src/card.rs`** (`Card::lines`): line 1 the state glyph
  (`State::style`) then one unstyled span holding the title, the gap and the age; line 2 one span
  of what the session does, with its style (`exited<note>` in the default style, why a stopped
  card cannot resume dimmed, what it waits for in red, the prompt dimmed); line 3 one unstyled
  span, `fit` of `repository · branch · #<n> · <n> subagents`, or with `nerd_font` on
  `fit_parts` of each fact behind its glyph.
- **`fit` and `fit_parts` both return the whole text or a byte prefix of it followed by `…`**
  (`src/text.rs`, `src/card.rs`): `fit` keeps whole graphemes up to the room less one column;
  `fit_parts` drops a glyph that would keep no column of its text, cutting before that fact. So
  the styled spans can be cut from the same text at the same byte, and each fact keeps its colour
  up to the `…`.
- **The exit note** comes from `exit_note` in `src/app/draw.rs`: empty for exit 0, ` <code>` for
  another code, ` (<signal>)` for a signal.
- **A stopped card is `State::Exited` too** (`State::of(ended.is_some(), ...)`), so one check of
  the state finds every card that ended.
- **The selected card is made bold by `Buffer::set_style` over its rows** in `Sidebar::draw`, which
  adds the modifier and keeps each cell's colour, so the fact colours stay under the bold. A
  dim over an ended card's rows can be laid the same way.
- **`State::Unknown` draws `DarkGray`** in `State::style`, used by the card's dot, the zoomed
  headline (`src/zoom.rs`) and the help legend; `tests/app/status.rs` asserts `DarkGray` four times.
  The tmux status line names its own `fg=brightblack` in `state::status_line`.
- **Settings read from the file only follow `[keys]`** in `src/config.rs`: a field of `Written`
  holds the table as a `Spanned<Value>`, refused when it is not a table, then the text is read
  again into a map of spanned keys, so each refusal names the line of its key; `harbour config`
  prints `keys.<action> = "<key>"  (<source>)`. `accent` is checked by `colour`: one of the 16
  names or `#rrggbb`, refused with `expected one of the 16 colour names or #rrggbb`.
- **The demo already has every fact** (`src/demo.rs`): a pull request on two cards, two
  subagents, a stopped card, an exited one and one of unknown state, so `docs/details.png` shows
  the change without a demo change. `src/config.rs` is 620 lines of the 700 allowed.

## Decided

- Fact colours: repository blue, branch magenta, pull request cyan (each settable), subagents
  dimmed, the ` · ` separators dimmed. With `nerd_font` on, each glyph and its text take the
  fact's colour, the spaces between facts stay default, the Neovim glyph stays default.
- The `…` of a cut takes the colour of the fact it cuts (the fact holding the first byte
  dropped), so a cut fact reads in its colour up to and through the `…`.
- The age on line 1 is dimmed; the title stays in the default foreground.
- `exited` (exit 0) is dimmed; `exited <code>` and `exited (<signal>)` are red.
- An ended card (exited or stopped) that is not selected is dimmed on every row, its colours kept
  under the dim; the selected one is drawn at full strength, bold as any selected card.
- The unknown dot uses the default foreground everywhere `State::style` is used (card, zoomed
  headline, help legend). The tmux status line keeps its own `fg=brightblack`: it is not the
  sidebar.
- `[colors]` keys: `repository`, `branch`, `pull_request`, from the file only. `harbour config`
  prints them after `prefix` and before the `keys.*` lines, as `colors.<fact> = "<name>"`.
- The colours reach the cards through the `Sidebar` (`Sidebar::with_colors`), so `draw` keeps its
  arguments and a test's `Sidebar::default()` draws the defaults.

## States

### What the card draws

| Case | What shows |
| --- | --- |
| Repository alone | `acme-api` in blue |
| Repository and branch | `acme-api` blue, ` · ` dim, `fix-retry` magenta |
| Every fact | `acme-api` blue, ` · ` dim, `fix-retry` magenta, ` · ` dim, `#40` cyan, ` · ` dim, `2 subagents` dim |
| Every fact, cut in the branch | the kept part of the branch magenta, the `…` magenta, nothing after |
| Cut exactly before a separator | the `…` dim, the colour of the separator it cuts |
| Narrowest sidebar (15 columns: 9 of text after the borders, the bar and the indent) | the repository cut, blue up to and through the `…` |
| CJK branch `修正-東京` at 20 columns | `修` magenta, `…` magenta, a wide character never split |
| CJK repository `日本語-acme` with icons at 20 columns | the repository glyph and text blue, the branch glyph dropped (no column of its text fits), the `…` magenta for the branch it stands for |
| Icons, every fact | each glyph and its text in its fact's colour, the subagents glyph and count dim, the Neovim glyph default |
| Icons, a branch starting with a combining mark at 18 columns | the glyph-with-no-text rule of `fit_parts` holds: `… ` after the repository, no branch glyph |
| Selected card | the same colours, every cell bold |
| Line 1 | dot in its state's style, title default, age dim |
| Line 1 with no room for the age | title alone, no dim cell |
| `exited` (0) | line 2 dim |
| `exited 1`, `exited (KILL)` | line 2 red |
| Exited card, not selected | every row dim, the red exit and the fact colours kept under it |
| Stopped card, not selected | every row dim |
| Exited or stopped card, selected | no whole-card dim, bold, the line styles above |
| Unknown state | the `·` in the default foreground |

### `[colors]` in `config.toml`

| State | What the user sees |
| --- | --- |
| No file, or no `[colors]` | blue, magenta, cyan; `harbour config` says `(default)` for each |
| `repository = "green"` | the repository green; `colors.repository = "green"  (file)` |
| `branch = "#d7875f"` | the branch in that truecolor |
| `pull_request = "LightRed"` | light red; reported as `"lightred"` |
| `pull_request = "red"` or `"yellow"` | taken: reserved for state only as defaults |
| `branch = "pink"`, `"#12345"`, `3` | refused: `<file>:<line>: colors.branch: expected one of the 16 colour names or #rrggbb` |
| `subagents = "red"` (unknown key) | refused: `<file>:<line>: colors.subagents: unknown key` |
| `colors = 3` | refused: `<file>:<line>: colors: expected a table of fact = "colour"` |
| `HARBOUR_COLORS` or any variable | ignored: from the file only, as `[keys]` |
| The file changed while harbour runs | read once at start, as every setting |

## Tasks

1. The plan, its `PLAN.md` and `plans/metrics.md` rows, the backfill of rows 36 and 37.
2. Line 3 coloured by kind without icons, the `…` in the cut fact's colour.
3. Line 3 with icons: each glyph and its text in its fact's colour, the subagents dim.
4. Line 1's age dim, line 2's exit dim or red, the unknown dot default.
5. Ended cards dimmed when not selected.
6. `[colors]` read, refused and reported; the app draws with it.
7. `harbour --help`, the `AGENTS.md` layout lines, the screenshots, the fragment.

## Outcome

## Hurdles
