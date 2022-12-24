use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::remainder::{Remainder, StreamData};
use crate::utf8_char_source::Utf8CharSource;
use std::io;
use std::io::Read;
use utf8_width::get_width;

/// Python bytes stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
///
/// This is the variant for unseekable streams. Chars are read in from Python one-by-one, which is
/// very slow but prevents readahead buffering.
pub struct SuitableUnbufferedBytesStream {
    inner: PyBytesStream,
}

impl SuitableUnbufferedBytesStream {
    pub fn new(inner: PyBytesStream) -> Self {
        SuitableUnbufferedBytesStream { inner }
    }
}

impl Utf8CharSource for SuitableUnbufferedBytesStream {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        let mut buf: [u8; 4] = [0; 4];
        let n_bytes_read = self.inner.read(&mut buf[..1])?;
        if n_bytes_read < 1 {
            // EOF
            return Ok(None);
        }
        if n_bytes_read > 1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "broken stream: returns more bytes than requested",
            ));
        }
        let n_bytes_in_char = get_width(buf[0]);
        if n_bytes_in_char == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("invalid UTF-8 start byte: {:x}", buf[0]),
            ));
        }
        let n_bytes_actual = {
            if n_bytes_in_char > 1 {
                // this should only return fewer bytes than requested if it's cut short by EOF
                // => will evaluate to invalid UTF-8 at the end and return an error
                self.inner.read(&mut buf[1..n_bytes_in_char])? + 1
            } else {
                1
            }
        };
        Ok(std::str::from_utf8(&buf[..n_bytes_actual])
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?
            .chars()
            .next())
    }
}

impl ParkCursorChars for SuitableUnbufferedBytesStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl Remainder for SuitableUnbufferedBytesStream {
    fn remainder(&self) -> StreamData {
        StreamData::Bytes(vec![0; 0])
    }
}
