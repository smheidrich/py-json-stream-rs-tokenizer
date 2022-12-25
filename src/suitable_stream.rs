/// Utility for dynamically constructing an appropriate "suitable" stream from a raw Python stream.
///
/// "Suitable" means that it implements ParkCursorChars (too lazy to make another trait for that),
/// although it can be a no-op.
use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::py_text_stream::PyTextStream;
use crate::remainder::Remainder;
use crate::suitable_seekable_buffered_bytes_stream::SuitableSeekableBufferedBytesStream;
use crate::suitable_seekable_buffered_text_stream::SuitableSeekableBufferedTextStream;
use crate::suitable_unbuffered_bytes_stream::SuitableUnbufferedBytesStream;
use crate::suitable_unbuffered_text_stream::SuitableUnbufferedTextStream;
use crate::suitable_unseekable_buffered_bytes_stream::SuitableUnseekableBufferedBytesStream;
use crate::suitable_unseekable_buffered_text_stream::SuitableUnseekableBufferedTextStream;
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

pub fn make_suitable_stream(
    stream: PyObject,
    correct_cursor: bool,
) -> PyResult<Box<dyn SuitableStream + Send>> {
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
            Ok(if correct_cursor {
                if seekable {
                    Box::new(SuitableSeekableBufferedTextStream::new(py_text_stream))
                } else {
                    Box::new(SuitableUnbufferedTextStream::new(py_text_stream))
                }
            } else {
                Box::new(SuitableUnseekableBufferedTextStream::new(py_text_stream))
            })
        }
        ReadReturnType::Bytes => {
            let py_bytes_stream = PyBytesStream::new(stream);
            Ok(if correct_cursor {
                if seekable {
                    Box::new(SuitableSeekableBufferedBytesStream::new(py_bytes_stream))
                } else {
                    Box::new(SuitableUnbufferedBytesStream::new(py_bytes_stream))
                }
            } else {
                Box::new(SuitableUnseekableBufferedBytesStream::new(py_bytes_stream))
            })
        }
        ReadReturnType::Other(t) => Err(PyTypeError::new_err(format!(
            "unsuitable stream data type '{}'",
            t
        ))),
    }
}
