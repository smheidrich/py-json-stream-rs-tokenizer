use pyo3::{prelude::PyErr, types::PyTraceback, Python};
/// Python error utilities.
use std::fmt::{Display, Error, Formatter};

// outer

pub trait TracebackDisplay<'a> {
    fn traceback_display(&'a self) -> Box<dyn 'a + Display>;
}

impl<'a> TracebackDisplay<'a> for PyErr {
    fn traceback_display(&'a self) -> Box<dyn 'a + Display> {
        Box::new(PyErrTracebackDisplayer { py_err: self })
    }
}

// inner

struct PyErrTracebackDisplayer<'a> {
    py_err: &'a PyErr,
}

impl<'a> Display for PyErrTracebackDisplayer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        String::fmt(
            &Python::with_gil(|py| {
                self.py_err.traceback(py).map_or(
                    Ok("(no traceback available)".to_string()),
                    PyTraceback::format,
                )
            })
            .unwrap_or("(error getting traceback)".to_string()),
            f,
        )
    }
}
