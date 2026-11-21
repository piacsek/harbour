use std::time::Instant;

use portable_pty::ExitStatus;
use ratatui::Frame;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};

use super::{App, HINT, Question, remember, user_home};
use crate::card::{Card, Ended};
use crate::editor::{Changes, Program};
use crate::help;
use crate::session::Session;
use crate::state;
use crate::tabs;
use crate::text;
use crate::transcript::Transcript;

impl App {
    fn follow_selected(&mut self) {
        if let Some(follow) = &mut self.follow {
            let seen = self
                .sessions
                .get(self.selected)
                .map(|entry| entry.seen.as_path());
            follow.to(seen, &self.home, user_home().as_deref());
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let began = Instant::now();
        self.compose(frame);
        self.log_frame(began);
    }

    fn compose(&mut self, frame: &mut Frame) {
        self.log_start(frame.area());
        self.finish_pick();
        self.finish_editors();
        self.read_files();
        self.settle_exited();
        self.serve();
        self.follow_selected();
        self.screen = frame.area();
        self.log_changes();
        self.settle_side();
        self.settle_zoom();
        let (sidebar, pane) = self.areas(frame.area());
        self.sidebar = sidebar;
        let notes: Vec<Option<Ended>> = self
            .sessions
            .iter_mut()
            .map(|entry| match &mut entry.session {
                None => Some(Ended::Stopped(entry.why)),
                Some(_) if entry.exited_at.is_none() => None,
                Some(session) => session
                    .exit()
                    .map(|status| Ended::Exited(exit_note(status))),
            })
            .collect();
        let counts = state::counts(
            self.sessions
                .iter()
                .zip(&notes)
                .filter(|(entry, _)| entry.session.is_some())
                .map(|(entry, note)| (note.is_some(), entry.record.as_ref())),
        );
        if let Some(summary) = &mut self.summary {
            summary.publish(counts);
        }
        let title = state::title(counts, usize::from(sidebar.width.saturating_sub(2)));
        let sidebar_block = bordered(!self.pane_focused, self.accent).title(title);
        let below_rows = sidebar_block.inner(sidebar);
        let beside = self
            .halves(frame.area())
            .map_or(Rect::default(), |[_, editor]| editor);
        let on_editor = self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.on_editor);
        let split = !beside.is_empty();
        let left_focused = self.pane_focused && !(split && on_editor);
        let mut beside_block = bordered(self.pane_focused && on_editor, self.accent);
        self.beside = beside_block.inner(beside);
        let mut pane_block = bordered(left_focused, self.accent);
        if split {
            [pane_block, beside_block] = self.side_borders(
                [pane_block, beside_block],
                [pane.width, beside.width],
                counts,
            );
        } else if self.zoomed || self.side_by_side {
            pane_block = self.zoom_border(pane_block, pane.width, counts);
        } else if let Some(tabs) = self.tabs(usize::from(pane.width.saturating_sub(2))) {
            pane_block = pane_block.title(tabs);
        }
        self.pane = pane_block.inner(if pane.is_empty() { frame.area() } else { pane });
        self.shown_pane = if pane.is_empty() {
            Rect::default()
        } else {
            self.pane
        };
        let editing = if self.beside.is_empty() {
            self.pane
        } else {
            self.beside
        };
        for entry in &mut self.sessions {
            if let Some(session) = &mut entry.session {
                session.resize(self.pane.height, self.pane.width);
            }
            for editor in &mut entry.editors {
                editor.session.resize(editing.height, editing.width);
            }
        }
        let neovim = self.editor.as_ref().is_some_and(Program::is_neovim);
        let cards: Vec<Card> = self
            .sessions
            .iter()
            .zip(notes)
            .map(|(entry, note)| {
                let card = Card::new(
                    &entry.name,
                    note,
                    entry.record.as_ref(),
                    &entry.place,
                    entry.elapsed(),
                    entry.transcript.as_ref().map(Transcript::facts),
                );
                if self.nerd_font {
                    card.with_icons(neovim && !entry.editors.is_empty())
                } else {
                    card
                }
            })
            .collect();
        let exited = self.sessions.get_mut(self.selected).is_some_and(|entry| {
            !entry.on_editor && entry.session.is_some() && entry.exited_at.is_some()
        });
        let mut cursor = (None, 0);
        match self.sessions.get(self.selected) {
            None => {
                let message = self.failed.as_deref().unwrap_or(HINT);
                frame.render_widget(
                    Paragraph::new(HINT)
                        .wrap(Wrap { trim: false })
                        .block(sidebar_block),
                    sidebar,
                );
                frame.render_widget(
                    Paragraph::new(message)
                        .wrap(Wrap { trim: false })
                        .block(pane_block),
                    pane,
                );
            }
            Some(shown) => {
                frame.render_widget(sidebar_block, sidebar);
                self.cards.draw(
                    &cards,
                    self.selected,
                    self.accent,
                    below_rows,
                    frame.buffer_mut(),
                );
                if !pane.is_empty() {
                    let left = if split {
                        shown.session.as_ref()
                    } else {
                        shown.shown()
                    };
                    match left {
                        Some(session) => {
                            frame.render_widget(pane_block, pane);
                            frame.render_widget(session, self.pane);
                            cursor = placed(session, self.pane);
                            cursor.0 = cursor.0.filter(|_| left_focused);
                        }
                        None => frame.render_widget(
                            Paragraph::new(remember::pane(shown.why))
                                .wrap(Wrap { trim: false })
                                .block(pane_block),
                            pane,
                        ),
                    }
                    if exited {
                        frame.buffer_mut().set_style(self.pane, Modifier::DIM);
                        cursor = (None, 0);
                    }
                }
                if let Some(editor) = shown.editor().filter(|_| split) {
                    frame.render_widget(beside_block, beside);
                    frame.render_widget(&editor.session, self.beside);
                    if self.pane_focused && on_editor {
                        cursor = placed(&editor.session, self.beside);
                    }
                }
            }
        }
        let mut free = below_rows;
        if let Some(question) = self.asking {
            let mut lines = question_lines(question, below_rows.width);
            if matches!(question, Question::Quit(_)) {
                lines.splice(0..0, self.quit_note(below_rows.width));
            }
            free = draw_bottom(frame, lines, free);
        }
        if let Some(failed) = &self.failed {
            let area = if sidebar.is_empty() && !self.sessions.is_empty() {
                self.pane
            } else {
                free
            };
            let above = draw_bottom(frame, failure_lines(failed, area.width), area);
            if cursor
                .0
                .is_some_and(|at| area.contains(at) && !above.contains(at))
            {
                cursor.0 = None;
            }
        }
        if self.helping {
            self.draw_help(frame);
            cursor = (None, 0);
        }
        if self.draw_changelog(frame) {
            cursor = (None, 0);
        }
        if let Some(pick) = &mut self.picking {
            let area = popup(frame.area());
            let title = if self.finding.is_some() {
                " sessions "
            } else {
                " directory "
            };
            let block = bordered(true, self.accent).title(title);
            let inner = block.inner(area);
            self.picking_area = inner;
            pick.resize(inner.height, inner.width);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            frame.render_widget(pick.session(), inner);
            cursor = placed(pick.session(), inner);
        }
        if self.draw_usage(frame) {
            cursor = (None, 0);
        }
        (self.cursor, self.shape) = cursor;
    }

