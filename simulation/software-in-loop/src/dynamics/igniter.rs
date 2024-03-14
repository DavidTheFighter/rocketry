use pyo3::prelude::*;

use super::{combustion::{calc_chamber_pressure, CombustionData}, fluid::LiquidDefinition, Scalar};

pub const MINIMUM_SUSTAINABLE_CHAMBER_PRESSURE_PA: Scalar = 206843.0; // 30 PSI

#[pyclass]
#[derive(Debug, Clone)]
pub struct InjectorConfig {
    #[pyo3(get, set)]
    pub injector_orifice_diameter_m: Scalar,
    #[pyo3(get, set)]
    pub injector_orifice_cd: Scalar,
    #[pyo3(get, set)]
    pub liquid: LiquidDefinition,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilIgniterDynamics {
    #[pyo3(get, set)]
    pub fuel_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub oxidizer_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub fuel_valve_open: bool,
    #[pyo3(get, set)]
    pub oxidizer_valve_open: bool,
    #[pyo3(get, set)]
    pub has_ignition_source: bool,

    pub fuel_injector: InjectorConfig,
    pub oxidizer_injector: InjectorConfig,
    pub combustion_data: CombustionData,
    pub throat_area_m2: Scalar,

    pub combustion_pressure_modifier: PyObject,

    #[pyo3(get)]
    pub chamber_pressure_pa: Scalar,
}

#[pymethods]
impl SilIgniterDynamics {
    #[new]
    pub fn new(
        py: Python,
        fuel_injector: &mut InjectorConfig,
        oxidizer_injector: &mut InjectorConfig,
        combustion_data: &mut CombustionData,
        throat_diameter_m: Scalar,
    ) -> Self {
        Self {
            fuel_pressure_pa: 0.0,
            oxidizer_pressure_pa: 0.0,
            fuel_valve_open: false,
            oxidizer_valve_open: false,
            has_ignition_source: false,
            fuel_injector: fuel_injector.clone(),
            oxidizer_injector: oxidizer_injector.clone(),
            chamber_pressure_pa: 0.0,
            combustion_data: combustion_data.clone(),
            combustion_pressure_modifier: py.None(),
            throat_area_m2: throat_diameter_m.powi(2) * std::f64::consts::PI / 4.0,
        }
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        let dt = dt as Scalar;

        let fuel_mass_flow_kg = self.calc_fuel_mass_flow_kg(dt);
        let oxidizer_mass_flow_kg = self.calc_oxidizer_mass_flow_kg(dt);

        let total_mass_flow_kg = fuel_mass_flow_kg + oxidizer_mass_flow_kg;

        let mut target_combustion_pressure_pa = if self.can_support_combustion(fuel_mass_flow_kg, oxidizer_mass_flow_kg) {
             calc_chamber_pressure(
                total_mass_flow_kg / dt,
                self.throat_area_m2,
                &self.combustion_data,
            )
        } else {
            0.0
        };

        if let Ok(result) = self.combustion_pressure_modifier.call1(py, (target_combustion_pressure_pa,)) {
            if let Ok(pressure) = result.extract::<Scalar>(py) {
                target_combustion_pressure_pa = pressure;
            }
        }

        let delta = target_combustion_pressure_pa - self.chamber_pressure_pa;
        self.chamber_pressure_pa += delta * 10.0 * dt;
    }

    pub fn set_combustion_pressure_modifier(&mut self, callback: PyObject) {
        self.combustion_pressure_modifier = callback;
    }
}

impl SilIgniterDynamics {
    fn can_support_combustion(
        &self,
        fuel_mass_flow_kg: Scalar,
        oxidizer_mass_flow_kg: Scalar,
    ) -> bool {
        let mixture_ratio = if fuel_mass_flow_kg == 0.0 {
            Scalar::INFINITY
        } else {
            oxidizer_mass_flow_kg / fuel_mass_flow_kg
        };

        // There needs to be mass flow within reasonable mixture ratio
        if !(fuel_mass_flow_kg > 0.0
            && oxidizer_mass_flow_kg > 0.0
            && mixture_ratio > 0.2
            && mixture_ratio < 3.0)
        {
            return false;
        }

        if self.chamber_pressure_pa > MINIMUM_SUSTAINABLE_CHAMBER_PRESSURE_PA { // 30 PSI
            return true;
        }

        if self.has_ignition_source {
            return true;
        }

        false
    }

    fn calc_fuel_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        if !self.fuel_valve_open || self.fuel_pressure_pa <= self.chamber_pressure_pa{
            return 0.0;
        }

        let mass_flow_rate_kg_s = self.fuel_injector.injector_area_m2()
            * self.fuel_injector.injector_orifice_cd
            * (2.0
                * self.fuel_injector.liquid.density_kg_m3
                * (self.fuel_pressure_pa - self.chamber_pressure_pa)
            ).sqrt();

        mass_flow_rate_kg_s * dt
    }

    fn calc_oxidizer_mass_flow_kg(&self, dt: Scalar) -> Scalar {
        if !self.oxidizer_valve_open || self.oxidizer_pressure_pa <= self.chamber_pressure_pa{
            return 0.0;
        }

        let mass_flow_rate_kg_s = self.oxidizer_injector.injector_area_m2()
            * self.oxidizer_injector.injector_orifice_cd
            * (2.0
                * self.oxidizer_injector.liquid.density_kg_m3
                * (self.oxidizer_pressure_pa - self.chamber_pressure_pa)
            ).sqrt();

        mass_flow_rate_kg_s * dt
    }
}

#[pymethods]
impl InjectorConfig {
    #[new]
    pub fn new(
        injector_orifice_diameter_m: Scalar,
        injector_orifice_cd: Scalar,
        liquid: LiquidDefinition,
    ) -> Self {
        Self {
            injector_orifice_diameter_m,
            injector_orifice_cd,
            liquid,
        }
    }
}

impl InjectorConfig {
    fn injector_area_m2(&self) -> Scalar {
        self.injector_orifice_diameter_m.powi(2) * std::f64::consts::PI / 4.0
    }
}