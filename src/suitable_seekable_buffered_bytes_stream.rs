use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::remainder::{Remainder, StreamData};
use crate::utf8_char_source::Utf8CharSource;
use std::io;
use std::io::{Seek, SeekFrom};
use std::mem::take;
use utf8_read::{Char, Reader};

/// Python bytes stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
pub struct SuitableSeekableBufferedBytesStream {
    // note that this is not actually optional, it's just a shitty hack because I'm too dumb to
    // placate Rust when temporarily moving the reader out of the struct within a method...
    reader: Option<Reader<PyBytesStream>>,
}

impl SuitableSeekableBufferedBytesStream {
    pub fn new(inner: PyBytesStream, bufsize: usize) -> Self {
        SuitableSeekableBufferedBytesStream {
            reader: Some(Reader::with_chunk_size(inner, bufsize)),
        }
    }
}

impl Utf8CharSource for SuitableSeekableBufferedBytesStream {
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
                Char::NoData => None, // for us this means the same as EOF I guess?
            },
        )
    }
}

impl ParkCursorChars for SuitableSeekableBufferedBytesStream {
    fn park_cursor(&mut self) -> io::Result<()> {
        let reader = take(&mut self.reader);
        let (mut inner, _pos, rem_buffered_bytes) = reader.unwrap().complete();
        inner.seek(SeekFrom::Current(-(rem_buffered_bytes.len() as i64)))?;
        // TODO this should be done even if ^ returns an error:
        self.reader = Some(Reader::new(inner));
        Ok(())
    }
}

impl Remainder for SuitableSeekableBufferedBytesStream {
    fn remainder(&self) -> StreamData {
        StreamData::Bytes(match &self.reader {
            Some(reader) => reader.borrow_buffer().to_owned(),
            None => vec![0; 0],
        })
    }
}
