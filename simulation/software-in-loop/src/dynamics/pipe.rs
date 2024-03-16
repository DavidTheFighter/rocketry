use pyo3::prelude::*;

use super::Scalar;

#[pyclass]
pub struct FluidPipe {
    #[pyo3(get, set)]
    pub inlet_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub outlet_pressure_pa: Scalar,
    #[pyo3(get)]
    pub pressure_pa: Scalar,
    pressure_velocity: Scalar,
}

#[pymethods]
impl FluidPipe {
    #[new]
    pub fn new(inlet_pressure_pa: Scalar, outlet_pressure_pa: Scalar) -> Self {
        Self {
            inlet_pressure_pa,
            outlet_pressure_pa,
            pressure_pa: 0.0,
            pressure_velocity: 1.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        
    }
}
