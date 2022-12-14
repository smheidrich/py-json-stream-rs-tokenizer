use crate::utf8_char_source::Utf8CharSource;
use std::io;

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
