use pyo3::{IntoPy, PyObject, Python};

pub enum StreamData {
    Text(String),
    Bytes(Vec<u8>),
}

/// An UTF-8 char source that can return its buffered but unprocessed chars.
///
/// This is an alternative to ParkCursorChars for underlying streams that don't support seeking,
/// although implementors of ParkCursorChars can also implement Remainder additionally.
pub trait Remainder {
    /// Return the current remainder as a Python string or bytes.
    ///
    /// Which type it is depends on the type of the underlying stream. Can be an empty string or
    /// bytes if there is no remainder.
    fn remainder(&self) -> StreamData;
}

impl IntoPy<PyObject> for StreamData {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            StreamData::Text(s) => s.into_py(py),
            StreamData::Bytes(b) => b.into_py(py),
        }
    }
}
