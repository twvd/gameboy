use std::io;
use std::sync::mpsc;

/// Wraps a mpsc::Sender<u8> to make it
/// implement the std::io::Write trait.
pub struct WritableSender {
    sender: mpsc::Sender<u8>,
}

impl WritableSender {
    pub fn new(sender: mpsc::Sender<u8>) -> Self {
        Self { sender }
    }
}

impl io::Write for WritableSender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &b in buf {
            self.sender
                .send(b)
                .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
