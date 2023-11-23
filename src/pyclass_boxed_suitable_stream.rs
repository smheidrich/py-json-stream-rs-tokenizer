use std::ops::{Deref, DerefMut};

use pyo3::prelude::*;

use crate::suitable_stream::SuitableStream;

/// Wrapper around `Box<SuitableStream>` that allows storing it on the Python side of things.
///
/// The advantage of this is that accesses are safeguarded by Python's GIL.
///
/// Only `PyClass` types can be put inside `Py<T>`, so all this does is wrap the actual object in
/// one.
#[pyclass]
pub struct PyClassBoxedSuitableStream {
    stream: Box<dyn SuitableStream + Send>,
}

impl PyClassBoxedSuitableStream {
    pub fn new(stream: Box<dyn SuitableStream + Send>) -> Self {
        Self { stream }
    }
}

// implement deref because this is basically meant as a smart pointer like thing
impl Deref for PyClassBoxedSuitableStream {
    type Target = Box<dyn SuitableStream + Send>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for PyClassBoxedSuitableStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
