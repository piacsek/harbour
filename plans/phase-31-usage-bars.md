# Phase 31: the usage popup draws its report as bars

## Context

Issue #107. The usage popup of `u` (phase 26) shows Claude Code's `/usage` report as it prints
it. This phase draws it instead: one bar per limit, one short bar per line of each breakdown,
the `Top ...` lines as label and values. `t` switches to the report as printed and back. The
text stays the fallback whenever the limit lines do not parse.

## Verified facts (2026-11-21, `main` at b190e4d)

Probed once with `claude` 2.1.330, `claude -p --no-session-persistence --setting-sources user
/usage`, in a scratch folder under the temporary directory, through `script` at 80 columns. The
shapes below are written in invented wording and numbers; nothing the probe printed is copied.

- The first line says what powers the usage (a subscription), then a blank line.
- Three limit lines, not indented, each `<name>: <n>% used · resets <when> (<time zone>)`: the
  name is `Current session`, `Current week (all models)` and `Current week (<model>)`.
- Then a question line ending in `?`, a line of caveats joined with ` · `, and one section per
  period: a heading not indented (`Last 24h · <n> requests · <n> sessions`), then lines indented
  two spaces: `<n>%  <tip>`, one per tip, then `Top <kind>: <name> <n>%, <name> <n>%`.
- Long lines are not broken by Claude Code: they run past the PTY width, so the terminal wraps
  them. `vt100`'s `Screen::contents` joins a soft-wrapped row with the next one (no newline
  between them), so the parser sees each line whole whatever the popup's width was.
- Unverified: an account with no weekly limit, one with no model line, and one billed by API
  key could not be probed here. The parser needs at least one limit line; anything else shows
  the report as printed, as today.
- Whether the report holds anything else worth a bar (extra usage, credits): nothing of that
  kind showed for this account; unverified for others. A line of the limit shape with any other
  name (`<name>: <n>% used`) is drawn as a limit too.

## Decisions

- **The parser is a pure function** in `src/usage.rs`: `parse(text) -> Option<Report>`, the
  limits, the breakdowns and the notes. `None` when no limit line parses. A limit is any line
  not indented of the shape `<name>: <n>% used`, with ` · resets <when>` kept as printed minus a
  trailing `(<time zone>)`. Its label drops `Current `, drops ` (all models)`, and turns
  ` (<model>)` into `, <model>`, first letter upper case: `Session`, `Week`, `Week, Fable`.
