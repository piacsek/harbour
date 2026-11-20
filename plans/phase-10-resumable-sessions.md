# Phase 10: sessions that come back after harbour closes, resumed on demand

## Context

Every session ends with harbour today: quitting, a crash, a closed terminal tab or a reboot
loses the sidebar, and nothing brings it back. Issue #10 asks harbour to remember each session
that has a Claude Code conversation and to show it again, on the next start, as a stopped card
that Enter resumes with `claude --resume <id>` in its directory. Files are written as sessions
change, so a crash loses little; one small file per session means two harbours never rewrite the
same file.

## Verified facts (2026-11-20 UTC, `main` at 5eec30a, Claude Code 2.1.329)

Read from `claude --help` and from the field names and value types of the files on this
machine; no `claude` was started, and nothing below quotes a value.

- `claude --resume <value>` (`-r`) resumes a conversation by session id or name, "or pick from a
  list (optionally with a search term)". So an id Claude Code does not know may open its picker
  with the id as the search term instead of failing; harbour therefore resumes only an id whose
  transcript it finds (see States), and never relies on that path.
- `--fork-session` exists to give a resumed conversation a new id "instead of reusing the
  original": without it, a resumed session keeps its id. `--session-id <uuid>` must be a valid
  UUID.
- A registry file `<config>/sessions/<pid>.json` (`<config>` is `CLAUDE_CONFIG_DIR`, else
  `~/.claude`) holds `pid` (integer), `sessionId` (string, 36 characters, four hyphens: a UUID),
  `cwd` (string), `startedAt`, `updatedAt` and `statusUpdatedAt` (integers), `procStart`
  (string), `status`, and a dozen other fields. `App::read_files` already reads `sessionId` from
  the file named after a session's pid, at most every 500 ms.
- The transcript is `<config>/projects/<folder>/<sessionId>.jsonl`; the folder is derived from
  the directory the session started in, and harbour finds it by id alone (phase 8). The id in the
  registry changes when the user runs `/clear` or `/resume` inside a session (phase 8 follows the
  new transcript for that reason).
- Not verified without starting `claude`: whether `--resume <id>` finds a conversation started
  in another directory. Harbour sidesteps it by resuming in the directory the session started in,
  the one the transcript's folder is derived from.
- The state directory rules exist in `src/summary.rs` (phase 5): an absolute `XDG_STATE_HOME`,
  else an absolute `HOME` joined with `.local/state`, else nothing; folders `0700`, refused when a
  symlink; files `0600`, written to a temporary name and renamed.
- `~/.local/state` exists on this machine; harbour's own folder there holds `status/` only.

## How harbour remembers

- One file per conversation: `<state>/harbour/sessions/<configuration>/<sessionId>.json`, a JSON
  object `{"version":1,"sessionId":…,"directory":…,"title":…,"position":…}`. The name is the id,
  so a conversation is one file however many harbours saw it. No file names another: a file never
  causes another file to be removed.
- `<configuration>` is one folder per Claude Code configuration: 16 hex digits of a stable hash
  (FNV-1a, 64 bits) of the resolved path of its `projects/` folder (symlinks followed, a tail that
  does not exist yet kept as spelled) (`$CLAUDE_CONFIG_DIR/projects`,
  else `~/.claude/projects`). A conversation belongs to the configuration it was made under, so a
  harbour recalls, resumes, reserves and prunes only its own configuration's files and never
  touches another's, even when the two share an id. Files written flat in `sessions/` by earlier
  builds of this branch are ignored: the feature has not shipped, so there is nothing to migrate.
- `directory` is a JSON string when the path is valid UTF-8, else the array of its bytes, so a
  directory whose name is not UTF-8 comes back byte for byte.
- An id is accepted only in Claude Code's shape: lowercase hex in groups of 8, 4, 4, 4 and 12
  joined by hyphens. So no name can start with `-`, and `--resume <id>` never reads one as an
  option.
- A running session is remembered once the registry gives its id and the transcript for that id
  is found. The file is written when the id, title, directory or position differs from what this
  harbour last wrote for it, or when the file is no longer there, checked with the registry read
  (at most every 500 ms). When the id changes, the new file is written, then the old one is
  removed; a crash in between leaves both, and the old one comes back as one more stopped card
  that `x` forgets.
- An exited session writes nothing after its last read: the exit is noticed in one place, which
  reads the registry and transcript one last time, writes, and only then marks the card exited
  and releases its reservation.
- Reconciliation: when a session exits, and at quit, harbour reads its registry and transcript
  one last time and writes what changed, so a title written just before the end is kept. A kill
  by signal skips it: what changed since the last read is lost (a known limit).
- Writing: the folder is made `0700` and refused when it is a symlink; the file is written to a
  fresh `0600` temporary name in the folder and renamed over `<id>.json`, so a reader sees the old
  file or the new one, never half of one. A file of a newer version is never overwritten or
  removed.
- Reservations are the one signal that a harbour runs a conversation: every running session
  with a remembered id holds one. It is taken when the id is first remembered (before resuming,
  for a stopped card), and when the id changes the new one is taken before the old one is
  released. When the new id is held elsewhere or the lock fails, the file is still written, with
  no reservation, and harbour tries again at each read (non-blocking) while the session runs
  and the id stays. The registry stays the
  signal for a Claude Code running outside any harbour.
- Taking one: harbour takes a non-blocking lock on a fresh
  `<id>.<unique>.lock` file (`0600`) in the same folder and holds it while the session runs; a
  second harbour that finds the id locked says `open elsewhere`. Dead lock files are swept when
  the folder is listed. A lock is kept only while its file is still the one at its path (same
  device and inode), so a sweep by another harbour between the create and the lock cannot leave
  a held lock that no listing shows; when the id was held, harbour retries once after 20 to
  80 ms. A reservation that fails with an error refuses the resume and says why. The
  reservation is released when the session exits and at `x`.
- Reading, once at start: every `<id>.json` whose name and `sessionId` agree, whose id has the
  shape above, whose `directory` is absolute and whose `version` is 1; each read is at most
  64 KiB of a regular file, never through a symlink. Anything else is skipped and left on disk,
  except a staged `.<id>.<unique>.tmp` older than a minute: a regular file with harbour's own
  naming, which the sweep removes (never through a symlink).
  The cards are ordered by `position`, then by id; at most the first 100 come back.
