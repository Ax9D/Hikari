use pyo3::{prelude::*, types::PyType};

use super::Entity;

#[pyclass]
pub struct Script {
    entity: Entity
}
impl Script {
}
#[pymethods]
impl Script {
}