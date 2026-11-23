use super::App;

impl App {
    pub(super) fn jump(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }
        let side = self.side_by_side;
        self.select(index);
        self.focus_or_resume();
        if side {
            self.stay_side();
        }
    }

    pub(super) fn jump_to(&mut self, position: usize) {
        let index = match position {
            9 => self.sessions.len().saturating_sub(1),
            position => position - 1,
        };
        self.jump(index);
    }
}
