use pyo3::conversion::IntoPy;
use pyo3::{PyObject, Python};

#[derive(Debug, Clone, Copy)]
pub enum PySeekWhence {
    Set = 0,
    Cur = 1,
    End = 2,
}

impl IntoPy<PyObject> for PySeekWhence {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self as u64).into_py(py)
    }
}
