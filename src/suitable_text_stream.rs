use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom, OpaqueSeekPos};
use crate::park_cursor::ParkCursorChars;
use crate::py_text_stream::PyTextStream;
use crate::read_string::ReadString;
use crate::utf8_char_source::Utf8CharSource;
use owned_chars::{OwnedChars, OwnedCharsExt};
use std::io;
use std::iter::Iterator;

/// Python text stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
pub struct SuitableTextStream {
    inner: PyTextStream,
    chars_iter: OwnedChars,
    chars_read_from_buf: usize,
    buf_start_seek_pos: Option<OpaqueSeekPos>,
}

impl SuitableTextStream {
    pub fn new(inner: PyTextStream) -> Self {
        SuitableTextStream {
            inner,
            chars_iter: OwnedChars::from_string("".to_owned()),
            chars_read_from_buf: 0,
            buf_start_seek_pos: None,
        }
    }
}

impl Utf8CharSource for SuitableTextStream {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        if let Some(c) = self.chars_iter.next() {
            self.chars_read_from_buf += 1;
            Ok(Some(c))
        } else {
            // TODO: I don't think this can handle actually getting to EOF very well (buf size
            // becomes 0? => no seek), but probably not relevant
            self.buf_start_seek_pos = Some(self.inner.seek(OpaqueSeekFrom::Current)?);
            let buf = self.inner.read_string(8000)?; // TODO make configurable
            self.chars_iter = buf.into_chars();
            self.chars_read_from_buf = 0;
            let oc = self.chars_iter.next();
            if let Some(_) = oc {
                self.chars_read_from_buf += 1;
            }
            Ok(oc)
        }
    }
}

impl ParkCursorChars for SuitableTextStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        let chars_read_from_buf = self.chars_read_from_buf;
        if let Some(buf_start_seek_pos) = self.buf_start_seek_pos {
            self.inner.seek(OpaqueSeekFrom::Start(buf_start_seek_pos))?;
            let buf = self.inner.read_string(chars_read_from_buf)?;
            self.chars_iter = buf.into_chars();
        }
        Ok(())
    }
}
