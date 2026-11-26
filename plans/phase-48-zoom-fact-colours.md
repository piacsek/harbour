# Phase 48: the zoomed top row shows the repository and branch in their card colours

Zoomed, the terminal's top row reads `○ Harbour orchestrator  harbour · main  1h`, all in the
accent. The owner wants the repository and branch there drawn as the sidebar card's line 3 draws
them: the repository in `[colors] repository` (blue by default), the branch in
`[colors] branch` (magenta by default), and with `nerd_font` on the card's glyphs in place of
the ` · `.

## Verified facts

- **One function draws the headline**: `zoom::info` (`src/zoom.rs`), called by
  `App::zoom_heading` (`src/app/zoom.rs`: the zoomed top row and the narrow side-by-side
  border) and `App::side_borders` (`src/app/side.rs`: the left border side by side). All three
  change the same way, which is intended.
- **The headline holds the place as one string**, `place`, `clean(&Place::to_string())`,
  which the session finder (`src/finder.rs`) also reads as its place column.
- **Card line 3 is drawn by `Kind`, `Card::segments`, `cut` and `fit_parts`** in
  `src/card.rs`: each fact in its colour, ` · ` dimmed between them, with icons a glyph and a
  space before each fact, the `…` of a cut in the colour of the fact it cuts, a glyph that keeps
  no column of its text cut with it.
- **Only `Sidebar` holds the `Colors`** (`App::with_colors`); the app keeps `nerd_font`.
- **The block's `title_style(accent)` colours every span with no colour of its own**, so the
  title stays in the accent; a span's `DIM` adds to it.

## Decided

- The owner's: repository and branch in their `[colors]`, with `nerd_font` the card's glyphs,
  each in its fact's colour with a space before the text, in place of the ` · `; the narrow
  side-by-side border follows.
- The orchestrator's: the time shows dimmed, as on the card; the title keeps the accent; the dot
  keeps its state colour.
- The builder's: the card's fact drawing moves to `src/facts.rs` (`Colors`, `Kind`, `fitted`),
  which card line 3 and the headline share, so the two cannot drift; `Headline` holds
  `repository` and `branch` in place of `place`, and `Headline::place` gives the finder its
  `repository · branch` as before.

## States

Widths are the room `info` gets. `R` is `icons::REPOSITORY`, `B` `icons::BRANCH`.

| Headline | Room | `nerd_font` | The user sees |
| --- | --- | --- | --- |
| `Fix the login`, `acme-api`, `main`, `3m` | 60 | off | ` ● Fix the login  acme-api · main  3m `, `acme-api` blue, ` · ` dimmed, `main` magenta, `3m` dimmed, the rest in the accent |
| the same | 60 | on | ` ● Fix the login  R acme-api B main  3m `, `R acme-api` blue, `B main` magenta |
| the same | 30 | off | ` ● Fix the login  acme-a…  3m `, the `…` blue (the repository is cut) |
| the same | 36 | off | ` ● Fix the login  acme-api · m…  3m `, the `…` magenta |
| the same | 38 | on | ` ● Fix the login  R acme-api B m…  3m `, the `…` magenta |
| the same | 37 | on | ` ● Fix the login  R acme-api …  3m `: a branch glyph that keeps no column of its text goes with it |
| the same | 20 | either | ` ● Fix the log…  3m `: no place, as today |
| the same | 9 | either | ` ● …  3m `, as today |
| the same | 3 | either | ` ● `; under 3 nothing, as today |
| no branch | 60 | off | ` ● Fix the login  acme-api  3m `, `acme-api` blue |
| no title | 60 | on | ` ● R acme-api B main  3m ` |
| `修正ログイン`, `acme-api`, `機能/ズーム` | 40 | off | ` ● 修正ログイン  acme-api · 機能/…  3m `, cut by display width, the `…` magenta |
| `修正ログイン`, `港口`, `機能` | 60 | on | ` ● 修正ログイン  R 港口 B 機能  3m `, CJK in each fact's colour |
| `[colors] repository = "green"` | any | either | the repository and its glyph green |
| a 9-column zoomed terminal | 9 | either | ` ● …  ●1 `: the counts and the dot keep their room, as today |
| the session finder's list | | | `repository · branch` as before, never coloured (fzf draws it) |

## Tasks

1. `src/facts.rs` takes card line 3's drawing; card tests stay green, untouched.
2. `zoom::info` takes `Colors` and the icons flag; `Headline` holds `repository` and `branch`
   (`tests/zoom.rs` red first); the app keeps its `Colors` and passes them and `nerd_font`
   (`tests/app/zoom_border.rs`, one wiring test); the `AGENTS.md` entries, the fragment, the
   screenshot.

## Outcome

## Hurdles
