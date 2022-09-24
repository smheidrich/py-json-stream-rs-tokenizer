/// Utilities to allow parsing large integers
///
/// This feature is not available for PyPy or when Py_LIMITED_API is set.
use pyo3::prelude::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ParseIntError {
    pub message: String
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use num_bigint::BigInt;

pub enum AppropriateInt {
    Normal(i64),
    Big(BigInt),
}

#[cfg(all(any(Py_LIMITED_API, PyPy)))]
pub enum AppropriateInt {
    Normal(i64),
}

impl FromStr for AppropriateInt {
    type Err = ParseIntError;

    #[inline]
    fn from_str(s: &str) -> Result<AppropriateInt, ParseIntError> {
        match s.parse::<i64>() {
            Ok(parsed_num) => {
                Ok(AppropriateInt::Normal(parsed_num))
            },
            Err(e) if e.to_string().contains("number too") => {
                #[cfg(not(any(Py_LIMITED_API, PyPy)))]
                match BigInt::from_str(s) {
                    Ok(parsed_num) => Ok(AppropriateInt::Big(parsed_num)),
                    Err(e) => Err(ParseIntError{message: format!("{e:?}")}),
                }
                #[cfg(any(Py_LIMITED_API, PyPy))]
                {
                    e
                }
            }
            Err(e) => {
                Err(ParseIntError{message: format!("{e:?}")})
            }
        }
    }
}

impl IntoPy<PyObject> for AppropriateInt {
    fn into_py(self, py: Python<'_>) -> PyObject {
        #![cfg(not(any(Py_LIMITED_API, PyPy)))]
        match self {
            AppropriateInt::Normal(num) => { num.into_py(py) },
            AppropriateInt::Big(num) => { num.into_py(py) },
        }
        #[cfg(any(Py_LIMITED_API, PyPy))]
        match self {
            AppropriateInt::Normal(num) => { num.into_py(py) },
        }
    }
}
