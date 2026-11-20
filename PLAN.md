# harbour plan

The index of phases and the backlog. Each phase's plan, outcome and hurdles live in its own
file under `plans/`; its time and tokens are a row of [`plans/metrics.md`](plans/metrics.md).

## Phases

| Phase | Plan | Release |
| --- | --- | --- |
| 0 | [the foundations, scaffolded from the `rust_tui` recipe](plans/phase-0-foundations.md) | [v0.0.1](https://github.com/piacsek/harbour/releases/tag/v0.0.1) |
| 1 | [agent sessions in a sidebar, each with a live Claude Code pane](plans/phase-1-session-pane.md) | [v0.1.0](https://github.com/piacsek/harbour/releases/tag/v0.1.0) |
| 2 | [the selected session and the focused side in cyan](plans/phase-2-cyan-highlight.md) | [v0.2.0](https://github.com/piacsek/harbour/releases/tag/v0.2.0) |
| 3 | [whether each agent is working, idle or waiting on you](plans/phase-3-status-dots.md) | [v0.3.0](https://github.com/piacsek/harbour/releases/tag/v0.3.0) |
| 4 | [choose the directory a new session starts in, by fuzzy search](plans/phase-4-directory-picker.md) | [v0.4.0](https://github.com/piacsek/harbour/releases/tag/v0.4.0) |
| 5 | [the counts in the tmux status line](plans/phase-5-tmux-status-line.md) | [v0.5.0](https://github.com/piacsek/harbour/releases/tag/v0.5.0) |
| 6 | [an editor beside each session's Claude Code pane](plans/phase-6-editor-column.md) | [v0.7.0](https://github.com/piacsek/harbour/releases/tag/v0.7.0) |
| 7 | [a faster macOS flake build: a Nix store cache, and no clippy or rustfmt in the flake's toolchain](plans/phase-7-flake-build-speed.md); the cache was dropped after measuring it | [v0.5.1](https://github.com/piacsek/harbour/releases/tag/v0.5.1) |
| 8 | [each session's title, activity, repository and branch in the sidebar](plans/phase-8-session-card.md) | [v0.6.0](https://github.com/piacsek/harbour/releases/tag/v0.6.0) |
| 9 | [no script dies of SIGPIPE when its reader stops early](plans/phase-9-pipe-race.md) | |
| 10 | [sessions that come back after harbour closes, resumed on demand](plans/phase-10-resumable-sessions.md) | |
| 11 | [tmux shortcuts that use the current directory follow the selected session](plans/phase-11-follow-selected-session.md) | |

## Backlog

### Later phases

In this order unless a retro says otherwise.

- One git worktree per session: harbour creates it, and the session and its editor both start in
  it.
- What each session is doing, in the sidebar: its current step and its subagents.
- Sessions that outlive harbour and are picked up again on the next start.

### Fixes to weigh

- Replace the terminal parser. `vt100` panics on a wide character cut by a resize and gives some
  emoji the wrong width; harbour contains its panics and keeps the modes the program turned on,
  but each one blanks a session's screen until it repaints. Weigh `alacritty_terminal` as the candidate.

### Bring from conveyor when the interface exists

These parts of conveyor need a product to show or drive, so phase 0 left them out. Bring each
over, renamed and with its tests, in the phase that gives harbour that interface.

- `tests/stage_dispatch.rs`, once harbour has a dispatch to keep in one place.
