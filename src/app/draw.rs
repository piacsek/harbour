use portable_pty::ExitStatus;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Clear, HighlightSpacing, List, ListState, Paragraph, Wrap,
};
use unicode_width::UnicodeWidthStr;

use super::{App, HINT, NARROW, Question};
use crate::editor::Changes;
use crate::help;
use crate::state;
use crate::tabs;

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        self.finish_pick();
        self.finish_editors();
        self.read_registry();
        self.screen = frame.area();
        let (sidebar, pane) = self.areas(frame.area());
        self.sidebar = sidebar;
        let notes: Vec<Option<String>> = self
            .sessions
            .iter_mut()
            .map(|entry| entry.session.exit().map(exit_note))
            .collect();
        let counts = state::counts(
            self.sessions
                .iter()
                .zip(&notes)
                .map(|(entry, note)| (note.is_some(), entry.record.as_ref())),
        );
        if let Some(summary) = &mut self.summary {
            summary.publish(counts);
        }
        let title = state::title(counts, usize::from(sidebar.width.saturating_sub(2)));
        let sidebar_block = bordered(!self.pane_focused).title(title);
        let below_rows = sidebar_block.inner(sidebar);
        let mut pane_block = bordered(self.pane_focused);
        if let Some(tabs) = self.tabs(pane.width) {
            pane_block = pane_block.title(tabs);
        }
        self.pane = pane_block.inner(if pane.is_empty() { frame.area() } else { pane });
        self.shown_pane = if pane.is_empty() {
            Rect::default()
        } else {
            self.pane
        };
        for entry in &mut self.sessions {
            entry.session.resize(self.pane.height, self.pane.width);
            for editor in &mut entry.editors {
                editor.session.resize(self.pane.height, self.pane.width);
            }
        }
        let rows: Vec<Line> = self
            .sessions
            .iter()
            .zip(notes)
            .map(|(entry, note)| {
                state::row(
                    &entry.name,
                    note,
                    entry.record.as_ref(),
                    usize::from(below_rows.width.saturating_sub(2)),
                )
            })
            .collect();
        let exited = self
            .sessions
            .get_mut(self.selected)
            .is_some_and(|entry| !entry.on_editor && entry.session.exit().is_some());
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
                let list = List::new(rows)
                    .block(sidebar_block)
                    .highlight_symbol(Line::from(vec![
                        Span::from("▌").fg(Color::Cyan),
                        Span::from(" "),
                    ]))
                    .highlight_spacing(HighlightSpacing::Always)
                    .highlight_style(Modifier::BOLD);
                let mut state = ListState::default().with_selected(Some(self.selected));
                frame.render_stateful_widget(list, sidebar, &mut state);
                if !pane.is_empty() {
                    frame.render_widget(pane_block, pane);
                    frame.render_widget(shown.shown(), self.pane);
                    if exited {
                        frame.buffer_mut().set_style(self.pane, Modifier::DIM);
                    }
                }
            }
        }
        let mut free = below_rows;
        if let Some(question) = self.asking {
            free = draw_bottom(frame, question_lines(question, below_rows.width), free);
        }
        if let Some(failed) = &self.failed {
            draw_bottom(frame, failure_lines(failed, free.width), free);
        }
        if self.helping {
            self.draw_help(frame);
        }
        if let Some(pick) = &mut self.picking {
            let area = popup(frame.area());
            let block = bordered(true).title(" directory ");
            let inner = block.inner(area);
            self.picking_area = inner;
            pick.resize(inner.height, inner.width);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            frame.render_widget(pick.session(), inner);
        }
    }

    fn tabs(&self, width: u16) -> Option<Line<'static>> {
        let program = self.editor.as_ref()?;
        let entry = self.sessions.get(self.selected)?;
        let directory = entry.editor().map(|editor| editor.label.as_str());
        let [claude, editor] = tabs::strip(
            program.name(),
            directory,
            usize::from(width.saturating_sub(2)),
        );
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
        let notes = help::NOTES.iter().map(|line| Line::from(*line));
        let lines: Vec<Line> = keys
            .chain([Line::default()])
            .chain(state::legend())
            .chain(notes)
            .collect();
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(" help ")),
            frame.area(),
        );
    }

    fn areas(&self, area: Rect) -> (Rect, Rect) {
        if area.width >= NARROW {
            let [sidebar, pane] =
                Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(area);
            (sidebar, pane)
        } else if self.pane_focused {
            (Rect::default(), area)
        } else {
            (area, Rect::default())
        }
    }
}

fn failure_lines(failed: &str, width: u16) -> Vec<Line<'static>> {
    let chars: Vec<char> = failed.chars().collect();
    chars
        .chunks(usize::from(width.max(1)))
        .map(|chunk| Line::from(chunk.iter().collect::<String>()))
        .collect()
}

fn question_lines(question: Question, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let mut lines: Vec<String> = Vec::new();
    for word in asked(question).split(' ') {
        match lines.last_mut() {
            Some(line) if line.width() + 1 + word.width() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
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

fn bordered(focused: bool) -> Block<'static> {
    if focused {
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Color::Cyan)
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
