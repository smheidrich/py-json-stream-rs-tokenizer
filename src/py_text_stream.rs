use crate::opaque_seek::{OpaqueSeek, OpaqueSeekFrom};
use crate::py_common::PySeekWhence;
use crate::py_err::TracebackDisplay;
use crate::read_string::ReadString;
use pyo3::types::{PyAny, PyAnyMethods};
use pyo3::{IntoPyObject, Py, PyObject, PyResult, Python};
use std::io;
use unwrap_infallible::UnwrapInfallible;

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
#[derive(Clone, Debug, IntoPyObject)]
pub struct PyOpaqueSeekPos(pub Py<PyAny>);

impl ReadString for PyTextStream {
    // TODO Find out if there is a way to transfer this string in a zero-copy way from Py to Rs.
    // But I guess there can't be because for that we'd have to KNOW that it will never be read
    // again in Python (so the lifetime can be entirely in our hands), which we can't because there
    // is no way to annotate such facts in Python.
    fn read_string(&mut self, size: usize) -> io::Result<String> {
        Python::with_gil(|py| -> PyResult<String> {
            self.inner
                .bind(py)
                .call_method1("read", (size,))?
                .extract::<String>()
        })
        .map_err(|e| {
            io::Error::other(format!(
                "Error reading up to {} bytes from Python text stream: {}\n{}",
                size,
                e,
                e.traceback_display(),
            ))
        })
    }
}

impl OpaqueSeek for PyTextStream {
    type OpaqueSeekPos = PyOpaqueSeekPos;

    fn seek(&mut self, pos: OpaqueSeekFrom<PyOpaqueSeekPos>) -> io::Result<PyOpaqueSeekPos> {
        Python::with_gil(|py| {
            let (offset, whence) = match pos {
                OpaqueSeekFrom::Start(x) => (x, PySeekWhence::Set),
                OpaqueSeekFrom::Current => (
                    PyOpaqueSeekPos(
                        0_u8.into_pyobject(py)
                            .unwrap_infallible()
                            .unbind()
                            .into_any(),
                    ),
                    PySeekWhence::Cur,
                ),
                OpaqueSeekFrom::End => (
                    PyOpaqueSeekPos(
                        0_u8.into_pyobject(py)
                            .unwrap_infallible()
                            .unbind()
                            .into_any(),
                    ),
                    PySeekWhence::End,
                ),
            };
            match self
                .inner
                .bind(py)
                .call_method1("seek", (offset.clone(), whence))
            {
                Ok(x) => Ok(PyOpaqueSeekPos(
                    x.into_pyobject(py).unwrap_infallible().unbind(),
                )),
                Err(e) => Err(io::Error::other(format!(
                    "Error seeking to offset {:?} (from {:?}) in Python text stream: {}\n{}",
                    offset,
                    whence,
                    e,
                    e.traceback_display(),
                ))),
            }
        })
    }
}