- Pruning: a file whose transcript Claude Code no longer has is removed once it is older than
  30 days; a newer one is kept and hidden. The transcripts are found with one listing of every
  `projects/` folder; when `projects/` cannot be read, nothing is recalled and nothing pruned.
- `x` on a stopped card removes its file, unless another card or a running session holds the id.

## What happens to the remembered files and the cards

Invariants:

1. One conversation id is shown by at most one card in a harbour, and owned by at most one
   running session across harbours: the reservation, held by every running session with a
   remembered id, is the single truth for that.
2. A conversation that was running or stopped when harbour last saw it comes back at the next
   start, unless the user forgot it or left it (`/clear` or `/resume` to another id, in any
   harbour).
3. A forgotten or left conversation never comes back, unless it is resumed again.
4. Nothing harbour writes can delete another conversation's file later, nor any file of another
   Claude Code configuration.

| Event | Files after | Cards after | Reservation | Next start shows | Test |
| --- | --- | --- | --- | --- | --- |
| a running session's transcript is first found | `<id>.json` written | the running card | taken for `<id>` before the file is written | a stopped card | `keeping::a_running_session_is_remembered_once_its_transcript_exists`, `switching::a_fresh_session_is_open_elsewhere_in_a_second_harbour_without_its_registry_file` |
| its id changes to a new id | `<new>.json` written, then `<old>.json` removed (except after `/clear` with harbour quit before a message: see below) | the same card, now the new id | the new id taken, then the old one released | one stopped card, the new id | `switching::a_session_whose_id_changes_holds_only_its_new_conversation`, `keeping::a_new_session_id_replaces_the_remembered_one`, `memory::keeping_a_replacement_forgets_the_old_file`, `memory::a_replacement_names_no_other_id` |
| its id changes to one a stopped card holds | `<old>.json` removed, the stopped card's file rewritten by the running card | the stopped card removed, the selection kept on the card that stays | the new id taken, then the old one released | one stopped card | `reconciling::a_new_session_that_resumes_a_stopped_card_merges_the_two`, `switching::merging_away_the_selected_last_card_selects_the_one_that_stays` |
| its id changes to one an exited card holds | as above | the exited card removed, the running one takes its place | the new id taken, then the old one released | one stopped card; the conversation it left comes back too once taken up again | `switching::a_session_that_resumes_an_exited_card_takes_its_place`, `switching::a_conversation_left_and_taken_up_again_comes_back_after_quitting` |
| its id changes to one another running card holds | `<old>.json` removed; the other card's file untouched | both cards stay; the switching card remembers nothing until the other stops running, then adopts the id | the old one released; the other card holds the new one | one stopped card, the other card's | `switching::a_session_that_switches_to_a_conversation_another_card_runs_forgets_the_one_it_left` |
| its id changes to one another harbour holds | `<new>.json` written, `<old>.json` removed | the same card, now the new id | the old one released; none taken while held there, tried again at each read and taken once the other harbour lets go | one stopped card | `switching::a_session_that_switches_to_a_conversation_open_elsewhere_is_remembered_once`, `switching::a_session_that_switched_to_a_conversation_held_elsewhere_takes_it_once_let_go` |
| the session exits | a last read and write, then nothing more | an exited card | released after the last write | a stopped card | `reconciling::a_title_written_just_before_the_session_exits_is_remembered`, `switching::a_resumed_session_that_exits_releases_its_conversation` |
| harbour quits | a last read and write of every running session | none | released with the process | every running and stopped card, stopped | `reconciling::a_title_written_just_before_quitting_is_remembered`, `cards::a_session_comes_back_stopped_after_quitting_and_resumes_with_enter` |
| `x` then `y` on a running card | kept | an exited card | released | a stopped card | `switching::ending_a_running_session_keeps_its_conversation` |
| `x` on an exited card | removed | removed | released before forgetting | nothing | `keeping::x_on_an_exited_remembered_session_forgets_it` |
| `x` on a stopped card | removed | removed | none (a stopped card holds none) | nothing | `remembering::x_forgets_a_stopped_card` |
| `x` on a card held elsewhere | kept (the holder owns it) | hidden here | none here | the card, if the holder is gone by then | `refusing::x_on_a_card_open_elsewhere_hides_it_and_keeps_its_file` |
| Enter on a stopped card | unchanged | a running card on its Claude Code pane | taken; on an error, nothing starts and the footer says `Could not reserve this conversation` | a stopped card | `remembering::enter_resumes_a_stopped_card_in_its_directory`, `switching::resuming_an_exited_card_shows_its_claude_code_pane`, `switching::enter_refuses_when_the_conversation_cannot_be_reserved` |
| harbour killed between any two writes | a staged temporary file, or both the new and the old id, or a lock file whose lock died | as before the kill | a dead lock, swept at the next listing | every good file; a left id as one more stopped card; a staged file is removed by the sweep once older than a minute | `memory::a_file_naming_another_in_replaces_deletes_nothing`, `memory::a_leftover_temporary_file_is_ignored`, `reserving::a_dead_reservation_is_swept_and_not_listed`, `reserving::a_staged_file_left_over_a_minute_is_swept`, `reserving::a_staged_file_being_written_is_not_swept`, `reserving::only_harbours_own_old_staged_files_are_swept` |
| another harbour forgets the id meanwhile | removed; an exited or stopped card here never writes it back | an exited card stays for this run; a stopped one too | none | nothing | `switching::an_exited_card_forgotten_elsewhere_stays_forgotten`, `switching::a_stopped_card_forgotten_elsewhere_does_not_come_back` |
| another harbour resumes the id meanwhile | unchanged | `open elsewhere` here; Enter starts nothing | held there; two harbours at the same instant never both hold it | the card | `reserving::a_conversation_resumed_in_one_harbour_is_open_elsewhere_in_another`, `reserving::two_reservations_of_one_id_at_once_never_both_hold`, `reserving::a_reservation_taken_while_another_harbour_sweeps_stays_listed` |
| another harbour replaces the id meanwhile (its session left it) | removed there | a stopped card here stays for this run | none | nothing | `switching::a_stopped_card_forgotten_elsewhere_does_not_come_back` |
| a session started in a directory whose name is not UTF-8 | `<id>.json` with the directory as an array of bytes | unchanged | as for any running session | a stopped card in that directory | `places::a_directory_that_is_not_utf8_comes_back_byte_for_byte`, `places::a_utf8_directory_is_written_as_text` |
| the same id remembered under two Claude Code configurations | one file in each configuration's folder | each harbour shows only its own configuration's card | per configuration | under each configuration, only its own card | `places::one_id_in_two_configurations_is_two_conversations`, `places::each_claude_code_configuration_remembers_its_own_conversations`, `places::a_configuration_named_the_same_way_twice_is_one_folder`, `cards::a_conversation_comes_back_only_under_the_claude_code_configuration_it_was_made_in` |
| a harbour starts under a configuration whose folder is gone | untouched, stale ones too | no stopped card | none | the cards, once the configuration is back | `refusing::a_stale_card_is_kept_while_its_claude_code_configuration_is_gone` |
| Enter or Ctrl-l on an exited card whose transcript is gone while its editor runs | removed | the card and its editor stay; the footer says the conversation is gone | released already (at exit) | nothing | `switching::enter_on_a_gone_conversation_keeps_the_card_while_its_editor_runs` |
| the file deleted from outside | a running card writes it again at its next read; an exited or stopped card does not | unchanged | unchanged | the card when it was running, else nothing | `reconciling::a_remembered_file_deleted_from_outside_is_written_again`, `switching::an_exited_card_forgotten_elsewhere_stays_forgotten` |
| `/clear`, then harbour quits before another message is sent | `<old>.json` kept: the new id has no transcript yet, so nothing replaces the old one (the exception to "leaving means forgetting"; not fixed, minor) | none | released with the process | a stopped card for the conversation left | none, documented |
| `x` on earlier stopped cards, then a new session starts | the files keep their positions: a stopped card's file is never rewritten, a running card's is its place now | with A, B, C recalled, forgetting A and B and starting D shows C then D | unchanged | D then C: the order can change across a restart (not fixed, minor) | none, documented |
| a project folder under the configuration cannot be read at start | a file older than 30 days whose transcript sits there is removed, as if the conversation were gone | its card not shown | none | nothing, though the conversation still exists and Claude Code itself resumes it (not fixed, known limit) | none, documented |
| Claude Code exits by itself while `End? y/n` is open, then `y` | removed: `y` takes the exited-card path, not the running-card one above | removed | released | nothing (not fixed, minor) | none, documented |
| a session here switches to a conversation another harbour runs, then back, while a stopped `open elsewhere` card here holds that id | kept (the other harbour holds it) | the stopped card merged into the switching one and does not return for this run | none here for that id | the card (not fixed, minor) | none, documented |

