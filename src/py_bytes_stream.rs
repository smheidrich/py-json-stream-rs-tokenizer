use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom, OpaqueSeekPos};
use crate::py_common::PySeekWhence;
use pyo3::{PyObject, PyResult};

/// Python file-like object (= stream) that outputs bytes.
pub struct PyBytesStream {
    inner: PyObject,
}

impl PyBytesStream {
    pub fn new(inner: PyObject) -> Self {
        PyBytesStream { inner }
    }
}

impl Read for PyBytesStream {
    // TODO Find out if there is a way to transfer this bytes vec in a zero-copy way from Py to Rs.
    // But I guess there can't be because for that we'd have to KNOW that it will never be read
    // again in Python (so the lifetime can be entirely in our hands), which we can't because there
    // is no way to annotate such facts in Python.
    pub fn read(&mut self, &mut buf: [u8]) -> Result<usize> {
        Python::with_gil(|py| -> PyResult<Vec<u8>> { self.inner.call_method1("read", (size,)) })?
    }
}

impl Seek for PyBytesStream {
    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(x) => (x, 0),
            SeekFrom::Current(x) => (x, 1),
            SeekFrom::End(x) => (x, 2),
        };
        Python::with_gil(|py| -> PyResult<u64> {
            self.inner.call_method1("seek", (offset, whence))
        })
    }
}
