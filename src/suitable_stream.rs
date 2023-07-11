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
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::types::{PyBytes, PyString};
use pyo3::{PyObject, PyResult, Python};

const DEFAULT_BUFSIZE: usize = 8000;

pub trait SuitableStream: ParkCursorChars + Remainder {}

impl<T: ParkCursorChars + Remainder> SuitableStream for T {}

enum ReadReturnType {
    String,
    Bytes,
    Other(String),
}

fn determine_read_return_type(stream: &PyObject) -> PyResult<ReadReturnType> {
    Python::with_gil(|py| -> PyResult<ReadReturnType> {
        let read_result = stream.as_ref(py).call_method1("read", (0,))?;
        Ok(if read_result.is_instance_of::<PyString>()? {
            ReadReturnType::String
        } else if read_result.is_instance_of::<PyBytes>()? {
            ReadReturnType::Bytes
        } else {
            ReadReturnType::Other(format!("{}", read_result.get_type()))
        })
    })
}

fn is_seekable(stream: &PyObject) -> PyResult<bool> {
    Python::with_gil(|py| -> PyResult<bool> {
        Ok(stream
            .as_ref(py)
            .call_method1("seekable", ())?
            .extract::<bool>()?)
    })
}

pub enum BufferingMode {
    Unbuffered,
    DontCare,
    BufferedWithSize(usize),
}

enum StreamSettings {
    Unbuffered,
    UnseekableBuffered(usize),
    SeekableBuffered(usize),
}

fn decide_stream_settings(
    correct_cursor: bool,
    buffering: BufferingMode,
    seekable: bool,
) -> PyResult<StreamSettings> {
    Ok(match buffering {
        BufferingMode::Unbuffered => StreamSettings::Unbuffered,
        BufferingMode::DontCare => {
            if !correct_cursor {
                StreamSettings::UnseekableBuffered(DEFAULT_BUFSIZE)
            } else {
                if seekable {
                    StreamSettings::SeekableBuffered(DEFAULT_BUFSIZE)
                } else {
                    StreamSettings::Unbuffered
                }
            }
        }
        BufferingMode::BufferedWithSize(bufsize) => {
            if !correct_cursor {
                StreamSettings::UnseekableBuffered(bufsize)
            } else {
                if seekable {
                    StreamSettings::SeekableBuffered(bufsize)
                } else {
                    return Err(PyValueError::new_err(format!(
                        "Incompatible stream requirements: correct_cursor and a buffer size > 1 \
                        are only possible if the given stream is seekable, which this one is not"
                    )));
                }
            }
        }
    })
}

pub fn make_suitable_stream(
    stream: PyObject,
    buffering: BufferingMode,
    correct_cursor: bool,
) -> PyResult<Box<dyn SuitableStream + Send>> {
    let read_return_type: ReadReturnType = determine_read_return_type(&stream)?;
    let seekable: bool = is_seekable(&stream)?;
    let stream_settings = decide_stream_settings(correct_cursor, buffering, seekable)?;
    Ok(match read_return_type {
        ReadReturnType::String => {
            let py_text_stream = PyTextStream::new(stream);
            match stream_settings {
                StreamSettings::Unbuffered => {
                    Box::new(SuitableUnbufferedTextStream::new(py_text_stream))
                }
                StreamSettings::UnseekableBuffered(bufsize) => Box::new(
                    SuitableUnseekableBufferedTextStream::new(py_text_stream, bufsize),
                ),
                StreamSettings::SeekableBuffered(bufsize) => Box::new(
                    SuitableSeekableBufferedTextStream::new(py_text_stream, bufsize),
                ),
            }
        }
        ReadReturnType::Bytes => {
            let py_bytes_stream = PyBytesStream::new(stream);
            match stream_settings {
                StreamSettings::Unbuffered => {
                    Box::new(SuitableUnbufferedBytesStream::new(py_bytes_stream))
                }
                StreamSettings::UnseekableBuffered(bufsize) => Box::new(
                    SuitableUnseekableBufferedBytesStream::new(py_bytes_stream, bufsize),
                ),
                StreamSettings::SeekableBuffered(bufsize) => Box::new(
                    SuitableSeekableBufferedBytesStream::new(py_bytes_stream, bufsize),
                ),
            }
        }
        ReadReturnType::Other(t) => {
            return Err(PyTypeError::new_err(format!(
                "unsuitable stream data type '{}'",
                t
            )))
        }
    })
}
