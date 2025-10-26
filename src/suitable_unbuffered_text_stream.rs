use crate::park_cursor::ParkCursorChars;
use crate::py_text_stream::PyTextStream;
use crate::read_string::ReadString;
use crate::remainder::{Remainder, StreamData};
use crate::utf8_char_source::Utf8CharSource;
use std::io;

/// Python text stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
///
/// This is the variant for unseekable streams. Chars are read in from Python one-by-one, which is
/// very slow but prevents readahead buffering.
pub struct SuitableUnbufferedTextStream {
    inner: PyTextStream,
}

impl SuitableUnbufferedTextStream {
    pub fn new(inner: PyTextStream) -> Self {
        SuitableUnbufferedTextStream { inner }
    }
}

impl Utf8CharSource for SuitableUnbufferedTextStream {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        let s = self.inner.read_string(1)?;
        if s.is_empty() {
            Ok(None)
        } else {
            let mut it = s.chars();
            let c = it.next();
            if it.next().is_some() {
                Err(io::Error::other(
                    "got more than 1 char from read_string(1), which should never happen...",
                ))
            } else {
                Ok(c)
            }
        }
    }
}

impl ParkCursorChars for SuitableUnbufferedTextStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        // no-op
        Ok(())
    }
}

impl Remainder for SuitableUnbufferedTextStream {
    fn remainder(&self) -> StreamData {
        StreamData::Text(String::from(""))
    }
}
