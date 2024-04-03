use pyo3::prelude::*;

use super::{fluid::GasDefinition, pipe::FluidConnection, Scalar, ATMOSPHERIC_PRESSURE_PA};

pub const GAS_CONSTANT: Scalar = 8.31446261815324;

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilTankFeedConfig {
    #[pyo3(get, set)]
    pub feed_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub feed_setpoint_pa: Scalar,
    #[pyo3(get, set)]
    pub feed_gas: GasDefinition,
    #[pyo3(get, set)]
    pub feed_orifice_diameter_m: Scalar,
    #[pyo3(get, set)]
    pub feed_orifice_cd: Scalar,
    #[pyo3(get, set)]
    pub feed_gas_temp_k: Scalar,
}

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct TankState {
    #[pyo3(get, set)]
    pub feed_valve_open: bool,
    #[pyo3(get, set)]
    pub vent_valve_open: bool,
    #[pyo3(get)]
    pub tank_pressure_pa: Scalar,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilTankDynamics {
    feed_config: SilTankFeedConfig,
    vent_orifice_diameter_area_m2: Scalar,
    vent_orifice_cd: Scalar,
    tank_volume_m3: Scalar,

    #[pyo3(get, set)]
    pub state: TankState,
    #[pyo3(get, set)]
    pub new_state: TankState,

    #[pyo3(get, set)]
    pub outlet: Py<FluidConnection>,
}

#[pymethods]
impl SilTankDynamics {
    #[new]
    pub fn new(
        feed_config: &mut SilTankFeedConfig,
        vent_orifice_diameter_m: Scalar,
        vent_orifice_cd: Scalar,
        initial_tank_pressure_pa: Scalar,
        tank_volume_m3: Scalar,
        outlet: Py<FluidConnection>,
    ) -> Self {
        Self {
            feed_config: feed_config.clone(),
            vent_orifice_diameter_area_m2: vent_orifice_diameter_m.powi(2) * std::f64::consts::PI
                / 4.0,
            vent_orifice_cd,
            tank_volume_m3,
            new_state: TankState {
                feed_valve_open: false,
                vent_valve_open: false,
                tank_pressure_pa: initial_tank_pressure_pa,
            },
            state: TankState {
                feed_valve_open: false,
                vent_valve_open: false,
                tank_pressure_pa: initial_tank_pressure_pa,
            },
            outlet,
        }
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        let dt = dt as Scalar;

        let feed_mass_flow_kg = self.calc_feed_mass_flow_kg(dt);
        let vent_mass_flow_kg = self.calc_vent_mass_flow_kg(dt);

        let mass_flow_kg = feed_mass_flow_kg - vent_mass_flow_kg;

        // Pressure difference is proportional to the mass flow
        let tank_mass_kg = self.calc_tank_mass_kg();

        self.new_state.tank_pressure_pa *= (tank_mass_kg + mass_flow_kg) / tank_mass_kg;
        self.outlet
            .borrow_mut(py)
            .new_state
            .applied_inlet_pressure_pa = self.state.tank_pressure_pa;
    }

    fn post_update(&mut self) {
        self.state = self.new_state.clone();
    }

    #[getter]
    pub fn tank_pressure_pa(&self) -> f64 {
        self.state.tank_pressure_pa
    }
}

impl SilTankDynamics {
    fn calc_feed_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        if self.state.tank_pressure_pa >= self.feed_config.feed_setpoint_pa
            || !self.state.feed_valve_open
        {
            return 0.0;
        }

        let upstream_gas_density = self.feed_config.feed_gas.molecular_weight_kg
            * self.feed_config.feed_pressure_pa
            / (GAS_CONSTANT * self.feed_config.feed_gas_temp_k);

        let expansibility_factor = 1.0;
        let regulator_factor = (self.feed_config.feed_setpoint_pa - self.state.tank_pressure_pa)
            / self.feed_config.feed_setpoint_pa;

        let mass_flow_rate_kg_s = regulator_factor
            * self.feed_config.feed_orifice_area_m2()
            * self.feed_config.feed_orifice_cd
            * expansibility_factor
            * (2.0
                * upstream_gas_density
                * (self.feed_config.feed_pressure_pa - self.state.tank_pressure_pa))
                .sqrt();

        mass_flow_rate_kg_s * dt
    }

    fn calc_vent_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        if self.state.tank_pressure_pa <= ATMOSPHERIC_PRESSURE_PA || !self.state.vent_valve_open {
            return 0.0;
        }

        let upstream_gas_density = self.feed_config.feed_gas.molecular_weight_kg
            * self.state.tank_pressure_pa
            / (GAS_CONSTANT * self.feed_config.feed_gas_temp_k);

        let expansibility_factor = 1.0;

        let mass_flow_rate_kg_s = self.vent_orifice_diameter_area_m2
            * self.vent_orifice_cd
            * expansibility_factor
            * (2.0
                * upstream_gas_density
                * (self.state.tank_pressure_pa - ATMOSPHERIC_PRESSURE_PA))
                .sqrt();

        mass_flow_rate_kg_s * dt
    }

    fn calc_tank_mass_kg(&self) -> Scalar {
        self.state.tank_pressure_pa
            * self.tank_volume_m3
            * self.feed_config.feed_gas.molecular_weight_kg
            / (GAS_CONSTANT * self.feed_config.feed_gas_temp_k)
    }
}

#[pymethods]
impl SilTankFeedConfig {
    #[new]
    pub fn new(
        feed_pressure_pa: Scalar,
        feed_setpoint_pa: Scalar,
        feed_gas: GasDefinition,
        feed_orifice_diameter_m: Scalar,
        feed_orifice_cd: Scalar,
        feed_gas_temp_k: Scalar,
    ) -> Self {
        Self {
            feed_pressure_pa,
            feed_setpoint_pa,
            feed_gas,
            feed_orifice_diameter_m,
            feed_orifice_cd,
            feed_gas_temp_k,
        }
    }
}

impl SilTankFeedConfig {
    fn feed_orifice_area_m2(&self) -> Scalar {
        std::f64::consts::PI * self.feed_orifice_diameter_m.powi(2) / 4.0
    }
}
