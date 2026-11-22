# Phase 35: choose the glyph of the Claude pane

With `nerd_font = true`, the pane border draws `md-creation` (U+F0674, a sparkle) before
`claude`. A user with a font that maps a Clawd glyph to U+100000 (the first code point of the
Supplementary Private Use Area-B) wants that glyph there instead.

## Verified facts

- **Ghostty** maps a code point to a font with `font-codepoint-map = U+100000=<family>`, and
  `printf '\U00100000 \n'` then draws the glyph (seen on the owner's machine).
- **The glyph is drawn in three places**: the pane border (`tabs::iconed`), and side by side the
  left and the swapped right border (`tabs::claude` through `src/app/side.rs`).
- **harbour ships no font**: the Clawd art is Anthropic's, so the repository holds only the code
  point and the setting, never a font or the drawing.

## Decided

- **A setting `claude_icon`**, from the file only, like `nerd_font`: `"sparkle"` (the default,
  U+F0674) or `"clawd"` (U+100000). Any other value is refused as
  `<file>:<line>: claude_icon: expected "sparkle" or "clawd"`.
- **It changes only the glyph `nerd_font` draws**: with `nerd_font = false` no glyph is drawn,
  whatever `claude_icon` says.
- **`harbour config`** prints `claude_icon = "sparkle"  (default)` after `nerd_font`.
- **`harbour --help`** names the setting, the code point and Ghostty's `font-codepoint-map`
  line; the README stays as it is.

## States

| `claude_icon` | `nerd_font` | What the user sees |
| --- | --- | --- |
| absent | false | `claude` |
| absent | true | the sparkle, `claude` |
| `"sparkle"` | true | the sparkle, `claude` |
| `"clawd"` | true | U+100000, `claude` |
| `"clawd"` | false | `claude` |
| `"Clawd"`, `"x"`, `1`, `true` | any | harbour refuses to open, in one line naming the file, the line and `claude_icon` |
| `"clawd"`, no font maps U+100000 | true | the terminal's missing-glyph box, one column, then `claude` |
| `"clawd"`, a terminal drawing it double-wide | true | the border shifts one column, as for every icon ("Known limits") |
| `"clawd"`, the narrowest sidebar | true | the border fits as with the sparkle: the directory goes first, then both glyphs together, never a glyph without its name |
| `"clawd"`, a CJK directory | true | the directory is cut from its start by display width, the glyph and `claude` kept whole |

The glyph counts as one column like every icon, so how the border fits a width, the narrowest
sidebar and a CJK directory included, is the same as with the sparkle.

## Tasks

1. `claude_icon` read, refused and reported by `src/config.rs`.
2. The chosen glyph on the pane border and the side-by-side borders.
3. `harbour --help`, the `AGENTS.md` lines, the fragment.

## Outcome

### Sweep

Range: phases 30 to 34. The findings were drafted (missed states in round 1 of every phase, the
parallel builds' collisions over `PLAN.md` rows and a phase number, tasks that could not ship
apart, two token cells left `unknown`, four one-offs) and the owner parked the sweep before
deciding any of them; nothing was applied, and deciding them is a backlog line in `PLAN.md`. Phase 34's ready, merged and released were read from
GitHub into `plans/metrics.md`.

## Hurdles
