use pyo3::prelude::*;

use super::{Scalar, ATMOSPHERIC_PRESSURE_PA};

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct FluidConnectionState {
    #[pyo3(get, set)]
    pub applied_inlet_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub applied_outlet_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub closed: bool,
    pub pressure_pa: Scalar,
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
        let applied_pressure = self.state.applied_inlet_pressure_pa.max(self.state.applied_outlet_pressure_pa);
        let delta = applied_pressure - self.state.pressure_pa;
        self.new_state.pressure_pa = self.state.pressure_pa + delta * self.pressure_velocity * dt as Scalar;
    }

    #[getter]
    pub fn outlet_pressure_pa(&self) -> Scalar {
        if self.state.closed {
            self.state.applied_outlet_pressure_pa
        } else {
            self.state.pressure_pa
        }
    }

    #[getter]
    pub fn inlet_pressure_pa(&self) -> Scalar {
        if self.state.closed {
            self.state.applied_inlet_pressure_pa
        } else {
            self.state.pressure_pa
        }
    }
}
