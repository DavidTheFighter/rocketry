pub mod combustion;
pub mod fluid;
pub mod igniter;
pub mod pipe;
pub mod pump;
pub mod tank;
pub mod vehicle;

type Scalar = f64;

pub const ATMOSPHERIC_PRESSURE_PA: Scalar = 101325.0;

pub use tank::SilTankDynamics;
pub use tank::SilTankFeedConfig;
pub use vehicle::SilVehicleDynamics;

use pyo3::prelude::*;

#[pyclass]
pub struct DynamicsManager {
    components: Vec<PyObject>,
}

#[pymethods]
impl DynamicsManager {
    #[new]
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add_dynamics_component(&mut self, component: PyObject) {
        self.components.push(component);
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        for component in self.components.iter_mut() {
            component.call_method1(py, "update", (dt,)).expect(&format!(
                "Failed to call update on {:?}",
                component.to_string()
            ));
        }

        for component in self.components.iter_mut() {
            component.call_method0(py, "post_update").expect(&format!(
                "Failed to call pre_update on {:?}",
                component.to_string()
            ));
        }
    }
}
