use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom};
use crate::py_common::PySeekWhence;
use crate::py_err::TracebackDisplay;
use crate::read_string::ReadString;
use pyo3::{IntoPy, PyObject, PyResult, Python};
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

/// It is an error to do arithmetic on this number.
#[derive(Clone, Debug)]
pub struct PyOpaqueSeekPos(pub PyObject);

impl IntoPy<PyObject> for PyOpaqueSeekPos {
    fn into_py(self, _py: Python<'_>) -> PyObject {
        self.0
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
    type OpaqueSeekPos = PyOpaqueSeekPos;

    fn seek(&mut self, pos: OpaqueSeekFrom<PyOpaqueSeekPos>) -> io::Result<PyOpaqueSeekPos> {
        Python::with_gil(|py| {
            let (offset, whence) = match pos {
                OpaqueSeekFrom::Start(x) => (x, PySeekWhence::Set),
                OpaqueSeekFrom::Current => {
                    (PyOpaqueSeekPos((0 as u8).into_py(py)), PySeekWhence::Cur)
                }
                OpaqueSeekFrom::End => (PyOpaqueSeekPos((0 as u8).into_py(py)), PySeekWhence::End),
            };
            match self
                .inner
                .as_ref(py)
                .call_method1("seek", (offset.clone(), whence))
            {
                Ok(x) => Ok(PyOpaqueSeekPos(x.into_py(py))),
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Error seeking to offset {:?} (from {:?}) in Python text stream: {}\n{}",
                        offset,
                        whence,
                        e,
                        e.traceback_display(),
                    ),
                )),
            }
        })
    }
}
