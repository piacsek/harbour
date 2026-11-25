# Known limits

Accepted limits, one line each, with where they are written down. The orchestrator reads this
file and treats them as decided when it rates review findings, and so does the `adversarial-review`
skill. The `/code-review` agent does not read it (the root `CLAUDE.md` only points here), so
filtering its findings against this list is the orchestrator's job. Pointing reviewers here is not
priming, since the list is in the repo. A limit joins it when a review finding is declined for
good; a finding on one still counts as must when it shows new impact.

- **The `version` check judges "feeds the binary" by path only**, can be pointed by cargo at
  manifests outside its copy, and refuses a fragment name git would quote
  (`.github/AGENTS.md`, `version.yml`).
- **`check` builds and tests on Linux only**: the tests never run on macOS, and a macOS-only
  break of the release build itself fails in the release run; the flake's macOS leg catches a
  plain macOS build break (`.github/AGENTS.md`).
- **C-h over Neovim, `x`, `q`, C-l from Claude Code, `side` and a jump while side by side, and
  swapped C-h from Claude Code and C-l over Neovim do their I/O on the key thread**: C-h, swapped
  C-l, `x` and `q` ask each Neovim `nvim_get_mode`, then `nvim_eval`, within one 100 ms deadline (a
  Neovim that never answers costs that much per key; C-h then shows Claude Code, a blocking
  Neovim gets the key, and `x` and `q` say it may have unsaved changes, `src/nvim.rs`); C-l
  reads the session's registry file, then lists the transcript projects for the transcript of
  the session id it follows and reads what was appended to it, at most its last MiB
  (`src/transcript.rs`), and so does swapped C-h from Claude Code, which opens the editor as C-l
  does; `side` and a jump while side by side (the prefix then a digit, a pick from the
  session finder, a stopped card that resumes) do the same reads for where the session works when
  it has no editor yet, and start that editor (`src/app/side.rs`); closing the usage popup while its report still loads hangs up its
  process group and waits up to 200 ms, killing it only if it is still alive (`src/session.rs`).
- **C-j and C-k under tmux run tmux on the key thread**: from the sidebar or Claude Code each
  press runs `tmux list-keys`, then `tmux display-message` when tmux binds the key, then
  `tmux select-pane` when a pane lies that way (about 7 ms each measured), the first two within
  one 100 ms deadline and the move within one of its own, so a stalled tmux server delays that
  key by at most 200 ms; a stalled check lets the key go on as without tmux, a stalled move
  takes the key whether or not tmux moved (`src/tmux.rs`).
- **An open of `harbour edit` into a running Neovim runs on the draw thread**: the listener's
  threads accept and read requests, but the RPC to Neovim (`nvim_get_mode`, retried while its
  socket is not up yet, then `nvim_cmd`, its mode asked every 50 ms while that waits, so a prompt it raises is refused within 50 ms) runs on the draw thread, at most 1 s each for a
  running Neovim and the whole request deadline (under 5 s) for one harbour has just started,
  4 a frame, so a Neovim that never answers holds the interface that long; once the command is
  sent, it may still land after `harbour edit` reported a failure (a swap prompt answered after
  harbour gave up still switches the buffer); every ask and every open first reads again where
  each running session works, as the draw thread's periodic read does (its registry file, the
  new end of its transcript, at most 1 MiB, and a listing of the projects directory at most every
  5 s while it has no transcript), so each `harbour edit` costs those reads once per session in every harbour it
  asks, then again in the one it opens in; a state directory path too long for a
  Unix socket (104 bytes on macOS) means harbour never listens and `harbour edit` says none is
  running (`plans/phase-12-edit-from-outside.md`).
- **A harbour ended by any signal but TERM, INT and HUP, SIGKILL included, or a crash, or a second TERM or INT that ends it before the clean-up, leaves its `harbour edit` socket behind** until the next
  harbour to start sweeps it: senders skip it, since its connect is refused; under a flood from
  the same user, a starting harbour can take a live harbour's socket for dead and sweep it; the
  listener binds again within 5 s (`src/handoff.rs`).
- **Every editor shares harbour's own tmux pane**: each one gets harbour's `TMUX` and
  `TMUX_PANE`, so a plugin that keeps state on "its" pane (a pane option, a renamed window,
  passthrough) collides between editors, and an editor ended by hangup leaves that state behind;
  a pane-navigation plugin may run `tmux select-pane` at a Neovim window edge and move tmux's
  focus out of harbour, C-h in the leftmost window in terminal or insert mode included (harbour
  hands it to Neovim), not checked with a real plugin; every program the editor starts (a
  `:terminal` shell, plugin jobs) inherits both too, so tmux run there acts on harbour's pane and
  a nested `tmux new` is refused (`src/editor.rs`).
