use pyo3::prelude::*;

use super::{
    fluid::{GasDefinition, LiquidDefinition},
    pipe::FluidConnection,
    Scalar, ATMOSPHERIC_PRESSURE_PA,
};

pub const GAS_CONSTANT: Scalar = 8.31446261815324;

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilTankPressConfig {
    #[pyo3(get, set)]
    pub press_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub press_setpoint_pa: Scalar,
    #[pyo3(get, set)]
    pub press_orifice_diameter_m: Scalar,
    #[pyo3(get, set)]
    pub press_orifice_cd: Scalar,
    #[pyo3(get, set)]
    pub press_gas_temp_k: Scalar,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct TankState {
    #[pyo3(get, set)]
    pub press_valve_open: bool,
    #[pyo3(get, set)]
    pub vent_valve_open: bool,

    #[pyo3(get)]
    pub propellant_mass_kg: Scalar,
    #[pyo3(get)]
    pub propellant_mass_flow_kg_s: Scalar,

    #[pyo3(get)]
    pub ullage_temperature_k: Scalar,
    #[pyo3(get)]
    pub ullage_gas_mass_kg: Scalar,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilTankDynamics {
    press_config: Option<SilTankPressConfig>,

    propellant_liquid: LiquidDefinition,
    ullage_gas: GasDefinition,

    vent_orifice_diameter_area_m2: Scalar,
    vent_orifice_cd: Scalar,
    tank_volume_m3: Scalar,

    #[pyo3(get, set)]
    pub state: TankState,
    #[pyo3(get, set)]
    pub new_state: TankState,

    #[pyo3(get, set)]
    pub ullage_inlet: Py<PyAny>,

    #[pyo3(get, set)]
    pub outlet: Py<FluidConnection>,

    #[pyo3(get, set)]
    pub ullage_outlet: Py<PyAny>,
}

#[pymethods]
impl SilTankDynamics {
    #[new]
    pub fn new(
        py: Python,
        press_config: PyObject,
        ullage_gas: GasDefinition,
        propellant_liquid: LiquidDefinition,
        vent_orifice_diameter_m: Scalar,
        vent_orifice_cd: Scalar,
        initial_propellant_mass_kg: Scalar,
        initial_tank_pressure_pa: Scalar,
        initial_tank_temperature_k: Scalar,
        tank_volume_m3: Scalar,
        outlet: Py<FluidConnection>,
    ) -> Self {
        let press_config: Option<SilTankPressConfig> = if press_config.is_none(py) {
            None
        } else {
            Some(press_config.extract(py).unwrap())
        };

        let initial_propellant_volume_m3 =
            initial_propellant_mass_kg / propellant_liquid.density_kg_m3;
        if initial_propellant_volume_m3 > tank_volume_m3 * 0.99 {
            panic!("Initial propellant mass exceeds tank volume");
        }

        println!(
            "Propellant {} volume fraction: {:.2}",
            propellant_liquid.name,
            initial_propellant_volume_m3 / tank_volume_m3
        );

        let initial_tank_state = TankState {
            press_valve_open: false,
            vent_valve_open: false,
            propellant_mass_kg: initial_propellant_mass_kg,
            propellant_mass_flow_kg_s: 0.0,
            ullage_temperature_k: initial_tank_temperature_k,
            ullage_gas_mass_kg: initial_tank_pressure_pa
                * (tank_volume_m3 - initial_propellant_volume_m3)
                * ullage_gas.molecular_weight_kg
                / (GAS_CONSTANT * initial_tank_temperature_k),
        };

        let tank = Self {
            press_config,
            propellant_liquid,
            ullage_gas,
            vent_orifice_diameter_area_m2: vent_orifice_diameter_m.powi(2) * std::f64::consts::PI
                / 4.0,
            vent_orifice_cd,
            tank_volume_m3,
            new_state: initial_tank_state.clone(),
            state: initial_tank_state.clone(),
            ullage_inlet: py.None(),
            outlet,
            ullage_outlet: py.None(),
        };

        println!(
            "Initial ullage pressure: {:.2} Pa",
            tank.calc_ullage_pressure_pa()
        );

        tank
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        let dt = dt as Scalar;

        let pressure_diff =
            self.propellant_liquid.vapor_pressure_pa - self.calc_ullage_pressure_pa();

        let press_mass_flow_kg = self.calc_press_mass_flow_kg(dt);
        let vent_mass_flow_kg = self.calc_vent_mass_flow_kg(dt);
        let boil_off_mass_flow_kg = if pressure_diff > 0.0 {
            pressure_diff * self.calc_propellant_volume_m3() * self.ullage_gas.molecular_weight_kg
                / (GAS_CONSTANT * self.new_state.ullage_temperature_k)
                * dt
        } else {
            0.0
        };

        let ullage_outlet_mass_flow_kg = if self.ullage_outlet.is_none(py) {
            0.0
        } else {
            self.ullage_outlet
                .as_ref(py)
                .extract::<Py<FluidConnection>>()
                .unwrap()
                .borrow(py)
                .state
                .mass_flow_rate_kg_s
        };

        let ullage_inlet_mass_flow_kg = self.calc_ullage_inlet_mass_flow(py, dt);

        let ullage_mass_flow_kg = press_mass_flow_kg - vent_mass_flow_kg + boil_off_mass_flow_kg
            - ullage_outlet_mass_flow_kg
            + ullage_inlet_mass_flow_kg;
        self.new_state.ullage_gas_mass_kg += ullage_mass_flow_kg;

        self.new_state.propellant_mass_flow_kg_s =
            -boil_off_mass_flow_kg - self.outlet.borrow(py).state.mass_flow_rate_kg_s;
        self.new_state.propellant_mass_kg += self.state.propellant_mass_flow_kg_s * dt;

        if self.new_state.propellant_mass_kg < 0.0 {
            self.new_state.propellant_mass_kg = 0.0;
        }

        if !self.ullage_outlet.is_none(py) {
            self.ullage_outlet
                .as_ref(py)
                .extract::<Py<FluidConnection>>()
                .unwrap()
                .borrow_mut(py)
                .new_state
                .applied_inlet_pressure_pa = self.calc_ullage_pressure_pa();
        }

        if !self.ullage_inlet.is_none(py) {
            self.ullage_inlet
                .as_ref(py)
                .extract::<Py<FluidConnection>>()
                .unwrap()
                .borrow_mut(py)
                .new_state
                .applied_outlet_pressure_pa = self.calc_ullage_pressure_pa();

            self.ullage_inlet
                .as_ref(py)
                .extract::<Py<FluidConnection>>()
                .unwrap()
                .borrow_mut(py)
                .new_state
                .mass_flow_rate_kg_s = ullage_inlet_mass_flow_kg;
        }

        self.outlet
            .borrow_mut(py)
            .new_state
            .applied_inlet_pressure_pa = self.calc_ullage_pressure_pa();
    }

    fn post_update(&mut self) {
        self.state = self.new_state.clone();
    }

    #[getter]
    pub fn tank_pressure_pa(&self) -> f64 {
        self.calc_ullage_pressure_pa()
    }
}

impl SilTankDynamics {
    fn calc_press_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        if let Some(press_config) = &self.press_config {
            let ullage_pressure_pa = self.calc_ullage_pressure_pa();

            if ullage_pressure_pa >= press_config.press_setpoint_pa || !self.state.press_valve_open
            {
                return 0.0;
            }

            let upstream_gas_density = self.ullage_gas.molecular_weight_kg
                * press_config.press_pressure_pa
                / (GAS_CONSTANT * press_config.press_gas_temp_k);

            let expansibility_factor = 1.0;
            let regulator_factor = (press_config.press_setpoint_pa - ullage_pressure_pa)
                / press_config.press_setpoint_pa;

            let mass_flow_rate_kg_s = regulator_factor
                * press_config.press_orifice_area_m2()
                * press_config.press_orifice_cd
                * expansibility_factor
                * (2.0
                    * upstream_gas_density
                    * (press_config.press_pressure_pa - ullage_pressure_pa))
                    .sqrt();

            mass_flow_rate_kg_s * dt
        } else {
            0.0
        }
    }

    fn calc_vent_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        let ullage_pressure_pa = self.calc_ullage_pressure_pa();

        if ullage_pressure_pa <= ATMOSPHERIC_PRESSURE_PA || !self.state.vent_valve_open {
            return 0.0;
        }

        let upstream_gas_density = self.ullage_gas.molecular_weight_kg * ullage_pressure_pa
            / (GAS_CONSTANT * self.state.ullage_temperature_k);

        let expansibility_factor = 1.0;

        let mass_flow_rate_kg_s = self.vent_orifice_diameter_area_m2
            * self.vent_orifice_cd
            * expansibility_factor
            * (2.0 * upstream_gas_density * (ullage_pressure_pa - ATMOSPHERIC_PRESSURE_PA)).sqrt();

        mass_flow_rate_kg_s * dt
    }

    fn calc_ullage_inlet_mass_flow(&self, py: Python, dt: Scalar) -> Scalar {
        if self.ullage_inlet.is_none(py) {
            return 0.0;
        }

        let inlet = self
            .ullage_inlet
            .as_ref(py)
            .extract::<Py<FluidConnection>>()
            .unwrap();

        if inlet.borrow(py).state.closed {
            return 0.0;
        }

        let inlet_pressure_pa = inlet.borrow(py).state.applied_inlet_pressure_pa;
        let outlet_pressure_pa = inlet.borrow(py).state.applied_outlet_pressure_pa;

        let upstream_gas_density = self.ullage_gas.molecular_weight_kg * inlet_pressure_pa
            / (GAS_CONSTANT * self.state.ullage_temperature_k);

        const PIPE_DIAMETER_M: Scalar = 0.004;
        let mut result = PIPE_DIAMETER_M
            * PIPE_DIAMETER_M
            * upstream_gas_density
            * (inlet_pressure_pa - outlet_pressure_pa).abs().sqrt()
            * dt;

        if inlet_pressure_pa < outlet_pressure_pa {
            result *= -1.0;
        }

        result
    }

    fn calc_propellant_volume_m3(&self) -> Scalar {
        self.state.propellant_mass_kg / self.propellant_liquid.density_kg_m3
    }

    fn calc_ullage_volume_m3(&self) -> Scalar {
        self.tank_volume_m3 - self.calc_propellant_volume_m3()
    }

    fn calc_ullage_pressure_pa(&self) -> Scalar {
        self.state.ullage_gas_mass_kg * GAS_CONSTANT * self.state.ullage_temperature_k
            / (self.ullage_gas.molecular_weight_kg * self.calc_ullage_volume_m3())
    }
}

#[pymethods]
impl SilTankPressConfig {
    #[new]
    pub fn new(
        press_pressure_pa: Scalar,
        press_setpoint_pa: Scalar,
        press_orifice_diameter_m: Scalar,
        press_orifice_cd: Scalar,
        press_gas_temp_k: Scalar,
    ) -> Self {
        Self {
            press_pressure_pa,
            press_setpoint_pa,
            press_orifice_diameter_m,
            press_orifice_cd,
            press_gas_temp_k,
        }
    }
}

impl SilTankPressConfig {
    fn press_orifice_area_m2(&self) -> Scalar {
        std::f64::consts::PI * self.press_orifice_diameter_m.powi(2) / 4.0
    }
}
