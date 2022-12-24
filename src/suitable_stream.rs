/// Utility for dynamically constructing an appropriate "suitable" stream from a raw Python stream.
///
/// "Suitable" means that it implements ParkCursorChars (too lazy to make another trait for that),
/// although it can be a no-op.
use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::py_text_stream::PyTextStream;
use crate::remainder::Remainder;
use crate::suitable_seekable_bytes_stream::SuitableSeekableBytesStream;
use crate::suitable_seekable_text_stream::SuitableSeekableTextStream;
use crate::suitable_unseekable_bytes_stream::SuitableUnseekableBytesStream;
use crate::suitable_unseekable_text_stream::SuitableUnseekableTextStream;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBytes, PyString};
use pyo3::{PyObject, PyResult, Python};

pub trait SuitableStream: ParkCursorChars + Remainder {}

impl<T: ParkCursorChars + Remainder> SuitableStream for T {}

enum ReadReturnType {
    String,
    Bytes,
    Other(String),
}

pub fn make_suitable_stream(stream: PyObject) -> PyResult<Box<dyn SuitableStream + Send>> {
    let read_return_type: ReadReturnType = Python::with_gil(|py| -> PyResult<ReadReturnType> {
        let read_result = stream.as_ref(py).call_method1("read", (0,))?;
        if read_result.is_instance_of::<PyString>()? {
            Ok(ReadReturnType::String)
        } else if read_result.is_instance_of::<PyBytes>()? {
            Ok(ReadReturnType::Bytes)
        } else {
            Ok(ReadReturnType::Other(format!("{}", read_result.get_type())))
        }
    })?;
    let seekable: bool = Python::with_gil(|py| -> PyResult<bool> {
        Ok(stream
            .as_ref(py)
            .call_method1("seekable", ())?
            .extract::<bool>()?)
    })?;
    match read_return_type {
        ReadReturnType::String => {
            let py_text_stream = PyTextStream::new(stream);
            Ok(if seekable {
                Box::new(SuitableSeekableTextStream::new(py_text_stream))
            } else {
                Box::new(SuitableUnseekableTextStream::new(py_text_stream))
            })
        }
        ReadReturnType::Bytes => {
            let py_bytes_stream = PyBytesStream::new(stream);
            Ok(if seekable {
                Box::new(SuitableSeekableBytesStream::new(py_bytes_stream))
            } else {
                Box::new(SuitableUnseekableBytesStream::new(py_bytes_stream))
            })
        }
        ReadReturnType::Other(t) => Err(PyTypeError::new_err(format!(
            "unsuitable stream data type '{}'",
            t
        ))),
    }
}
