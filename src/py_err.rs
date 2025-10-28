use pyo3::{prelude::PyErr, types::PyTracebackMethods, Python};
/// Python error utilities.
use std::fmt::{Display, Error, Formatter};

// outer

pub trait TracebackDisplay<'a> {
    type Displayer: 'a + Display;

    fn traceback_display(&'a self) -> Self::Displayer;
}

impl<'a> TracebackDisplay<'a> for PyErr {
    type Displayer = PyErrTracebackDisplayer<'a>;

    fn traceback_display(&'a self) -> Self::Displayer {
        PyErrTracebackDisplayer { py_err: self }
    }
}

// inner

pub struct PyErrTracebackDisplayer<'a> {
    py_err: &'a PyErr,
}

impl<'a> Display for PyErrTracebackDisplayer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        String::fmt(
            &Python::attach(|py| {
                self.py_err
                    .traceback(py)
                    .map_or(Ok("(no traceback available)".to_string()), |x| x.format())
            })
            .unwrap_or("(error getting traceback)".to_string()),
            f,
        )
    }
}
