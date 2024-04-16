use pyo3::prelude::*;

use super::pipe::FluidConnection;

#[pyclass]
pub struct FluidSplitter {
    #[pyo3(get)]
    pub inlet: Py<FluidConnection>,
    #[pyo3(get)]
    pub outlets: Vec<Py<FluidConnection>>,
}

#[pymethods]
impl FluidSplitter {
    #[new]
    pub fn new(
        inlet: Py<FluidConnection>,
        outlets: Vec<Py<FluidConnection>>,
    ) -> Self {
        Self {
            inlet,
            outlets,
        }
    }

    fn post_update(&mut self) {
        // Nothing
    }

    pub fn update(&mut self, py: Python, _dt: f64) {
        let applied_pressure_pa = self.inlet.borrow_mut(py).outlet_pressure_pa();

        for outlet in &self.outlets {
            outlet.borrow_mut(py).new_state.applied_inlet_pressure_pa = applied_pressure_pa;
        }
    }
}
