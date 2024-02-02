use pyo3::prelude::*;

use super::{combustion::{calc_chamber_pressure, CombustionData}, fluid::LiquidDefinition, Scalar};

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
pub struct SilIgniterDynamics {
    #[pyo3(get, set)]
    pub fuel_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub oxidizer_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub fuel_valve_open: bool,
    #[pyo3(get, set)]
    pub oxidizer_valve_open: bool,

    pub fuel_injector: InjectorConfig,
    pub oxidizer_injector: InjectorConfig,
    pub combustion_data: CombustionData,
    pub throat_area_m2: Scalar,

    #[pyo3(get)]
    pub chamber_pressure_pa: Scalar,
}

#[pymethods]
impl SilIgniterDynamics {
    #[new]
    pub fn new(
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
            fuel_injector: fuel_injector.clone(),
            oxidizer_injector: oxidizer_injector.clone(),
            chamber_pressure_pa: 0.0,
            combustion_data: combustion_data.clone(),
            throat_area_m2: throat_diameter_m.powi(2) * std::f64::consts::PI / 4.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let dt = dt as Scalar;

        let fuel_mass_flow_kg = self.calc_fuel_mass_flow_kg(dt);
        let oxidizer_mass_flow_kg = self.calc_oxidizer_mass_flow_kg(dt);

        let total_mass_flow_kg = fuel_mass_flow_kg + oxidizer_mass_flow_kg;
        let mixture_ratio = if fuel_mass_flow_kg == 0.0 {
            Scalar::INFINITY
        } else {
            oxidizer_mass_flow_kg / fuel_mass_flow_kg
        };

        let target_combustion_pressure_pa = if total_mass_flow_kg / dt > 1e-4 && mixture_ratio > 0.2 && mixture_ratio < 3.0 {
             calc_chamber_pressure(
                total_mass_flow_kg / dt,
                self.throat_area_m2,
                &self.combustion_data,
            )

            // println!("{} = {} / {}", (oxidizer_mass_flow_kg / fuel_mass_flow_kg) / dt, oxidizer_mass_flow_kg / dt, fuel_mass_flow_kg / dt);
            // println!("{:.2} Pa {} kg/s", self.chamber_pressure_pa, total_mass_flow_kg / dt);
        } else {
            0.0
        };

        let delta = target_combustion_pressure_pa - self.chamber_pressure_pa;
        self.chamber_pressure_pa += delta * 10.0 * dt;
    }
}

impl SilIgniterDynamics {
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