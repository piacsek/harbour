use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use super::App;
use crate::keymap::{Action, After, key_name, name, typed};

const REPEAT: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(super) struct Prefixing {
    waiting: bool,
    repeat: Option<(bool, Instant)>,
    closed: Option<Action>,
}

impl App {
    pub(super) fn prefix_key(&mut self, key: KeyEvent) -> bool {
        if !self.prefixing.waiting {
            if self.repeated(key) {
                return true;
            }
            self.prefixing.repeat = None;
            let unescaped = KeyEvent {
                modifiers: key.modifiers - KeyModifiers::ALT,
                ..key
            };
            if !self.keymap.is_prefix(unescaped) {
                return false;
            }
            if key.modifiers.contains(KeyModifiers::ALT) {
                self.forward(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
            }
            self.prefixing.waiting = true;
            self.log_prefix("");
            return true;
        }
        self.prefixing.waiting = false;
        let closed = self.prefixing.closed.take();
        let after = self.keymap.after(key);
        self.log_prefix(&logged(after));
        match after {
            After::Act(action) if closed == Some(action) => {}
            After::Act(action) => self.act(action),
            After::Jump(position) => self.jump_to(position),
            After::Prefix => {
                self.forward(self.keymap.prefix_key());
            }
            After::Cancel => {}
            After::Unknown => {
                let prefix = self.keymap.prefix_name();
                let key = key_name(key);
                self.fail_unlogged(match self.keymap.key_of(Action::Help) {
                    Some(help) => {
                        format!("{prefix} {key} does nothing; {prefix} {help} lists the keys")
                    }
                    None => format!("{prefix} {key} does nothing"),
                });
            }
        }
        true
    }

    pub(super) fn prefix_closes_view(&mut self, key: KeyEvent) -> bool {
        let closed = match (self.helping, &self.reading) {
            (true, _) => Action::Help,
            (false, Some(_)) => Action::Changelog,
            (false, None) => return false,
        };
        if !self.keymap.is_prefix(key) {
            return false;
        }
        self.helping = false;
        self.close_changelog();
        self.prefixing.waiting = true;
        self.prefixing.closed = Some(closed);
        self.log_prefix("");
        true
    }

    pub(super) fn prefix_waiting(&self) -> bool {
        self.prefixing.waiting
    }

    pub(super) fn waiting_mark(&self) -> Option<Span<'static>> {
        self.prefixing.waiting.then(|| {
            Span::styled(
                self.keymap.prefix_name(),
                Style::new().fg(self.accent).add_modifier(Modifier::BOLD),
            )
        })
    }

    pub(super) fn bordered_mark(&self) -> Option<Line<'static>> {
        let mark = self.waiting_mark()?;
        Some(Line::from(vec![Span::from(" "), mark, Span::from(" ")]).right_aligned())
    }

    pub(super) fn help_hint(&self) -> Option<String> {
        let key = self.keymap.key_of(Action::Help)?;
        Some(format!("{} {key} help", self.keymap.prefix_name()))
    }

    pub(super) fn bordered_hint(&self, room: u16) -> Option<Line<'static>> {
        let hint = format!(" {} ", self.help_hint()?);
        (hint.width() <= usize::from(room))
            .then(|| Line::styled(hint, Modifier::DIM).right_aligned())
    }

    pub(super) fn cancel_prefix(&mut self) {
        self.prefixing.waiting = false;
        self.prefixing.repeat = None;
        self.prefixing.closed = None;
    }

    fn repeated(&mut self, key: KeyEvent) -> bool {
        let Some((wider, at)) = self.prefixing.repeat else {
            return false;
        };
        let KeyCode::Char(c) = key.code else {
            return false;
        };
        let action = if wider {
            Action::Wider
        } else {
            Action::Narrower
        };
        if !(key.modifiers - KeyModifiers::SHIFT - KeyModifiers::CONTROL).is_empty()
            || at.elapsed() > REPEAT
            || self.keymap.action_on(typed(key, c)) != Some(action)
        {
            return false;
        }
        self.log_prefix(&format!("key={}", name(action)));
        self.resize_again(wider);
        true
    }

    fn resize_again(&mut self, wider: bool) {
        self.prefixing.repeat = self.resize(wider).then(|| (wider, Instant::now()));
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::Zoom => self.zoom(),
            Action::Side => self.toggle_side(),
            Action::SwapLeft | Action::SwapRight if self.side_by_side => {
                self.swapped = action == Action::SwapRight;
            }
            Action::SwapLeft | Action::SwapRight => {}
            Action::Find if !self.sessions.is_empty() => self.open_finder(),
            Action::Find => {}
            Action::Rename => self.open_rename(),
            Action::New => self.pick_or_start(),
            Action::End => self.end_or_remove(),
            Action::Close => self.close_or_remove(),
            Action::Editor => self.next_editor(),
            Action::Sidebar => self.pane_focused = false,
            Action::Pane => self.focus_or_resume(),
            Action::Narrower => self.resize_again(false),
            Action::Wider => self.resize_again(true),
            Action::Usage => self.open_usage(),
            Action::Changelog => self.show_changelog(),
            Action::Help => self.helping = true,
            Action::Quit => self.quit(),
        }
    }
}

fn logged(after: After) -> String {
    match after {
        After::Act(action) => format!("key={}", name(action)),
        After::Jump(position) => format!("key={position}"),
        After::Prefix => "key=prefix".to_string(),
        After::Cancel => "cancelled".to_string(),
        After::Unknown => "key=other".to_string(),
    }
}
