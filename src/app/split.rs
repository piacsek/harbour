use ratatui::layout::{Constraint, Layout, Rect};

use super::App;

const NARROW: u16 = 60;

impl App {
    pub(super) fn areas(&self, area: Rect) -> (Rect, Rect) {
        if self.zoomed {
            (Rect::default(), area)
        } else if area.width >= NARROW {
            two_columns(area, self.sidebar_width)
        } else {
            self.one_side(area)
        }
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
