/// Streamable strings support.
///
/// Adapted almost verbatim from json-stream's `strings.py` module.

use compact_str::CompactString;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::borrow::BorrowMut;
use std::io::BufRead;

const DEFAULT_BUFFER_SIZE: usize = 8192; // TODO get from Python's io module somehow?

enum State {
    Char,
    StringEscape,
    Unicode,
    UnicodeSurrogateStart,
    UnicodeSurrogateStringEscape,
    UnicodeSurrogate,
}

#[pyclass]
pub struct JsonStringReader {
    stream: Box<dyn SuitableStream + Send>,
    buffer: String,
    readline_buffer: String,
    unicode_buffer: CompactString,
    state: State,
    end_of_string: bool,
    index: usize,
}

#[pymethods]
impl JsonStringReader {
    pub fn complete(slf: PyRef<'_, Self>) -> bool {
        Self::_complete(&slf)
    }

    pub fn read(mut slf: PyRefMut<'_, Self>, size: Option<usize>) -> PyResult<String> {
        let mut result = String::new();
        let mut length = DEFAULT_BUFFER_SIZE;
        while !Self::_complete(&slf) && (size == None || result.is_empty()) {
            if let Some(_size) = size {
                length = _size - result.len()
            }
            // TODO performance will be trash here:
            result.push_str(Self::read_chunk(slf.borrow_mut(), length)?.as_str())
        }
        return Ok(result);
    }
}

impl JsonStringReader {
    pub fn new(stream: Box<dyn BufRead + Send>, initial_buffer: String) -> JsonStringReader {
        JsonStringReader {
            stream,
            buffer: initial_buffer,
            readline_buffer: String::new(),
            unicode_buffer: CompactString::with_capacity(4),
            state: State::Char,
            end_of_string: false,
            index: 0,
        }
    }

    pub fn _complete(slf: &Self) -> bool {
        slf.end_of_string && slf.readline_buffer.is_empty()
    }

    pub fn read_chunk(slf: &mut Self, size: usize) -> PyResult<String> {
        if !slf.readline_buffer.is_empty() {
            let result = slf.readline_buffer[..size].to_string();
            slf.readline_buffer = slf.readline_buffer[size..].to_string();
            return Ok(result.to_string());
        }
        let chunk = if slf.buffer.len() > 0 {
            slf.buffer
        } else {
            let newbuf = String::with_capacity(4*size);
            slf.stream.read(&mut newbuf.as_bytes());
            newbuf
        };
        if chunk.is_empty() {
            return Err(PyValueError::new_err(format!("Unterminated string at end of file")));
        }
        let mut result = String::new();
        let start = 0;
        for (i, c) in chunk.chars().enumerate() {
            slf.index += 1;
            if i == size {
                if let State::Char = slf.state {
                    result.push_str(&chunk[start..i]);
                }
                slf.buffer = chunk[i..].to_string();
                break
            }
        }
        Ok(String::new())
    }
}
