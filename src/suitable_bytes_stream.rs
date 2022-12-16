use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::utf8_char_source::Utf8CharSource;
use std::io;
use std::io::{Seek, SeekFrom};
use std::mem::take;
use utf8_read::{Char, Reader};

/// Python bytes stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
pub struct SuitableBytesStream {
    // note that this is not actually optional, it's just a shitty hack because I'm too dumb to
    // placate Rust when temporarily moving the reader out of the struct within a method...
    reader: Option<Reader<PyBytesStream>>,
}

impl SuitableBytesStream {
    pub fn new(inner: PyBytesStream) -> Self {
        SuitableBytesStream {
            reader: Some(Reader::new(inner)),
        }
    }
}

impl Utf8CharSource for SuitableBytesStream {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        Ok(
            match self
                .reader
                .as_mut()
                .unwrap()
                .next_char()
                .map_err(|e| (io::Error::new(io::ErrorKind::Other, format!("{}", e))))?
            {
                Char::Eof => None,
                Char::Char(c) => Some(c),
                Char::NoData => {
                    return io::Result::Err(io::Error::new(
                        io::ErrorKind::Other,
                        "should never happen",
                    ));
                }
            },
        )
    }
}

impl ParkCursorChars for SuitableBytesStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        let reader = take(&mut self.reader);
        let (mut inner, _pos, rem_buffered_bytes) = reader.unwrap().complete();
        inner.seek(SeekFrom::Current(-(rem_buffered_bytes.len() as i64)))?;
        // TODO this should be done even if ^ returns an error:
        self.reader = Some(Reader::new(inner));
        Ok(())
    }
}
