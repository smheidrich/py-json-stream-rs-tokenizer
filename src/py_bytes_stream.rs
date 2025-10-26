use crate::py_common::PySeekWhence;
use crate::py_err::TracebackDisplay;
use pyo3::{PyObject, PyResult, Python};
use std::io;
use std::io::{Read, Seek, SeekFrom};

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
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let vec = Python::with_gil(|py| -> PyResult<Vec<u8>> {
            self.inner
                .as_ref(py)
                .call_method1("read", (buf.len(),))?
                .extract::<Vec<u8>>()
        })
        .map_err(|e| {
            io::Error::other(
                format!(
                    "Error reading up to {} bytes from Python bytes stream: {}\n{}",
                    buf.len(),
                    e,
                    e.traceback_display(),
                ),
            )
        })?;
        buf[..vec.len()].clone_from_slice(&vec);
        Ok(vec.len())
    }
}

impl Seek for PyBytesStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(x) => (x as i64, PySeekWhence::Set),
            SeekFrom::Current(x) => (x, PySeekWhence::Cur),
            SeekFrom::End(x) => (x, PySeekWhence::End),
        };
        Python::with_gil(|py| -> PyResult<u64> {
            self.inner
                .as_ref(py)
                .call_method1("seek", (offset, whence))?
                .extract::<u64>()
        })
        .map_err(|e| {
            io::Error::other(
                format!(
                    "Error seeking to offset {} (from {:?}) in Python bytes stream: {}\n{}",
                    offset,
                    whence,
                    e,
                    e.traceback_display(),
                ),
            )
        })
    }
}
