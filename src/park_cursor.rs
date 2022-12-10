use crate::opaque_seek::OpaqueSeek;
use crate::read_rewind::{ReadRewind, RewindReader};
use crate::utf8_char_source::Utf8CharSource;
use replace_with::replace_with_or_abort;
use std::io;
use std::io::{BufReader, Read};
use utf8_chars::BufReadCharsExt;

/// A UTF8 source that allows parking the cursor of the underlying stream.
///
/// By parking, we mean placing the cursor after the last read character,
/// regardless of any read-ahead performed by the buffering mechanism.
///
/// Note that you MUST NOT read from the BufReader afterwards because it could
/// be nonsense (I haven't thought about what happens b/c I don't need it).
pub trait ParkCursorChars: Utf8CharSource {
    fn park_cursor(&mut self) -> io::Result<()>;
}

pub struct ParkCursorBufReader<R: Read + OpaqueSeek> {
    inner: BufReader<RewindReader<R>>,
    chars_since_last_read: Option<u64>,
}

impl<R: Read + OpaqueSeek> ParkCursorBufReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner: BufReader::with_capacity(200, RewindReader::new(inner)),
            chars_since_last_read: None,
        }
    }
}

impl<R: Read + OpaqueSeek> Utf8CharSource for ParkCursorBufReader<R> {
    fn read_char(&mut self) -> io::Result<Option<char>> {
        let c = self.inner.read_char_raw()?;
        self.chars_since_last_read = match self.chars_since_last_read {
            None => Some(1),
            Some(x) => Some(x + 1),
        };
        Ok(c)
    }
}

impl<R: Read + OpaqueSeek> ParkCursorChars for ParkCursorBufReader<R> {
    fn park_cursor(&mut self) -> io::Result<()> {
        let chars_between_last_read_and_cur = match self.chars_since_last_read
        {
            None => {
                return Ok(());
            }
            Some(x) => x,
        };
        let mut unbuffered_rewind_reader = self.inner.get_mut();
        unbuffered_rewind_reader.rewind_read()?;
        println!("chars to re-read: {}", chars_between_last_read_and_cur);
        // XXX This is an extreme hack that ONLY works for pyo3-file patched
        // with my bufsize/4 modification!
        // https://github.com/omerbenamram/pyo3-file/pull/7
        // In any other case, you can't know that reading 4 bytes means reading
        // exactly one char!
        // The proper solution here would be to have an Utf8CharSource that
        // reads individual chars directly from Python streams instead of going
        // the utf8 -> bytes -> utf8 route. => TODO Write PR for pyo3-file.
        let mut one_char_per_read_reader =
            BufReader::with_capacity(4, unbuffered_rewind_reader);
        for _ in 0..chars_between_last_read_and_cur {
            one_char_per_read_reader.read_char_raw()?;
        }
        // Recreate inner BufReader to avoid inconsistent state issues
        replace_with_or_abort(
            &mut self.inner,
            |inner: BufReader<RewindReader<R>>| {
                BufReader::new(inner.into_inner())
            },
        );
        Ok(())
    }
}
