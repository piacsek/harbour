use ratatui::layout::{Constraint, Layout, Rect};

use super::App;

const NARROW: u16 = 60;

impl App {
    pub(super) fn areas(&self, area: Rect) -> (Rect, Rect) {
        if area.width >= NARROW {
            two_columns(area)
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

fn two_columns(area: Rect) -> (Rect, Rect) {
    let [sidebar, pane] =
        Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(area);
    (sidebar, pane)
}
