use pyo3::types::PyAny;
use pyo3::{Bound, IntoPyObject, PyErr, Python};
use unwrap_infallible::UnwrapInfallible;

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

impl<'py> IntoPyObject<'py> for StreamData {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(match self {
            StreamData::Text(s) => s.into_pyobject(py).unwrap_infallible().into_any(),
            StreamData::Bytes(b) => b.as_slice().into_pyobject(py)?,
        })
    }
}