- **Sections.** After the first limit, a line not indented opens a section and the indented
  lines below it belong to it. A section with at least one `<n>%` or `Top <kind>:` line is a
  breakdown; one with no indented line is a note, shown at the end, unless it ends with `?` (the
  report's question heading); one whose indented lines all fail to parse is left out. An
  indented line that is neither (a line the report broke itself) continues the line before it.
  Lines before the first limit are left out.
- **When the bars show:** only once the popup would say `printed nothing` or an exit today
  (its process exited and its output read to the end, or half a second after the exit), the
  report printed something and it parses. Before that, `loading…`, then the text as it arrives.
  The report is parsed again each frame once finished (a 200-row screen, cheap), so nothing is
  cached that could go stale.
- **`t` or `T`** with no modifier but Shift switches between the bars and the report as printed
  only while the bars can show; otherwise it goes to the report's process like any other key, as
  today. A popup opened again starts on the bars.
- **The hints.** The bars view ends with a blank line and `t raw text · Esc close`, kept as the
  last row when the drawing is cut. The report as printed, switched to with `t`, names
  ` t bars ` on the popup's bottom border, a free spot; the fallback text shows no hint.
- **Layout at 80 columns** (78 inside): limit rows indented 2, the label padded to the longest,
  3 spaces, a 24-cell bar, the percentage right-aligned in 4 columns after 2 spaces, 3 spaces,
  the reset. Breakdown rows indented 3, a 12-cell bar, the percentage, 2 spaces, the tip. Top
  rows indented 3, the kind padded to the longest kind, 3 spaces, the values with `, ` shown as
  ` · `. Bars count `█` for the share rounded to the nearest cell, `░` for the rest.
- **Narrower:** every bar loses one cell per column the inside is under 78, down to 4 cells;
  then each line is cut at the right with `…` (`src/text.rs`).
- **Colour:** a limit bar is green under 70%, yellow from 70%, red from 90% (the named colours
  of the terminal's palette, so a 16-colour terminal draws them); breakdown bars take the
  accent. A percentage above 100 fills the bar and prints as given.
- No row in `KEYS` (the help view has no spare row); `t` is named in the popup itself.
  `harbour --help` names no key of the popup today, so it stays as it is.

## What the user sees

| State | The popup shows |
| --- | --- |
| The report is loading | `loading…`, as today |
| It prints, still running | the text as it arrives, as today |
| It finished and parses | the bars, `t raw text · Esc close` on the last row |
| The limit lines do not parse | the report as printed, no hint, no message |
| The limit lines parse, a breakdown does not | the limit bars, without that breakdown |
| A limit the report does not name (no model line) | no bar for it |
| A limit line of another name (`Extra usage: 5% used`) | a bar for it, under that name |
| A percentage above 100 | a full red bar, the number as printed |
| A breakdown line with a percentage and no tip | its bar and number, no text |
| An unknown heading with lines that do not parse | left out of the bars; `t` shows it |
| A line the report broke itself, its rest indented | joined to the line before it |
| A line the terminal wrapped at the popup's width | read whole (`vt100` joins soft-wrapped rows) |
| A CJK tip or label | measured by display width, padded and cut by it |
| The report prints nothing, or its command fails | `printed nothing` or `exit <code>` in the title, as today; a failed report that still parses shows the bars under that title |
| The popup is narrower than 80 columns (small terminal) | the bars shrink first, down to 4 cells, then each line is cut with `…` |
| The narrowest popup (a terminal of a few columns) | each line cut to the inside, `…` last; the border stays whole |
| The popup is shorter than the drawing | the limit bars first, what does not fit left out from the end, the hint kept on the last row |
| The terminal resized while it is open, narrower then wider | the bars drawn again for each width; the text view keeps its PTY width, as today |
| `t` pressed while loading | sent to the report's process, as today; the bars show once it finishes |
| `t` pressed on the bars | the report as printed, ` t bars ` on the bottom border; `t` again, the bars |
| `t` on the fallback text | sent to the report's process; nothing changes |
| Ctrl-t, Alt-t | sent to the report's process; nothing changes |
| Esc, `q`, `Q` | closes, as today, from either view |
| No colour for the accent, a 16-colour terminal | the named green, yellow and red of the terminal's palette |
| The narrowest sidebar, the narrow layout | the popup over the whole terminal, as today |

## Tasks

1. The limit bars: `parse` reads the limit lines, the popup draws one bar per limit once the
   report finished, and the fallback stays the text.
2. The breakdowns and the notes, wrapped and broken lines, CJK.
3. `t` switches views, the hints.
4. Narrower and shorter popups, the colours, a percentage above 100.
5. The demo's report, the screenshots, the fragment, the guidelines.

## Outcome

- Shipped as planned, in five tasks: the limit bars, the breakdowns and notes, `t` and the
  hints, the cut and the colours, the demo and `docs/usage.png`.
- The bars show only once the report finished; the existing app tests whose made-up reports
  read `<n>% used` were reworded to `% spent`, so they keep testing the text view.
- `lines` cuts each line span by span: a span that overflows, or fills the room while more
  follows, is cut with `text::fit`, so a line that does not fit always ends in `…`.
- Unverified, as in "Verified facts": the report of an account with no weekly limit, no model
  line, or an API key.

## Hurdles

- `test-seam` The first ship failed on the end-to-end demo test, which waited for the report's
  raw text that the bars now replace → code
- `design` Two expected rows of the drawing tests were counted by hand wrongly (a 4-cell bar's
  rounding, a 58-column inside taken for 60) and fixed in the test before the code → none
- `review-loop` Round 1 found the parser trusting the report's shape: a share printed as `<1%`,
  `~5%` or `12,5%` glued onto the tip above, its text drawn without `text::clean`, wider
  percentages out of line; plus `t` logged as a key and two tests judging the screen too early
  → code