    pub(super) fn tabs(&self, room: usize) -> Option<Line<'static>> {
        let program = self.editor.as_ref()?;
        let entry = self.sessions.get(self.selected)?;
        let directory = entry.editor().map(|editor| editor.label.as_str());
        let [claude, editor] = if self.nerd_font {
            tabs::iconed(program.name(), program.is_neovim(), directory, room)
        } else {
            tabs::strip(program.name(), directory, room)
        };
        if claude.is_empty() {
            return None;
        }
        let (claude_style, editor_style) = if entry.on_editor {
            (Modifier::DIM, Modifier::BOLD)
        } else {
            (Modifier::BOLD, Modifier::DIM)
        };
        Some(Line::from(vec![
            Span::from(" "),
            Span::styled(claude, claude_style),
            Span::from(" | "),
            Span::styled(editor, editor_style),
            Span::from(" "),
        ]))
    }

    fn draw_help(&self, frame: &mut Frame) {
        let keys = help::KEYS
            .iter()
            .map(|(key, what)| Line::from(format!("{key:<10} {what}")));
        let lines: Vec<Line> = keys.chain(state::legend()).collect();
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(" help ")),
            frame.area(),
        );
    }
}

fn placed(session: &Session, area: Rect) -> (Option<Position>, u8) {
    (session.cursor(area), session.cursor_shape())
}

fn failure_lines(failed: &str, width: u16) -> Vec<Line<'static>> {
    text::wrap(failed, usize::from(width))
        .into_iter()
        .map(Line::from)
        .collect()
}

fn question_lines(question: Question, width: u16) -> Vec<Line<'static>> {
    text::wrap(&asked(question), usize::from(width))
        .into_iter()
        .map(|line| Line::from(line).bold())
        .collect()
}

fn draw_bottom(frame: &mut Frame, lines: Vec<Line<'static>>, area: Rect) -> Rect {
    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .min(area.height);
    let bottom = Rect::new(area.x, area.bottom() - height, area.width, height);
    frame.render_widget(Clear, bottom);
    frame.render_widget(Paragraph::new(lines), bottom);
    Rect {
        height: area.height - height,
        ..area
    }
}

pub(super) fn exit_note(status: &ExitStatus) -> String {
    match (status.signal(), status.exit_code()) {
        (Some(signal), _) => format!(" ({signal})"),
        (None, 0) => String::new(),
        (None, code) => format!(" {code}"),
    }
}

pub(super) fn bordered(focused: bool, accent: Color) -> Block<'static> {
    if focused {
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(accent)
    } else {
        Block::bordered().border_type(BorderType::Plain)
    }
}

pub(super) fn popup(area: Rect) -> Rect {
    area.centered(Constraint::Percentage(80), Constraint::Percentage(80))
}

fn asked(question: Question) -> String {
    let (changes, what) = match question {
        Question::End(changes) => (changes, "End"),
        Question::Quit(changes) => (changes, "Quit"),
    };
    let lead = match changes {
        Changes::Saved => "",
        Changes::Unknown => "May have unsaved changes. ",
        Changes::Unsaved => "Unsaved changes. ",
    };
    format!("{lead}{what}? y/n")
}
