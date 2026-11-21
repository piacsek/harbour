use std::io;

#[derive(Default)]
pub struct Shape {
    sent: u8,
}

impl Shape {
    pub fn send(&mut self, out: &mut impl io::Write, wanted: u8) -> io::Result<()> {
        if wanted == self.sent {
            return Ok(());
        }
        write!(out, "\x1b[{wanted} q")?;
        out.flush()?;
        self.sent = wanted;
        Ok(())
    }
}
