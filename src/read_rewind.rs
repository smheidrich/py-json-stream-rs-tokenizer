use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom, OpaqueSeekPos};
use std::io;
use std::io::Read;

// trait

/// Stream that can be rewound back to its position before the last read().
pub trait ReadRewind: Read {
    /// Rewind the stream's cursor back to its position before the last read().
    fn rewind_read(&mut self) -> Result<(), io::Error>;
}

// impls

pub struct RewindReader<R: Read + OpaqueSeek> {
    inner: R,
    last_pos: Option<OpaqueSeekPos>,
}

impl<R: Read + OpaqueSeek> RewindReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            last_pos: None,
        }
    }
}

impl<R: Read + OpaqueSeek> Read for RewindReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.last_pos = Some(self.inner.seek(OpaqueSeekFrom::Current)?);
        self.inner.read(buf)
    }
}

impl<R: Read + OpaqueSeek> ReadRewind for RewindReader<R> {
    fn rewind_read(&mut self) -> Result<(), io::Error> {
        match self.last_pos {
            None => {}
            Some(x) => {
                self.inner.seek(OpaqueSeekFrom::Start(x))?;
            }
        };
        Ok(())
    }
}
