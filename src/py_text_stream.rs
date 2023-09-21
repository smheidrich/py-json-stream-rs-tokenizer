use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom, OpaqueSeekPos};
use crate::py_common::PySeekWhence;
use crate::py_err::TracebackDisplay;
use crate::read_string::ReadString;
use pyo3::{PyObject, PyResult, Python};
use std::io;

/// Python file-like object (= stream) that outputs text.
pub struct PyTextStream {
    inner: PyObject,
}

impl PyTextStream {
    pub fn new(inner: PyObject) -> Self {
        PyTextStream { inner }
    }
}

impl ReadString for PyTextStream {
    // TODO Find out if there is a way to transfer this string in a zero-copy way from Py to Rs.
    // But I guess there can't be because for that we'd have to KNOW that it will never be read
    // again in Python (so the lifetime can be entirely in our hands), which we can't because there
    // is no way to annotate such facts in Python.
    fn read_string(&mut self, size: usize) -> io::Result<String> {
        Python::with_gil(|py| -> PyResult<String> {
            self.inner
                .as_ref(py)
                .call_method1("read", (size,))?
                .extract::<String>()
        })
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Error reading up to {} bytes from Python text stream: {}\n{}",
                    size,
                    e,
                    e.traceback_display(),
                ),
            )
        })
    }
}

impl OpaqueSeek for PyTextStream {
    fn seek(&mut self, pos: OpaqueSeekFrom) -> io::Result<OpaqueSeekPos> {
        let (offset, whence) = match pos {
            OpaqueSeekFrom::Start(x) => (x.0, PySeekWhence::Set),
            OpaqueSeekFrom::Current => (0, PySeekWhence::Cur),
            OpaqueSeekFrom::End => (0, PySeekWhence::End),
        };
        Python::with_gil(|py| -> PyResult<u64> {
            self.inner
                .as_ref(py)
                .call_method1("seek", (offset, whence))?
                .extract::<u64>()
        })
        .map(|x| OpaqueSeekPos(x))
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Error seeking to offset {} (from {:?}) in Python text stream: {}\n{}",
                    offset,
                    whence,
                    e,
                    e.traceback_display(),
                ),
            )
        })
    }
}