- **Each `u` leaves a folder in the Claude Code configuration**: `-p /usage` makes one empty `projects/<encoded usage folder>/memory/` for harbour's usage folder, once, and one `session-env/<id>/` holding the user's `SessionStart` hook script per press, which harbour never removes (`plans/phase-26-usage-popup.md`).
- **While the usage report of `u` loads, `t` switches the popup's view instead of reaching the report**: a report waiting for typed input could never receive a `t`; `claude -p` takes none (`plans/phase-33-usage-loading.md`).
- **One kitty keyboard stack per session, not one per screen**: the kitty protocol keeps separate flags for the main and the alternate screen, harbour one set, so a program that pushes its flags on one screen and leaves them on the other gets keys in the form it left behind, such as `ESC [ 27 ; 5 u` for Ctrl-Esc where a plain Esc was expected (`src/keys.rs`).
- **An editor's empty socket directory stays behind when any signal but TERM, INT and HUP, SIGKILL included, or a crash ends harbour**, or a second TERM or INT ends it before the clean-up: each
  Neovim's private directory in the temp directory is removed when its editor is dropped, which
  neither reaches (`src/editor.rs`).
- **A harbour ended by any signal but TERM, INT and HUP, SIGKILL included, leaves its terminal modes on**, as does a second TERM or INT that ends harbour before the clean-up: mouse capture, bracketed paste
  and the extended-keys request stay set, so at the shell after it a click or the wheel prints
  escape codes, a paste comes wrapped in `^[[200~`, and Ctrl-C, Ctrl-D, Shift-Enter or Ctrl with
  any key prints an escape code such as `^[[27;2;13~` (`^[[27;5;104~` for C-h) until `reset`;
  a frame cut by it leaves synchronized output on, so the terminal shows nothing new until its
  own timeout for that mode ends it;
  a quit, a panic, a first TERM or INT and any HUP turn them off (`src/tui.rs`).
- **A session that exited by itself leaves behind what ignores hangup**: once its command is
  reaped its pid can be reused, so harbour does not signal its group again, and a background
  process that ignores hangup keeps running after its row is removed or harbour quits, as in any
  terminal (`src/session.rs`). The one exception is the hangup grace: harbour may reap a
  session's command during it and still SIGKILL its group when it ends, but only while a probe
  just before shows the group has members. A group number reused inside that 5 ms window is
  accepted as unreachable in practice, because keeping the command unreaped instead makes every
  quit on Linux wait the full grace.
- **Harbour keeps its working directory in the selected session's directory**, an exited
  session's too, so it can keep a volume busy until another session is selected or harbour
  quits (`plans/phase-11-follow-selected-session.md`).
- **tmux's `#{pane_current_path}` follows only when harbour leads its pane's foreground process
  group**: typed at a shell prompt, or a pane command or wrapper script ending with
  `exec harbour`, which `harbour --help` says (`plans/phase-11-follow-selected-session.md`).
- **A subagent whose meta file never says `"finished": true` counts as running** until its session
  exits, as one interrupted inside a live session may (unverified) (`plans/phase-8-session-card.md`).
- **No tag ruleset guards `v*` tags**: the owner can still push or delete one by hand
  (`.github/AGENTS.md`).
- **A Nix outage blocks merges that touch the flake's inputs once the ruleset requires
  `flake-ok`**: a Nix, nixpkgs or `cache.nixos.org` outage then holds them until the ruleset is
  disabled; requiring it is the owner's step by hand (`.github/AGENTS.md`, the flake build, has
  the command that shows whether it is required).
- **The session registry, each session's git files and the new end of its transcript are read,
  and the counts for `harbour status` written, on the draw thread**: a Claude config directory,
  a repository or a state directory on slow storage (a network home) delays keys by the time
  those reads take (a few small files and at most 1 MiB of transcript per session), at most
  every 500 ms, by one listing of the projects directory per session still without its
  transcript at most every 5 s, by the time a write takes,
  only when a count changed, and by two file checks or one fresh publication (with its sweep of
  dead ones) at most every 5 s
  (`plans/phase-3-status-dots.md`, `plans/phase-5-tmux-status-line.md`).
