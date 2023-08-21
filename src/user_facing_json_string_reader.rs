use crate::JsonStreamingError;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

use crate::RustTokenizer;

#[pyclass]
#[derive(Clone)]
pub struct UserFacingJsonStringReader {
    tokenizer: Py<RustTokenizer>,
}

#[pymethods]
impl UserFacingJsonStringReader {
    pub fn read(slf: PyRefMut<'_, Self>, size: Option<isize>, py: Python<'_>) -> PyResult<String> {
        // normalize size arg
        let max_n_chars: Option<usize> = match size {
            None => None,
            Some(size) if size < 0 => None,
            Some(size) if size == 0 => return Ok("".to_owned()),
            Some(size) => Some(size as usize),
        };
        // /normalize
        Ok(
            match RustTokenizer::parse_string_contents(
                &mut slf.tokenizer.borrow_mut(py),
                max_n_chars,
            )
            // TODO refactor (duplicate code in lib.rs)
            .map_err(|e| -> PyErr {
                let index = slf.tokenizer.borrow(py).index;
                match e {
                    JsonStreamingError::ParsingError(e) => {
                        PyValueError::new_err(format!("{e} at index {index}"))
                    }
                    JsonStreamingError::IOError(e) => PyIOError::new_err(format!(
                        "I/O error while parsing (index {index}): {e:?}"
                    )),
                }
            })? {
                Some(s) => s,
                None => "".to_owned(),
            },
        )
    }
}

impl UserFacingJsonStringReader {
    pub fn new(tokenizer: Py<RustTokenizer>) -> Self {
        UserFacingJsonStringReader { tokenizer }
    }
}
