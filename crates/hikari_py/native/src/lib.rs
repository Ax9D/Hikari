mod math;
mod ecs;

use pyo3::prelude::*;

#[pymodule]
fn hikari(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<math::Vec2>()?;
    m.add_class::<math::Vec3>()?;
    m.add_class::<math::Vec4>()?;

    Ok(())
}