## States

| State | What the user sees | Test |
| --- | --- | --- |
| no state folder, or no file | nothing new: the sidebar starts empty, the hint shows | `memory::nothing_is_recalled_without_a_folder` |
| one file, everything in place | a stopped card at its position: `○`, its title, `stopped` on line 2 in dim, its repository and branch on line 3; the pane says `Stopped. Enter resumes it, x forgets it.`; nothing is started | `remembering::a_remembered_session_comes_back_as_a_stopped_card`, `remembering::a_stopped_card_says_stopped_dimmed`, `remembering::the_pane_of_a_stopped_card_says_how_to_resume_it`, `remembering::stopped_cards_come_back_in_their_order` |
| Enter or `Ctrl-l` on a stopped card | `<command> --resume <id>` starts in its directory with no editors, the pane takes focus, the card runs like any other | `remembering::enter_resumes_a_stopped_card_in_its_directory`, `remembering::ctrl_l_resumes_a_stopped_card_and_focuses_its_pane`, `remembering::a_resumed_card_is_no_longer_stopped`, `cards::a_session_comes_back_stopped_after_quitting_and_resumes_with_enter` |
| its directory is gone | line 2 says `directory gone`, the pane says `Its directory is gone. x forgets it.`; Enter starts nothing and the footer says `Not a directory`; `x` forgets it | `refusing::a_stopped_card_whose_directory_is_gone_says_so`, `refusing::enter_on_a_card_whose_directory_is_gone_says_why_and_starts_nothing`, `refusing::the_pane_of_a_card_whose_directory_is_gone_says_x_forgets_it` |
| the transcript is gone, file under 30 days old | the card is not shown; its file stays on disk; the same check runs at Enter, which then drops the card and says so in the footer | `refusing::a_stopped_card_whose_transcript_is_gone_is_not_shown_and_its_file_is_kept`, `refusing::a_stopped_card_whose_transcript_exists_is_shown`, `refusing::enter_on_a_card_whose_transcript_went_away_drops_it_and_says_why` |
| the transcript is gone, file over 30 days old | the card is not shown and its file is removed | `refusing::a_stale_card_whose_transcript_is_gone_is_forgotten`, `memory::an_old_file_is_forgotten_as_stale`, `memory::a_recent_file_is_not_forgotten_as_stale` |
| more than 100 files | the first 100 by position come back; the rest stay on disk | `refusing::at_most_a_hundred_cards_come_back` |
| the id is running elsewhere: a live registry file names it, or another harbour holds its reservation | line 2 says `open elsewhere`, the pane says to close it there; Enter starts no second copy and the footer says it is open elsewhere; `x` hides the card and keeps the file | `refusing::a_stopped_card_open_elsewhere_says_so`, `refusing::enter_on_a_card_open_elsewhere_starts_no_second_copy`, `refusing::x_on_a_card_open_elsewhere_hides_it_and_keeps_its_file`, `refusing::the_pane_of_a_card_open_elsewhere_says_to_close_it_there` |
| the footer said gone or open elsewhere | it goes when another card is selected, at the next attempt, or once the card can resume | `refusing::the_held_message_goes_once_another_card_is_selected`, `refusing::the_held_message_goes_once_the_card_can_be_resumed`, `refusing::the_gone_message_goes_once_another_card_is_selected` |
| the other harbour or terminal ends it | with the next registry read the card says `stopped` again and resumes | `refusing::a_card_open_elsewhere_says_stopped_once_the_other_one_ends`, `refusing::a_registry_file_of_an_ended_process_does_not_hold_a_card`, `reserving::a_conversation_ended_in_one_harbour_resumes_in_another` |
| a registry file left by a crash whose pid now belongs to another process | the card says `open elsewhere` until that process ends, cannot be forgotten and returns at every start (known limit) | none, documented |
| two harbours, same conversation | the one that resumes it holds the reservation; the other shows `open elsewhere` and Enter there starts no second copy | `reserving::a_conversation_resumed_in_one_harbour_is_open_elsewhere_in_another`, `reserving::enter_in_a_second_harbour_starts_no_second_copy` |
| two harbours, different conversations | each resumes its own; neither holds the other | `reserving::two_harbours_resume_different_conversations`, `reserving::another_id_is_reserved_on_its_own` |
| a reservation left by a crash | its lock is gone with the process, so the next listing sweeps it and the id resumes | `reserving::a_dead_reservation_is_swept_and_not_listed`, `reserving::a_released_id_can_be_reserved_again` |
| a resumed session exits (by itself or `x` then `y`) | an exited card, resumable with Enter; `x` then forgets it | `remembering::a_resumed_session_that_exits_resumes_again_with_enter`, `keeping::a_remembered_session_that_exits_resumes_with_enter`, `keeping::x_on_an_exited_remembered_session_forgets_it` |
| resumed twice | the second Enter on a running card focuses the pane; no second process | `remembering::enter_on_a_resumed_card_focuses_it_and_starts_no_second_copy` |
| a resumed session whose conversation left unfinished subagent files from its previous run | the card counts only the subagents started after the resume | `switching::a_resumed_card_counts_only_the_subagents_started_after_it` |
| Enter on a card whose process exited since the last draw | the exit is settled first, so its own reservation never refuses it; it resumes | `switching::enter_right_after_a_session_exits_resumes_it` |
| Enter on an exited card another harbour holds | the footer says it is open elsewhere and stays while the other harbour holds it | `switching::the_held_message_of_an_exited_card_stays_while_it_is_held_elsewhere` |
| a session here switches to the id of a stopped card here | the two merge in the same read; the stopped card never says `open elsewhere` | `switching::a_stopped_card_a_session_here_switches_to_never_says_open_elsewhere` |
| the configuration reached through a symlink, or before its `projects/` folder exists | the same remembered folder: the path is resolved before hashing | `places::a_configuration_reached_through_a_symlink_is_the_same_folder`, `places::a_configuration_is_the_same_folder_before_and_after_its_projects_folder_exists` |
| `/clear` or `/resume` inside the session (its id changes) | the file of the new id is written, the old one removed | `keeping::a_new_session_id_replaces_the_remembered_one`, `memory::a_replacement_names_no_other_id`, `memory::keeping_a_replacement_forgets_the_old_file` |
| a crash between writing the new id and removing the old | both come back as stopped cards; `x` forgets the old one; a file naming another in a `replaces` field (round 1's format) deletes nothing | `memory::a_file_naming_another_in_replaces_deletes_nothing`, `memory::a_file_replacing_a_missing_one_is_recalled` |
| `/resume` inside a new session into a conversation that is a stopped card | the two cards merge into the running one; one file | `reconciling::a_new_session_that_resumes_a_stopped_card_merges_the_two` |
| a title written just before the session exits, or just before quitting | the last title is kept | `reconciling::a_title_written_just_before_the_session_exits_is_remembered`, `reconciling::a_title_written_just_before_quitting_is_remembered` |
| the remembered file deleted from outside while the session runs | written again with the next read | `reconciling::a_remembered_file_deleted_from_outside_is_written_again` |
| harbour killed by a signal, or its tab closed | the files are as of the last read; the next start shows the cards; what changed since is lost (known limit) | `cards::a_session_comes_back_stopped_after_quitting_and_resumes_with_enter` |
| a temporary file left by a crash between write and rename | ignored: not a `<id>.json` name | `memory::a_leftover_temporary_file_is_ignored`, `reserving::a_reservation_is_not_recalled` |
| a file half written by a foreign tool, malformed JSON, a wrong field type, a relative directory, a name that disagrees with its `sessionId`, larger than 64 KiB, a symlink, a folder, unreadable | skipped, never removed; every good file still shows | `memory::malformed_files_are_skipped_and_kept`, `memory::a_wrong_field_type_is_skipped`, `memory::a_relative_directory_is_skipped`, `memory::a_file_whose_name_disagrees_with_its_id_is_skipped`, `memory::an_oversized_file_is_skipped`, `memory::a_symlinked_file_is_skipped`, `memory::a_folder_named_like_a_file_is_skipped`, `memory::an_unreadable_file_is_skipped` |
| an id not in Claude Code's shape, or a file named like an option (`--help.json`) | never kept, never shown, never passed to the command | `memory::an_id_that_is_not_shaped_like_a_claude_code_session_id_is_skipped`, `memory::a_file_named_like_an_option_is_skipped`, `memory::an_id_shaped_like_an_option_is_not_kept`, `refusing::a_remembered_file_named_like_an_option_is_not_shown` |
| a file from a newer harbour (`version` above 1) | skipped, never overwritten or removed | `memory::a_file_from_a_newer_harbour_is_skipped`, `memory::a_file_from_a_newer_harbour_is_never_overwritten`, `memory::a_file_from_a_newer_harbour_is_never_forgotten` |
| `HARBOUR_SESSION_COMMAND` names another program | resuming runs that program with `--resume <id>`; a program that writes no registry file with a `sessionId` and no transcript is never remembered | `remembering::enter_resumes_a_stopped_card_in_its_directory`, `keeping::a_running_session_without_a_transcript_is_not_remembered` |
| no `HOME` and no `XDG_STATE_HOME` | nothing is remembered or read (the state folder rule `harbour status` uses) | `status::without_home_or_xdg_state_home_status_prints_nothing` |
| the state folder is a symlink, or cannot be made | nothing is written; the sessions run as today | `memory::a_symlinked_folder_is_refused` |
| many remembered sessions | the sidebar scrolls to the selected card; `j`/`k` reach every card | `remembering::many_stopped_cards_scroll_to_the_selected_one` |
| narrowest sidebar (60 columns, 18 inside) and narrow layout (under 60) | line 2's `directory gone`, `open elsewhere` and `stopped` are cut by `src/text.rs`; a CJK title fits | `stopping::the_narrowest_sidebar_shows_why_a_card_is_stopped`, `stopping::the_narrowest_sidebar_keeps_a_cjk_title_inside_its_border`, `stopping::a_narrow_terminal_shows_a_stopped_card` |
| a title from disk holding control characters or bidi marks | cleaned and cut by `src/text.rs` like any card | `remembering::a_title_with_control_and_bidi_characters_from_disk_is_cleaned` |
| `q` with a running session | `Quit? y/n` as today, and, when harbour remembers sessions, a dim line above it: `Conversations come back stopped.` | `stopping::q_with_a_running_session_says_its_sessions_come_back_stopped`, `stopping::q_in_a_harbour_that_remembers_nothing_does_not_say_sessions_come_back` |
| `q` with only stopped cards | quits at once, as with only exited sessions today | `stopping::q_with_only_stopped_cards_quits_without_asking` |
| counts in the sidebar title and `harbour status` | a card that never ran in this harbour counts nowhere; an exited one counts as today | `stopping::a_stopped_card_is_counted_nowhere_in_the_title`, `stopping::a_stopped_card_is_not_published_to_harbour_status` |
| the directory picker, the mouse | unchanged; a click in the pane of a card with no process does not focus the pane | `stopping::a_click_in_the_pane_of_a_stopped_card_does_not_focus_the_pane` |

## Keys

| Key | Running card | Exited card | Stopped card |
| --- | --- | --- | --- |
| Enter, `Ctrl-l` | focus the pane | resume it when remembered, else focus the pane (when its transcript is gone, the footer says so and the card goes, or stays while one of its editors runs) | resume it (when its directory is gone or it is open elsewhere, the footer says why and nothing starts) |
| `x` | ask, then end it | remove it and forget it | forget it (only hide it when open elsewhere) |
| `q` | ask, saying sessions come back | quit | quit |

## Decisions

- **The issue's open readings, taken conservatively without the owner**:
  - A card whose transcript is gone is not shown, but its file is kept: dropping files on a
    missing transcript would wipe every memory the day harbour runs with another
    `CLAUDE_CONFIG_DIR`.
  - `x` on a card open elsewhere hides it here and leaves the file, which the running harbour
    owns.
  - Resuming runs in the directory the session started in, not the last `cwd` of its
    transcript, since the transcript's folder is derived from the start directory.
  - A stopped card that never ran in this harbour is not counted in the title or by
    `harbour status`, so two harbours never count one conversation twice.
  - Resuming runs the session command (`claude`, or `HARBOUR_SESSION_COMMAND`) with
    `--resume <id>`, so a scripted program stands in for Claude Code in every test.
- **The code**: `src/memory.rs` reads and writes the files; `src/app/remember.rs`, a child module
  of `src/app.rs`, holds what the app does with them. `src/app.rs` stays where it is with small
  hooks, so the phase 6 branch, which moves it to `src/app/mod.rs`, merges with little to
  resolve.
- **On the draw thread**: the files are read once at start; a write is a few hundred bytes, only
  when something changed; "open elsewhere" lists the registry folder and the reservations once
  per registry read while a card is stopped; the transcripts are found with one listing of every
  `projects/` folder at start, and one search at Enter. A slow file system stalls the interface
  for that long: a known limit.
- **Taken in review round 1, without the owner**:
  - `--session-id` is never passed together with `--resume`: a resumed session keeps its id.
  - Enter or `Ctrl-l` on a stopped card resumes it; on a running card they focus the pane.
  - A resumed session starts with no editors, `seen` is its directory and `followed` is false,
    as a fresh session.
  - Merge: when a session's id becomes one another card holds, a stopped or exited card is
    removed and the running one adopts the file; while the other card still runs, the id is not
    adopted, so two entries never own one file. Forgetting skips an id another card or a running
    session holds.
  - Pruning: a file is removed only when its transcript is missing and it is older than 30 days,
    and only when `projects/` could be read; at most 100 cards come back, by position.
  - A reservation is a lock file per resume, not one per id, so a crash never leaves a file that
    blocks: a dead lock is swept.
- **Taken in review round 2, without the owner**:
  - The `replaces` marker is dropped: any marker that deletes a file at recall can delete a file
    written later for the same id. A kill between writing the new id and removing the old one
    leaves both; the old one comes back as one more stopped card, which breaks no invariant.
  - Leaving a conversation (`/clear` or `/resume` to another id), in any harbour, counts as
    forgetting it, once the new id has a transcript: after `/clear`, quitting before another
    message keeps the conversation left, and it comes back at the next start. A stopped card here whose file another harbour removed stays for this run and
    is gone at the next start.
  - A session that switches to an id another running card holds leaves its old id: the old file
    is removed, the card remembers nothing and drops its reservation, and the other card keeps
    the id. It adopts the id once the other card stops running.
  - A session that switches to an id another harbour holds follows it: one file, one card at
    the next start. Its old id is removed without checking whether another harbour runs it; a
    running holder elsewhere writes the file again at its next read.
  - An exited session writes nothing after its last read, and exactly one place notices an exit.
  - `x` releases the reservation before forgetting; a reservation error refuses the resume.
  - When the id is held, the reservation retries once after 20 to 80 ms of jitter. Two harbours
    pressing Enter on one conversation at the same instant still both refuse about 2 times in
    100 (measured in process); pressing again resumes it (a known limit).
- **Taken in review round 3, without the owner**:
  - One folder of remembered files per Claude Code configuration, named by a stable hash of
    the absolute path of its `projects/` folder. Old flat files are ignored, not migrated: the
    feature has not shipped.
  - Every running session with a remembered id holds its reservation, so the reservation alone
    says a harbour runs a conversation; a new id is taken before the old one is released, and
    an id held elsewhere is remembered without one, not retried until the id changes.
  - Enter on a gone conversation never ends a running editor: the card stays with its editor
    and only the remembered file goes.
  - A directory that is not UTF-8 is stored as its bytes; staged files older than a minute are
    swept like dead locks.
  - Not fixed (minor): only the last folder of the state path is checked for being a symlink; a
    symlink planted higher up in the user's own state directory is followed.
- **Taken in review round 4, without the owner**:
  - A running session without its reservation tries again at each read (non-blocking, and only
    when no listing shows the id held), so it takes the id once the other harbour lets go.
  - A resumed session counts only the subagents started after the resume, as a fresh one does.
  - Exited cards follow whether the id is held, like stopped ones, so the footer's "open
    elsewhere" stays while it is true; Enter settles exits first, so a card's own reservation
    never refuses it.
  - The configuration's path is resolved before hashing (symlinks followed; a tail that does not
    exist yet kept as spelled), so a symlinked `CLAUDE_CONFIG_DIR` finds the same cards.
  - Declined as unreachable: a stopped card saying `open elsewhere` because a session in the same
    harbour switched to its id. The switch and the merge happen in the same read, so no frame
    shows it; a test pins that.
  - Not fixed (known limits): at most 100 remembered conversations are shown, the rest neither
    shown nor removed while their transcripts exist; a remembered folder that cannot be made
    private or written still shows its cards, and Enter refuses with `Could not reserve this
    conversation`.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`, and the phase 8 backfill.
2. `src/memory.rs`: write, read and forget, with every file state of the table; the fragment.
3. Stopped cards at start, resume with Enter and `Ctrl-l`, `x` forgets, the pane's message.
4. Running sessions are remembered and followed (id change, title, position).
5. Directory gone, transcript gone, open elsewhere.
6. Quit line, counts, mouse, narrow layout.
7. The binary wires it in (e2e: quit, start again, resume), help, README, `AGENTS.md`.
8. The demo shows a stopped card, the screenshots are rerun.

## Outcome

### Sweep

Range: phases 6 to 9, read from `scripts/sweep-report.sh 6` and each plan's `## Outcome` and
`## Hurdles`; all four are on `main` (phase 6 merged during this phase, so its hurdles are final).
The builder ran this sweep without the owner, so no `AGENTS.md` changed for it when it was
written; the owner decided every row on 2026-11-20 (the last column, and "Decided 2026-11-20"
below). The report counted 31 hurdles: `review-loop` 11,
`test-seam` 9, `tooling` 5, `process` 3, `ci` 3; eight went `→ none` (phase 6: 4, phase 7: 2,
phase 8: 2).

