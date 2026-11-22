use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use super::App;
use crate::keymap::{Action, After, key_name, name};

const REPEAT: Duration = Duration::from_millis(500);

#[derive(Default)]
pub(super) struct Prefixing {
    waiting: bool,
    repeat: Option<(bool, Instant)>,
}

impl App {
    pub(super) fn prefix_key(&mut self, key: KeyEvent) -> bool {
        if !self.prefixing.waiting {
            if self.repeated(key) {
                return true;
            }
            self.prefixing.repeat = None;
            if !self.keymap.is_prefix(key) {
                return false;
            }
            self.prefixing.waiting = true;
            self.log_prefix("");
            return true;
        }
        self.prefixing.waiting = false;
        let after = self.keymap.after(key);
        self.log_prefix(&logged(after));
        match after {
            After::Act(action) => self.act(action),
            After::Jump(position) => self.jump_to(position),
            After::Prefix => {
                self.forward(self.keymap.prefix_key());
            }
            After::Cancel => {}
            After::Unknown => {
                self.fail_unlogged(format!(
                    "{prefix} {key} does nothing; {prefix} ? lists the keys",
                    prefix = self.keymap.prefix_name(),
                    key = key_name(key),
                ));
            }
        }
        true
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
        if !(key.modifiers - KeyModifiers::SHIFT).is_empty()
            || at.elapsed() > REPEAT
            || self.keymap.action_on(c) != Some(action)
        {
            return false;
        }
        self.log_prefix(&format!("key={}", name(action)));
        self.resize_again(wider);
        true
    }

    fn resize_again(&mut self, wider: bool) {
        self.resize(wider);
        self.prefixing.repeat = Some((wider, Instant::now()));
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
            Action::New => self.pick_or_start(),
            Action::End => self.end_or_remove(),
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
