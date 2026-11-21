use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Default)]
pub struct Shape {
    sent: Arc<AtomicU8>,
}

impl Shape {
    pub fn send(&mut self, out: &mut impl io::Write, wanted: u8) -> io::Result<()> {
        if wanted == self.sent.load(Ordering::SeqCst) {
            return Ok(());
        }
        write!(out, "\x1b[?25l\x1b[{wanted} q")?;
        out.flush()?;
        self.sent.store(wanted, Ordering::SeqCst);
        Ok(())
    }

    pub fn leave(&self, out: &mut impl io::Write) -> io::Result<()> {
        if self.sent.load(Ordering::SeqCst) == 0 {
            return Ok(());
        }
        write!(out, "\x1b[0 q")
    }
}
