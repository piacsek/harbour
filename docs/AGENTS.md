# docs/AGENTS.md

The screenshots, and the documentation rule in full. The root `AGENTS.md` keeps its summary.
`docs/process.md` holds the working agreements and `docs/known-limits.md` the accepted limits.

## Documentation rule

`README.md` is minimal and holds only what a user needs, in this order and nothing else: the
`# harbour` heading, the one line "Harbour: where shipping begins.", then `## Installation`, `## Usage`
and `## Development`, with no subsection. `tests/readme.rs` fails when that shape changes, when
the tap, the Nix or the source install line (built from `stable`) goes, when the usage stops showing `--version` and `--help`, or when the key table stops mirroring `KEYS` of `src/help.rs`, the keys the help view shows.
Feature depth belongs in the `AGENTS.md` of the directory it is about, never in the README or the root `AGENTS.md`.
**Every visible change to the main interface shows in `docs/details.png`**: a new or changed
line, glyph, bar, title, column or footer element. The `details` shot of `scripts/screenshots.sh`
takes it from `harbour --demo`, `README.md` embeds it right below the description, and
`tests/readme.rs` pins it there as the README's only image. Extend the demo (`src/demo.rs`)
until the change appears on screen, rerun the script, and commit the image in the same pull
request; the orchestrator's review reports a missing or stale screenshot as a should. The script also shoots
`docs/help.png`, the output of `harbour --help`, which the README does not embed. Run `scripts/screenshots.sh` (needs tmux, fzf, `brew install
charmbracelet/tap/freeze`, Google Chrome or chromium, and the FiraCode Nerd Font in
`~/Library/Fonts`) and commit the new images. Freeze lays out an SVG with the font embedded;
headless Chrome rasterises it, because freeze's own PNG output cannot draw Nerd Font glyphs.
The script adds Menlo (DejaVu Sans Mono on Linux) as the SVG's fallback font family: FiraCode
lacks `✗ ⊘ ▸`, and Chrome's default fallback is wider, so in conveyor every row holding one
shifted and the borders jogged. Without the font file the SVG names the family alone and Chrome
falls back; without Chrome freeze writes the PNG itself; `CHROME` names another browser binary.
The script runs the release binary in a scratch tmux server with `HOME` a tempdir, through `env`
with `XDG_CONFIG_HOME`, `XDG_STATE_HOME` and `CLAUDE_CONFIG_DIR` unset, since all three outrank `HOME`, and `TMPDIR`
under that `HOME`, the random name of the demo's scratch directory replaced in each capture so
the editor shot's border reads the same every run. Demo data looks
real (SHAs like `8c1f2a7d…`, not `33333333`). Do not screenshot real data: this repository is
public.

Whenever behaviour changes, check that `README.md` is still true: an argument, a subcommand, an
install path. Do not add prose for a feature; if it needs explaining, the binary's help should
carry it.
