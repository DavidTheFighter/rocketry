use pyo3::prelude::*;

use super::{pipe::FluidConnection, Scalar};

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct PumpState {
    #[pyo3(get)]
    pub pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub motor_duty_cycle: f64,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilPumpDynamics {
    #[pyo3(get, set)]
    pub state: PumpState,
    #[pyo3(get, set)]
    pub new_state: PumpState,

    #[pyo3(get, set)]
    pub inlet: Py<FluidConnection>,
    #[pyo3(get, set)]
    pub outlet: Py<FluidConnection>,
    #[pyo3(get)]
    pub maximum_pressure_rise_pa: Scalar,

    pub pressure_velocity: Scalar,
}

#[pymethods]
impl SilPumpDynamics {
    #[new]
    pub fn new(
        inlet: Py<FluidConnection>,
        outlet: Py<FluidConnection>,
        maximum_pressure_rise_pa: Scalar,
    ) -> Self {
        Self {
            state: PumpState::default(),
            new_state: PumpState::default(),
            inlet,
            outlet,
            maximum_pressure_rise_pa,
            pressure_velocity: 3.0,
        }
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        let dt = dt as Scalar;

        let inlet_pressure = self.inlet.borrow(py).outlet_pressure_pa();

        let target_pressure_rise_pa = self.calc_target_pressure_rise_pa();
        let delta = target_pressure_rise_pa - self.state.pressure_pa + inlet_pressure;

        self.new_state.pressure_pa += delta * self.pressure_velocity * dt;
        self.outlet
            .borrow_mut(py)
            .new_state
            .applied_outlet_pressure_pa = self.new_state.pressure_pa;
    }

    fn post_update(&mut self) {
        self.state = self.new_state.clone();
    }
}

impl SilPumpDynamics {
    fn calc_target_pressure_rise_pa(&self) -> Scalar {
        let duty_cycle = self.state.motor_duty_cycle as Scalar;

        duty_cycle * self.maximum_pressure_rise_pa
    }

    fn calc_target_inlet_suction_pa(&self) -> Scalar {
        0.0
    }

    fn calc_target_inducer_pressure_rise_pa(&self) -> Scalar {
        0.0
    }
}
