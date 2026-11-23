use ratatui::layout::{Constraint, Layout, Rect};

use super::App;

const NARROW: u16 = 60;
const NARROWEST: u16 = 15;

impl App {
    pub(super) fn areas(&self, area: Rect) -> (Rect, Rect) {
        if let Some([claude, _]) = self.halves(area) {
            (Rect::default(), claude)
        } else if self.zoomed || self.side_by_side {
            (Rect::default(), area)
        } else if area.width >= NARROW {
            two_columns(area, self.sidebar_width)
        } else {
            self.one_side(area)
        }
    }

    pub(super) fn resize(&mut self, wider: bool) -> bool {
        if self.sidebar.is_empty() || self.shown_pane.is_empty() {
            return false;
        }
        let drawn = self.sidebar.width;
        let widest = self.screen.width / 2;
        let width = match wider {
            true if drawn < widest => (drawn + 2).min(widest),
            false if drawn > NARROWEST => (drawn - 2).max(NARROWEST),
            _ => return false,
        };
        self.sidebar_width = Some(width);
        true
    }

    fn one_side(&self, area: Rect) -> (Rect, Rect) {
        if self.pane_focused {
            (Rect::default(), area)
        } else {
            (area, Rect::default())
        }
    }
}

fn two_columns(area: Rect, width: Option<u16>) -> (Rect, Rect) {
    let sidebar = match width {
        Some(width) => Constraint::Length(width.min(area.width / 2)),
        None => Constraint::Ratio(1, 3),
    };
    let [sidebar, pane] = Layout::horizontal([sidebar, Constraint::Fill(1)]).areas(area);
    (sidebar, pane)
}
