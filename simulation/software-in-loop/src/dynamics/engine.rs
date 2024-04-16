use pyo3::prelude::*;

use super::{
    combustion::{calc_chamber_pressure, CombustionData}, pipe::FluidConnection, InjectorConfig, Scalar, ATMOSPHERIC_PRESSURE_PA
};

pub const MINIMUM_SUSTAINABLE_CHAMBER_PRESSURE_PA: Scalar = 206843.0; // 30 PSI

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct EngineState {
    #[pyo3(get)]
    pub chamber_pressure_pa: Scalar,
    #[pyo3(get, set)]
    pub has_ignition_source: bool,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct SilEngineDynamics {
    #[pyo3(get, set)]
    pub state: EngineState,
    #[pyo3(get, set)]
    pub new_state: EngineState,

    #[pyo3(get, set)]
    pub fuel_inlet: Py<FluidConnection>,
    #[pyo3(get, set)]
    pub oxidizer_inlet: Py<FluidConnection>,

    #[pyo3(get, set)]
    pub allow_ignition: bool,

    pub fuel_injector: InjectorConfig,
    pub oxidizer_injector: InjectorConfig,
    pub combustion_data: CombustionData,
    pub throat_area_m2: Scalar,

    pub combustion_pressure_modifier: PyObject,

    test_t: f64,
}

#[pymethods]
impl SilEngineDynamics {
    #[new]
    pub fn new(
        py: Python,
        fuel_inlet: Py<FluidConnection>,
        oxidizer_inlet: Py<FluidConnection>,
        fuel_injector: &mut InjectorConfig,
        oxidizer_injector: &mut InjectorConfig,
        combustion_data: &mut CombustionData,
        throat_diameter_m: Scalar,
    ) -> Self {
        Self {
            state: EngineState::default(),
            new_state: EngineState::default(),
            fuel_inlet,
            oxidizer_inlet,
            fuel_injector: fuel_injector.clone(),
            oxidizer_injector: oxidizer_injector.clone(),
            allow_ignition: true,
            combustion_data: combustion_data.clone(),
            combustion_pressure_modifier: py.None(),
            throat_area_m2: throat_diameter_m.powi(2) * std::f64::consts::PI / 4.0,
            test_t: 0.0,
        }
    }

    fn post_update(&mut self) {
        self.state = self.new_state.clone();
    }

    fn update(&mut self, py: Python, dt: f64) {
        let dt = dt as Scalar;
        self.test_t += dt;

        let fuel_mass_flow_kg = self.calc_fuel_mass_flow_kg(dt, self.fuel_inlet.borrow(py));
        let oxidizer_mass_flow_kg =
            self.calc_oxidizer_mass_flow_kg(dt, self.oxidizer_inlet.borrow(py));

        let total_mass_flow_kg = fuel_mass_flow_kg + oxidizer_mass_flow_kg;

        let mut target_combustion_pressure_pa =
            if self.can_support_combustion(fuel_mass_flow_kg, oxidizer_mass_flow_kg) {
                calc_chamber_pressure(
                    total_mass_flow_kg / dt,
                    self.throat_area_m2,
                    &self.combustion_data,
                    ATMOSPHERIC_PRESSURE_PA,
                )
            } else {
                ATMOSPHERIC_PRESSURE_PA
            };

        if let Ok(result) = self
            .combustion_pressure_modifier
            .call1(py, (target_combustion_pressure_pa,))
        {
            if let Ok(pressure) = result.extract::<Scalar>(py) {
                target_combustion_pressure_pa = pressure;
            }
        }

        let delta = target_combustion_pressure_pa - self.state.chamber_pressure_pa;

        // if self.test_t % 0.1 < dt {
        //     println!("Target pressure: {}, Current pressure: {}", target_combustion_pressure_pa, self.state.chamber_pressure_pa);
        //     println!("\tFuel mass flow: {}, Oxidizer mass flow: {}", fuel_mass_flow_kg, oxidizer_mass_flow_kg);
        //     println!("\tFuel pressure: {}, Oxidizer pressure: {}", self.fuel_inlet.borrow(py).outlet_pressure_pa(), self.oxidizer_inlet.borrow(py).outlet_pressure_pa());
        // }

        self.new_state.chamber_pressure_pa += delta * 10.0 * dt;
        self.fuel_inlet
            .borrow_mut(py)
            .new_state
            .applied_outlet_pressure_pa = self.new_state.chamber_pressure_pa;
        self.oxidizer_inlet
            .borrow_mut(py)
            .new_state
            .applied_outlet_pressure_pa = self.new_state.chamber_pressure_pa;
    }

    pub fn set_combustion_pressure_modifier(&mut self, callback: PyObject) {
        self.combustion_pressure_modifier = callback;
    }

    #[getter]
    pub fn fuel_pressure_pa(&self, py: Python) -> f64 {
        self.fuel_inlet.borrow(py).outlet_pressure_pa()
    }

    #[getter]
    pub fn oxidizer_pressure_pa(&self, py: Python) -> f64 {
        self.oxidizer_inlet.borrow(py).outlet_pressure_pa()
    }

    #[getter]
    pub fn chamber_pressure_pa(&self) -> f64 {
        self.state.chamber_pressure_pa
    }
}

impl SilEngineDynamics {
    fn can_support_combustion(
        &self,
        fuel_mass_flow_kg: Scalar,
        oxidizer_mass_flow_kg: Scalar,
    ) -> bool {
        if !self.allow_ignition {
            return false;
        }

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

        if self.state.chamber_pressure_pa > MINIMUM_SUSTAINABLE_CHAMBER_PRESSURE_PA {
            // 30 PSI
            return true;
        }

        if self.state.has_ignition_source {
            return true;
        }

        false
    }

    fn calc_fuel_mass_flow_kg(&self, dt: Scalar, inlet: PyRef<'_, FluidConnection>) -> Scalar {
        if inlet.state.closed || inlet.outlet_pressure_pa() <= self.state.chamber_pressure_pa {
            return 0.0;
        }

        let mass_flow_rate_kg_s = self.fuel_injector.injector_area_m2()
            * self.fuel_injector.injector_orifice_cd
            * (2.0
                * self.fuel_injector.liquid.density_kg_m3
                * (inlet.outlet_pressure_pa() - self.state.chamber_pressure_pa))
                .sqrt();

        mass_flow_rate_kg_s * dt
    }

    fn calc_oxidizer_mass_flow_kg(&self, dt: Scalar, inlet: PyRef<'_, FluidConnection>) -> Scalar {
        if inlet.state.closed || inlet.outlet_pressure_pa() <= self.state.chamber_pressure_pa {
            return 0.0;
        }

        let mass_flow_rate_kg_s = self.oxidizer_injector.injector_area_m2()
            * self.oxidizer_injector.injector_orifice_cd
            * (2.0
                * self.oxidizer_injector.liquid.density_kg_m3
                * (inlet.outlet_pressure_pa() - self.state.chamber_pressure_pa))
                .sqrt();

        mass_flow_rate_kg_s * dt
    }
}