- **Remembered sessions are read and written on the draw thread too**: at start every remembered
  file is read once (at most 64 KiB each) and every projects folder listed once, so a few
  thousand files cost seconds once (2000 files over 300 folders took about 2.6 s before the cap
  and the pruning); then at most 100 cards, each a directory check, plus one listing of the
  registry and of the reservations every 500 ms while a card is stopped, and a write of a few
  hundred bytes only when a session's id, title or position changed
  (`plans/phase-10-resumable-sessions.md`).
- **Any signal but TERM, INT and HUP, SIGKILL included, loses what changed since the last read**:
  it skips the read at quit (a first TERM or INT and any HUP do it, as a quit does), so a title or id that changed in the last 500 ms is not remembered,
  and its reservation files stay until the next harbour sweeps them as dead. A new conversation
  is remembered only once its transcript is found, which Claude Code creates at the first message
  and harbour searches for at most every 5 s, so such a signal in the up to 5 s after that first
  message loses the whole new conversation
  (`plans/phase-10-resumable-sessions.md`).
- **Two harbours pressing Enter on one conversation at the same instant can both refuse**: each
  sees the other's lock, retries once after 20 to 80 ms, and about 2 tries in 100 still find it
  held in both (measured in process); pressing Enter again resumes it, never a second copy
  (`plans/phase-10-resumable-sessions.md`).
- **A card held `open elsewhere` by a reused pid cannot be forgotten and returns at every
  start**: a registry file left by a crash whose pid now belongs to another process names its id
  until that process ends, the safe side, never a second copy of a conversation
  (`plans/phase-10-resumable-sessions.md`).
- **At most 100 remembered conversations are shown**: the files beyond the first 100 by
  position are neither shown nor removed while their transcripts exist
  (`plans/phase-10-resumable-sessions.md`).
- **A damaged state directory still shows its cards**: when the remembered folder cannot be made
  private or written, the cards show and Enter refuses with `Could not reserve this
  conversation` (`plans/phase-10-resumable-sessions.md`).
- **A project folder harbour cannot read counts as holding no conversations**: at start, a
  remembered file older than 30 days whose transcript sits in an unreadable folder under the
  Claude Code configuration is removed, though the conversation still exists and Claude Code
  itself still resumes it (`plans/phase-10-resumable-sessions.md`).
- **The release attestation records the `main` commit that started the run, not the release
  commit the tag points at**: its subjects are the right tarballs, but the source commit it
  names is the merge (`.github/AGENTS.md`, "What the attestation proves").
- **The border's and the cards' icons are counted as one column each**: a terminal set to draw
  ambiguous-width characters double-wide pushes the pane's top border out of place with
  `nerd_font` on, and the zoomed top row's names a column into its counts (`src/tabs.rs`), and a card's third line can reach the sidebar's border
  (`src/card.rs`).
- **A card move can undo another harbour's change made in the same instant**: `Memory::place`,
  like `keep` and `forget`, reads a file, then renames a new one over it with nothing guarding the
  gap, so a forget or a title saved by another harbour inside that sub-millisecond window is
  undone (`src/memory.rs`).
- **Ctrl+C twice in the sidebar quits and ends every running session** when nothing is unsaved:
  the owner asked for Ctrl+C to answer the quit question, so a double tap meant for Claude Code
  while the sidebar has the keys quits harbour (`plans/phase-41-ctrl-c-quits.md`).
- **A merge keeps one given title**: when a stopped card is merged into the session that resumed
  its conversation and both have a title the user gave, the session's wins, on the card and in
  that conversation's remembered file (`src/app/remember.rs`).
- **Ctrl held after the prefix on a key that needs Shift works only where the terminal reports
  the shifted character**: `?`, `{` and `}` with Ctrl held reach harbour as that character only
  in some terminals and layouts, and with the default prefix, Ctrl-Shift-/ in the csi-u form
  matches the prefix and sends it to the program; without extended keys a Ctrl key whose control
  byte is another key's acts as that key (C-- and C-7 as C-_ send the default prefix on, C-4 as
  C-\ toggles side by side, C-2 is C-Space, C-3 is Esc and cancels, C-5, C-6 and C-8 are C-],
  C-^ and Backspace, and Ctrl-Shift with a letter is Ctrl with it, so `C-/ C-S-x` ends rather than
  closes); after a width key pressed with Ctrl held, the same Ctrl key within 500 ms repeats it
  instead of reaching the program, which matters only when `wider` or `narrower` is a letter that
  is also a Ctrl shortcut (`plans/phase-46-ctrl-held-after-prefix.md`).
