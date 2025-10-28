/// Utilities to allow parsing large integers
///
/// This feature is not available for PyPy or when Py_LIMITED_API is set.
use pyo3::prelude::*;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseIntError {
    #[error("general integer parsing error: {0}")]
    General(String),
    #[error("integer too large or small")]
    #[allow(dead_code)]
    TooLargeOrSmall, // deprecated
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use num_bigint::BigInt;

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
pub enum AppropriateInt {
    Normal(i64),
    Big(BigInt),
}

#[cfg(any(Py_LIMITED_API, PyPy))]
pub enum AppropriateInt {
    Normal(i64),
    Big(String), // to be converted into int on the Python side
}

impl FromStr for AppropriateInt {
    type Err = ParseIntError;

    #[inline]
    fn from_str(s: &str) -> Result<AppropriateInt, ParseIntError> {
        match s.parse::<i64>() {
            Ok(parsed_num) => Ok(AppropriateInt::Normal(parsed_num)),
            Err(e) if e.to_string().contains("number too") => {
                #[cfg(not(any(Py_LIMITED_API, PyPy)))]
                match BigInt::from_str(s) {
                    Ok(parsed_num) => Ok(AppropriateInt::Big(parsed_num)),
                    Err(e) => Err(ParseIntError::General(format!("{e:?}"))),
                }
                #[cfg(any(Py_LIMITED_API, PyPy))]
                Ok(AppropriateInt::Big(s.to_owned()))
            }
            Err(e) => Err(ParseIntError::General(format!("{e:?}"))),
        }
    }
}

impl<'py> IntoPyObject<'py> for AppropriateInt {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(match self {
            AppropriateInt::Normal(num) => num.into_pyobject(py)?.into_any(),
            AppropriateInt::Big(num) => num.into_pyobject(py)?.into_any(),
        })
    }
}

pub fn supports_bigint() -> bool {
    // TODO: I think both of these *do* support BigInt in recent PyO3 versions => test & lift
    //       restriction
    #[cfg(any(Py_LIMITED_API, PyPy))]
    {
        return false;
    }
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    {
        true
    }
}
