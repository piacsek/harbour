use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::editor::{self, EditorDir};

const CHECK_EVERY: Duration = Duration::from_millis(500);

pub fn temporary() -> PathBuf {
    let temporary = std::env::temp_dir();
    std::path::absolute(&temporary).unwrap_or(temporary)
}

pub type Mover = Box<dyn FnMut(&Path) -> io::Result<()>>;

pub struct Follow {
    mover: Mover,
    at: Option<(PathBuf, Option<(u64, u64)>)>,
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

    pub fn to(
        &mut self,
        seen: Option<&Path>,
        home: &Path,
        user_home: Option<&Path>,
        editor_dir: EditorDir,
    ) {
        if self.seen.as_deref() == seen
            && self.checked_at.is_some_and(|at| at.elapsed() < CHECK_EVERY)
        {
            return;
        }
        self.seen = seen.map(Path::to_path_buf);
        self.checked_at = Some(Instant::now());
        let target = seen
            .map(|seen| editor::directory(seen, user_home, editor_dir))
            .filter(|root| root.is_dir())
            .unwrap_or_else(|| home.to_path_buf());
        if !self.enter(&target) {
            self.enter(home);
        }
    }

    fn enter(&mut self, directory: &Path) -> bool {
        let identity = identity(directory);
        if self.at.as_ref() == Some(&(directory.to_path_buf(), identity)) {
            return true;
        }
        let entered = (self.mover)(directory).is_ok();
        if entered {
            self.at = Some((directory.to_path_buf(), identity));
        }
        entered
    }
}

fn identity(directory: &Path) -> Option<(u64, u64)> {
    fs::metadata(directory)
        .ok()
        .map(|meta| (meta.dev(), meta.ino()))
}
