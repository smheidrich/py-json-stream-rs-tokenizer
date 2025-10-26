use crate::park_cursor::ParkCursorChars;
use crate::py_text_stream::PyTextStream;
use crate::read_string::ReadString;
use crate::remainder::{Remainder, StreamData};
use crate::utf8_char_source::Utf8CharSource;
use owned_chars::{OwnedChars, OwnedCharsExt};
use std::io;
use std::iter::Iterator;

/// Python unseekable text stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
pub struct SuitableUnseekableBufferedTextStream {
    inner: PyTextStream,
    buffer_size: usize,
    chars_iter: OwnedChars,
    chars_read_from_buf: usize,
}

impl SuitableUnseekableBufferedTextStream {
    pub fn new(inner: PyTextStream, buffer_size: usize) -> Self {
        SuitableUnseekableBufferedTextStream {
            inner,
            buffer_size,
            chars_iter: OwnedChars::from_string("".to_owned()),
            chars_read_from_buf: 0,
        }
    }
}

impl Utf8CharSource for SuitableUnseekableBufferedTextStream {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        if let Some(c) = self.chars_iter.next() {
            self.chars_read_from_buf += 1;
            Ok(Some(c))
        } else {
            let buf = self.inner.read_string(self.buffer_size)?;
            self.chars_iter = OwnedCharsExt::into_chars(buf);
            self.chars_read_from_buf = 0;
            let oc = self.chars_iter.next();
            if oc.is_some() {
                self.chars_read_from_buf += 1;
            }
            Ok(oc)
        }
    }
}

impl ParkCursorChars for SuitableUnseekableBufferedTextStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl Remainder for SuitableUnseekableBufferedTextStream {
    fn remainder(&self) -> StreamData {
        StreamData::Text(String::from(self.chars_iter.as_str()))
    }
}