| # | Finding | Phases | Proposal | Where | Decided 2026-11-20 |
| --- | --- | --- | --- | --- | --- |
| 1 | A test green on its first run, proved only by breaking the code by hand (`wait_for` returns after 5 s without failing; the tui wiring), each `→ none` | 8, and 1 to 3 before | Phase 5's proposal 1 came back: take it. A green first run is not red: break the code it covers, show the red run, restore, say so | root `AGENTS.md`, "TDD is never skipped" | Taken: root `AGENTS.md`, "TDD is never skipped" |
| 2 | Waits in tests that time out without failing: phase 8 made it a rule that every `wait_for` asserts on what it returned; phase 6 still hit a wait that timed out under load (`→ none`) | 8, 6 | Issue: make `wait_for` and `wait_for_all` fail on timeout, naming the text they waited for, so the rule is enforced by the harness, not by review | issue | Filed as issue #46 |
| 3 | Tests that depend on the checkout's folder name: phase 2, phase 8 round 2 (became a rule in `tests/AGENTS.md`), phase 6 again despite the rule | 2, 8, 6 | Issue: a test (like `tests/source_size.rs`) that runs the in-process and e2e suites from a checkout whose folder name differs, or a gate that greps tests for `current_dir`/`CARGO_MANIFEST_DIR` folder names; the rule alone did not hold | issue | Filed as issue #48 |
| 4 | Review rounds that found states of an outside program the plan's table missed: fzf and directories (phase 4), locks and hosts (phase 5), Neovim's states over three of phase 6's rounds | 4, 5, 6 | Phase 5's proposal 8 came back, a third time: take it. A plan's state table lists, for each thing read from outside harbour, what the user sees when it is missing, changed in between, or shared | root `AGENTS.md`, "Plan before code" | Taken: root `AGENTS.md`, "Plan before code" |
| 5 | Width arithmetic in more than one place (phase 8 round 1's emoji, phases 1 and 3 before) | 1, 3, 8 | Phase 5's proposal 4 is now mostly done by code: `src/text.rs` is the one place widths are measured and cut. Narrow the rule to "a plan that draws text adds the narrowest sidebar and a CJK name to its state table"; decline the rest | root `AGENTS.md`, "Plan before code" | Taken as narrowed: root `AGENTS.md`, "Plan before code" |
| 6 | `main` moved mid-phase and conflicted in `PLAN.md` and `plans/metrics.md`; a brief named a wrong test file | 7 | Decline: one occurrence each, and the merge rule already covers catching up | none | Declined, as proposed |
| 7 | Prose through a shell argument blocked by the em dash hook | 2 | Phase 5's proposal 2 did not come back in phases 6 to 9: decline it unless it recurs | none | Declined: not seen since phase 2 |

Metrics (from `plans/metrics.md`): review rounds 5, 6, 2, 4, 1 for phases 5 to 9; start to
release 1 h 50 min (phase 5), 4 h 43 min (phase 6, read from GitHub in this phase), 56 min
(phase 7), 2 h 49 min (phase 8), and 35 min to merge for phase 9, which released nothing. Claude
reviewer tokens per round stay between 97k and 164k (phase 6 round 4: 163360), with phase 8
rising over its rounds (136k, 98k, 154k, 158k); builder notices per phase 10, 11, 3, 7, 2. The rounds track the size of the
change, not a drift; Codex and orchestrator usage are still unknown in every row, so no trend in
total tokens can be read.

**Decided 2026-11-20.** The owner decided this sweep and phase 5's together with the hurdles of
phases 11 to 13, read from `scripts/sweep-report.sh` (which starts after this phase). Two
findings there are new: builds and tests that differed on Linux (phase 12) became a rule in
`tests/AGENTS.md`, a phase that touches sockets or other system calls runs its tests in a Linux
container before its first push; tests that time out or race under a ship's full load (phases
1, 6, 12) became issue #47. The rest repeat rows above: a test green on its first run (phase
12, row 1), states the plan's table missed (phases 11 and 12, row 4), a wait that returns at
its timeout (phase 12, row 2). Declined: putting a sweep to the owner before its phase goes
ready; Codex without network (phase 13; the Codex reviewer was removed in PR #45); a reviewer
running one read-only command in the branch's worktree (phase 13, once); one-off tooling dead
ends. The phase 15 sweep starts after phase 13 with these already handled
(`scripts/sweep-report.sh 14`; the report's default start, after this phase, would count them
again).

### Built

- Tasks 2 and 3 shipped as one commit (`a4c5caa`): `src/memory.rs` alone left `pub` items no
  code called, which the dead code gate refuses, so the memory and the stopped cards that read
  it landed together. Tasks 4 to 8 shipped one commit each (`d60f95e`, `074aeac`, `6105c12`,
  `f77aa95`, `4276a8d`).
- harbour remembers each running Claude Code session in `<state>/harbour/sessions/<id>.json`
  (its id, start directory, title and place in the sidebar) and follows the id when Claude Code
  changes it. On the next start the sessions come back as stopped cards in their remembered
  order: a dim `stopped` line, no process, counted nowhere. Enter or `Ctrl-l` resumes one with
  `--resume <id>` in its start directory; `x` forgets it.
- A stopped card says why it cannot resume, in every layout, the narrowest sidebar included:
  `directory gone`, `open elsewhere` (its id is in a live registry file), or, at Enter, a footer
  line when the conversation is gone (the card is hidden, the file kept). A failed start shows
  the refusal like any other session.
- The quit question (asked while a session runs) adds `Conversations come back stopped.` on
  its last row.
- The help names resuming and forgetting (`KEYS`, the README table) and its legend says `○` is
  also stopped; it still fits 80x24. The demo shows a stopped `acme-docs` card, and every
  screenshot was rerun (two runs byte-identical).
- Further decisions taken without the owner, beside those under Decisions: the quit line shows
  only when harbour remembers sessions (never in tests that wire no memory); a directory gone
  refuses with the same `Not a directory` reason a fresh start gives.
- Two known limits joined the root `AGENTS.md`: the remembered files are read and written on
  the draw thread, and a stale registry file whose pid was reused keeps a card `open elsewhere`
  until that process ends.
- `main` (phase 6's `src/app/` split and the editor column, phase 8's sidebar lines) was merged
  in before review round 1 (`5bff725`).
- Review round 1's fixes, shipped as one commit: a last read and write when a session exits and
  at quit; `replaces` in the new file and the replaced file dropped at start; a `/resume` into a
  remembered conversation merges the two cards; a per-resume reservation lock that a second
  harbour sees as `open elsewhere`; ids only in Claude Code's lowercase UUID shape; the pane text
  follows the card's reason; the gone and open elsewhere footers clear on selection, at the next
  attempt and once the card can resume; a missing file is written again; stale files pruned after
  30 days, at most 100 cards, one listing of `projects/`; the quit question has its own screenshot
  (`docs/quit.png`). Three known limits joined or were rewritten: the start-up cost, a kill by
  signal, and the reused-pid card that cannot be forgotten.
- Review round 2's fixes, shipped as one commit, after the table "What happens to the remembered
  files and the cards": the `replaces` marker dropped, so no file deletes another; a reservation
  kept only while its lock file is still at its path, retried once with jitter, its errors
  refusing the resume, released at exit and at `x`; an exited card never writes its file again,
  and only the settle notices an exit (the draw no longer marks it); the selection follows the
  card that stays after a merge; a resume opens on the Claude Code pane; a switch to an id
  another running card holds leaves the old id. Every row of the table names its tests.
- Review round 3's fixes, shipped as one commit: one folder of remembered files per Claude Code
  configuration, so a harbour under another `CLAUDE_CONFIG_DIR` neither shows nor prunes them; a
  reservation held by every running session with a remembered id, taken when the id is first
  known and moved when it changes; Enter on a gone conversation keeps a card whose editor still
  runs; a directory that is not UTF-8 stored as bytes; staged files left by a kill swept after a
  minute.
- Review round 4's fixes, shipped as one commit: a running session without its reservation takes
  it once the other harbour lets go; a resumed card counts only its own subagents; the "open
  elsewhere" footer of an exited card stays while true; Enter right after an exit resumes; a
  symlinked configuration finds its cards; the e2e tests of this phase assert every wait.
- Review round 5 found no must and no should: the reviewer who drove harbour live found none for
  the second round running. Its five items below should are recorded as rows of the table and in
  the PR body, and one as a known limit in the root `AGENTS.md`.

### Review

Five rounds, each run by fresh, unprimed reviewers (a Claude reviewer and a Codex reviewer); the
last was clean.

| Round | What it found |
| --- | --- |
| 1 | a title lost at exit and quit; two cards after a crash between two writes; a `/resume` into a remembered conversation leaving two owners of one file; two harbours both resuming before a registry file existed; an id shaped like an option; stale footer and pane wording, a deleted file never rewritten, an unbounded start-up search |
| 2 | the `replaces` marker deleting a file written later; a lock swept between its creation and its lock; an exited card writing back a file forgotten elsewhere; the selection lost after a merge; a resume opening on the editor; a switch to an id another running card holds leaving two owners |
| 3 | one configuration pruning another's files; a fresh session without a reservation resumable by a second harbour; Enter on a gone conversation killing a running editor; a directory that is not UTF-8 never remembered; staged files kept forever |
| 4 | a resumed card counting its previous run's subagents; a reservation never retried; an exited card's `open elsewhere` footer and an Enter right after an exit reading the wrong state; a symlinked configuration; e2e waits that dropped their result |
| 5 | nothing at must or should; five items below should, recorded as table rows |

What removed a class of finding instead of patching it:

- the table "What happens to the remembered files and the cards", with its invariants and a test
  named in every row, written after round 2's second strike on the same mechanism;
- dropping the `replaces` marker: no file names another, so nothing harbour writes deletes a file
  written later;
- the reservation as the one truth for "running in a harbour": every running session with a
  remembered id holds it, so the registry is no longer needed to tell;
- one folder per Claude Code configuration, so no harbour prunes or shows another
  configuration's conversations.

### Decisions taken without the owner

For the owner to judge, in one place (the details are under Decisions):

- leaving a conversation with `/clear` or `/resume` forgets it (except after `/clear` with harbour
  quit before another message);
- a file whose transcript is gone is pruned after 30 days; at most 100 cards come back;
- Enter and `Ctrl-l` both resume a stopped card;
- stopped cards count nowhere: not in the sidebar title, not in `harbour status`;
- a card whose transcript is gone is hidden and its file kept; `x` on a card open elsewhere only
  hides it;
- resuming runs in the directory the session started in, with the session command and
  `--resume <id>`;
- the quit line shows only when harbour remembers sessions;
- one folder per Claude Code configuration, its path resolved before hashing; old flat files are
  ignored, not migrated;
- the sweep's seven proposals above wait for the owner's yes.

## Hurdles

- `process` Tasks 2 and 3 of the plan could not ship apart: the memory module alone was dead code to `tests/dead_code.rs` → none
- `test-seam` Several tests were green on their first run (the tui wiring, the quit line, the CJK cut, the pane focus); each was proved by breaking the code by hand, seeing it red and restoring it → none
- `tooling` A ship log in the shared scratchpad was overwritten by another agent's run, so the builder could not read its own result; a unique log name fixed it → none
- `tooling` A shell command that read stdin by mistake hung a background wait until it was stopped by hand → none
- `test-seam` An e2e assertion on the title counts raced the demo's first frame, which shows the cards before their sessions report; waiting for the counts fixed it → code
- `review-loop` Round 1 found that the id was written only on change, so a title written just before an exit or quit was lost; the session now gets a last read when it exits and at quit → code
- `review-loop` Round 1 found that a crash between writing a new id and removing the old one left two cards for one conversation; the new file names the id it replaces → code
- `review-loop` Round 1 found that a `/resume` into a remembered conversation left two entries owning one file, and forgetting one deleted the other's; the cards now merge → code
- `review-loop` Round 1 found that two harbours could both resume a conversation before either's registry file was written; a reservation lock closes the window → code
- `review-loop` Round 1 found that a file name starting with `-` would reach `--resume` as an option; ids are now only Claude Code's shape → code
- `review-loop` Round 1 found the footer and pane wording stale after the card's reason changed, a deleted file never rewritten, and an unbounded start-up search; each got a state row and a test → code
- `review-loop` Round 1 found the plan's sweep counts left out phase 6 and its Test cells named files, not tests; every cell now names a test, checked by a script against the test sources → none
- `test-seam` Four tests of the round were green on their first run (the forged option name, already refused by the memory check; the bidi title; the scroll; resumed twice): they are credits of the states table, covering code that already held → none
- `review-loop` Round 2 found that the `replaces` marker could delete a file written later for the same id; the marker is dropped → code
- `review-loop` Round 2 found that a sweep by another harbour between creating a lock file and locking it left a held lock no listing showed, and that two harbours pressing Enter at once both refused about 40% of the time; the lock is checked against its path and retried once → code
- `review-loop` Round 2 found that an exited card wrote back a file forgotten elsewhere, and that the draw marked exits before the last read; one place notices an exit and exited cards write nothing → code
- `review-loop` Round 2 found the selection lost after a merge, a resume opening on the editor, and a switch to an id another running card holds leaving two owners; each got a row and a test → code
- `design` A second round of holes in the same mechanism (the remembered files) triggered "two strikes": a table of every event and its files, cards and reservation replaced patching, and the marker was dropped instead of hardened → code
- `test-seam` Five tests of round 2 were green on their first run: two harbours reserving at once is a credit of the invariant, and the other four (a resumed exit releasing, a switch open elsewhere, ending keeps, a stopped card forgotten elsewhere) were proved by breaking the code they cover and seeing them red → none
- `review-loop` Round 3 found that a harbour under another `CLAUDE_CONFIG_DIR` pruned the remembered files of the first one, whose transcripts it could not see; files now live in one folder per configuration → code
- `review-loop` Round 3 found that only a resumed session held a reservation, so a fresh session whose registry file was gone could be resumed by a second harbour; every running session with a remembered id now holds one → code
- `review-loop` Round 3 found that Enter on a gone conversation removed the card and killed its running editor without asking; the card now stays while an editor runs → code
- `review-loop` Round 3 found that a directory whose name is not UTF-8 was never remembered, and that staged files left by a kill stayed forever; directories are stored as bytes and old staged files swept → code
- `test-seam` Two tests of round 3 were green on their first run: the e2e configuration test (a card whose transcript is missing is hidden anyway, so it had to age the file and give the other configuration a `projects/` folder before it went red), and a configuration whose folder is gone, proved by breaking the recall and seeing it red → none
- `review-loop` Round 4 found that a resumed card counted subagents its previous run left unfinished, that a running session refused its reservation never tried again, and that an exited card's "open elsewhere" footer and an Enter right after an exit read the wrong state; each got a row and a test → code
- `test-seam` The first fix for a symlinked configuration resolved only an existing `projects/` folder, so the folder changed once Claude Code created it and every card vanished; the e2e resume test caught it, and a unit test now pins a path resolved before and after it exists → code
- `test-seam` The round 4 test for a stopped card switched to in the same harbour was green on its first run: the merge happens in the same read, so the finding was unreachable; breaking the merge turned it red (`filter(|_| false)` on the other card) → none
- `test-seam` The e2e tests this phase added dropped the screen of six waits, so a launch that crashed could pass; each wait now asserts what it returned, as the rule in `tests/AGENTS.md` already asks → code
- `review-loop` Round 5 found no must or should; its five items below should (`/clear` then quit, card order after forgetting, an unreadable project folder, an exit while `End? y/n` is open, a switch away and back) became rows of the table, one a known limit → known limit
