use crate::park_cursor::ParkCursorChars;
use crate::py_bytes_stream::PyBytesStream;
use crate::py_text_stream::PyTextStream;
use crate::suitable_bytes_stream::SuitableBytesStream;
use crate::suitable_text_stream::SuitableTextStream;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyBytes, PyString};
use pyo3::{PyObject, PyResult, Python};

enum ReadReturnType {
    String,
    Bytes,
    Other(String),
}

pub fn make_suitable_stream(stream: PyObject) -> PyResult<Box<dyn ParkCursorChars + Send>> {
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
    match read_return_type {
        ReadReturnType::String => Ok(Box::new(SuitableTextStream::new(PyTextStream::new(stream)))),
        ReadReturnType::Bytes => Ok(Box::new(SuitableBytesStream::new(PyBytesStream::new(
            stream,
        )))),
        ReadReturnType::Other(t) => Err(PyTypeError::new_err(format!(
            "unsuitable stream data type '{}'",
            t
        ))),
    }
}
