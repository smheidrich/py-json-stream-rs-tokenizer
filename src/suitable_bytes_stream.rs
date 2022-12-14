use crate::py_bytes_stream::PyBytesStream;

/// Python bytes stream wrapper that makes it "suitable" for use in the Tokenizer.
///
/// This means that the necessary traits (see below) are implemented for it.
pub struct SuitableBytesStream {
    inner: PyBytesStream,
    // TODO
}

impl SuitableBytesStream {
    pub fn new(inner: PyBytesStream) -> Self {
        SuitableBytesStream {
            inner,
        }
    }
}

// TODO:

//impl Utf8CharSource for SuitableBytesStream {
    //fn read_char(&mut self) -> io::Result<Option<char>> {
    //}
//}

//impl ParkCursorChars for SuitableBytesStream {
    //fn park_cursor(&mut self) -> io::Result<()> {
    //}
//}
