use pyo3::prelude::*;

use super::Scalar;

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct FluidConnectionState {
    #[pyo3(get, set)]
    pub inlet_pressure_pa: Scalar,
    #[pyo3(get)]
    pub outlet_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub closed: bool,
}

#[pyclass]
pub struct FluidConnection {
    #[pyo3(get, set)]
    pub state: FluidConnectionState,
    #[pyo3(get, set)]
    pub new_state: FluidConnectionState,

    #[pyo3(get, set)]
    pressure_velocity: Scalar,
}

#[pymethods]
impl FluidConnection {
    #[new]
    pub fn new() -> Self {
        Self {
            state: FluidConnectionState::default(),
            new_state: FluidConnectionState::default(),
            pressure_velocity: 20.0,
        }
    }

    fn post_update(&mut self) {
        self.state = self.new_state.clone();
    }

    pub fn update(&mut self, dt: f64) {
        let pressure = if self.state.closed {
            0.0
        } else {
            self.state.inlet_pressure_pa
        };

        let delta = pressure - self.state.outlet_pressure_pa;
        self.new_state.outlet_pressure_pa = self.state.outlet_pressure_pa + delta * self.pressure_velocity * (dt as Scalar);

    }
}
