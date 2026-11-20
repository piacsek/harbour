use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::editor;

const CHECK_EVERY: Duration = Duration::from_millis(500);

pub type Mover = Box<dyn FnMut(&Path) -> io::Result<()>>;

pub struct Follow {
    mover: Mover,
    at: Option<PathBuf>,
    seen: Option<PathBuf>,
    checked_at: Option<Instant>,
}

impl Follow {
    pub fn new(mover: Mover) -> Follow {
        Follow {
            mover,
            at: None,
            seen: None,
            checked_at: None,
        }
    }

    pub fn to(&mut self, seen: Option<&Path>, home: &Path, user_home: Option<&Path>) {
        if self.seen.as_deref() == seen
            && self.checked_at.is_some_and(|at| at.elapsed() < CHECK_EVERY)
        {
            return;
        }
        self.seen = seen.map(Path::to_path_buf);
        self.checked_at = Some(Instant::now());
        let target = seen
            .map(|seen| editor::root(seen, user_home))
            .filter(|root| root.is_dir())
            .unwrap_or_else(|| home.to_path_buf());
        if !self.enter(&target) {
            self.enter(home);
        }
    }

    fn enter(&mut self, directory: &Path) -> bool {
        if self.at.as_deref() == Some(directory) {
            return true;
        }
        let entered = (self.mover)(directory).is_ok();
        if entered {
            self.at = Some(directory.to_path_buf());
        }
        entered
    }
}
