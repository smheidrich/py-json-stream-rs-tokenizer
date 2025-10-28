use pyo3::types::PyInt;
use pyo3::{Bound, IntoPyObject, Python};

#[derive(Debug, Clone, Copy)]
pub enum PySeekWhence {
    Set = 0,
    Cur = 1,
    End = 2,
}

impl<'py> IntoPyObject<'py> for PySeekWhence {
    type Target = PyInt;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (self as u32).into_pyobject(py)
    }
}
