# harbour plan

The index of phases and the backlog. Each phase's plan, outcome and hurdles live in its own
file under `plans/`; its time and tokens are a row of [`plans/metrics.md`](plans/metrics.md).

## Phases

| Phase | Plan | Release |
| --- | --- | --- |
| 0 | [the foundations, scaffolded from the `rust_tui` recipe](plans/phase-0-foundations.md) | [v0.0.1](https://github.com/piacsek/harbour/releases/tag/v0.0.1) |
| 1 | [agent sessions in a sidebar, each with a live Claude Code pane](plans/phase-1-session-pane.md) | |

## Backlog

### Later phases

In this order unless a retro says otherwise.

- An editor tab beside the Claude Code tab of each session, running Neovim, with `Ctrl-h` moving
  between its splits and leaving for the sidebar only from the leftmost one. A spike showed
  Neovim can be asked over its `--listen` socket in about 0.1 ms.
- One git worktree per session: harbour creates it, and the session and its editor both start in
  it.
- What each session is doing, in the sidebar: busy, idle or waiting for the user, its current
  step and its subagents.
- Sessions that outlive harbour and are picked up again on the next start.

### Fixes to weigh

- Replace the terminal parser. `vt100` panics on a wide character cut by a resize and gives some
  emoji the wrong width; harbour contains its panics and keeps the modes the program turned on,
  but each one blanks a session's screen until it repaints. Weigh `alacritty_terminal` as the candidate.

### Bring from conveyor when the interface exists

These parts of conveyor need a product to show or drive, so phase 0 left them out. Bring each
over, renamed and with its tests, in the phase that gives harbour that interface.

- `tests/stage_dispatch.rs`, once harbour has a dispatch to keep in one place.
