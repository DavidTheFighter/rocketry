use pyo3::{prelude::*, PyClass, pyclass::boolean_struct::False};
use shared::ecu_hal::{EcuBinaryValve, EcuDriver};

use crate::{dynamics, ecu::EcuSil, mission_ctrl::MissionControl};

#[pyclass]
pub struct SilGlue {
    pub ecu: Option<Py<EcuSil>>,
    pub mission_ctrl: Option<Py<MissionControl>>,
    pub fuel_tank: Option<Py<dynamics::SilTankDynamics>>,
    pub oxidizer_tank: Option<Py<dynamics::SilTankDynamics>>,
    pub igniter: Option<Py<dynamics::igniter::SilIgniterDynamics>>,

    #[pyo3(get, set)]
    pub test_allow_igniter_ignition: bool,
}

#[pymethods]
impl SilGlue {
    #[new]
    pub fn new() -> Self {
        SilGlue {
            ecu: None,
            mission_ctrl: None,
            fuel_tank: None,
            oxidizer_tank: None,
            igniter: None,
            test_allow_igniter_ignition: true,
        }
    }

    pub fn update(&self, py: Python, _dt: f64) {
        if let Some(ecu) = borrow_py(py, &self.ecu) {
            if let Some(mut fuel_tank) = borrow_py(py, &self.fuel_tank) {
                fuel_tank.feed_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::FuelPress);
                fuel_tank.vent_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::FuelVent);
            }

            if let Some(mut oxidizer_tank) = borrow_py(py, &self.oxidizer_tank) {
                oxidizer_tank.feed_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::OxidizerPress);
                oxidizer_tank.vent_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::OxidizerVent);
            }

            if let Some(mut igniter) = borrow_py(py, &self.igniter) {
                igniter.fuel_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::IgniterFuelMain);
                igniter.oxidizer_valve_open = ecu._driver.borrow_mut().get_binary_valve(EcuBinaryValve::IgniterOxidizerMain);
                igniter.has_ignition_source = self.test_allow_igniter_ignition && ecu._driver.borrow_mut().get_sparking();
            }
        }

        if let Some(mut igniter) = borrow_py(py, &self.igniter) {
            if let Some(fuel_tank) = borrow_py(py, &self.fuel_tank) {
                igniter.fuel_pressure_pa = fuel_tank.tank_pressure_pa;
            }

            if let Some(oxidizer_tank) = borrow_py(py, &self.oxidizer_tank) {
                igniter.oxidizer_pressure_pa = oxidizer_tank.tank_pressure_pa;
            }
        }
    }

    pub fn set_from_self(&mut self, py: Python, cls: PyObject) {
        if let Ok(ecu) = cls.getattr(py, "ecu") {
            self.ecu = Some(ecu.extract(py).unwrap());
        }

        if let Ok(mission_ctrl) = cls.getattr(py, "mission_ctrl") {
            self.mission_ctrl = Some(mission_ctrl.extract(py).unwrap());
        }

        if let Ok(fuel_tank) = cls.getattr(py, "fuel_tank_dynamics") {
            self.fuel_tank = Some(fuel_tank.extract(py).unwrap());
        }

        if let Ok(oxidizer_tank) = cls.getattr(py, "oxidizer_tank_dynamics") {
            self.oxidizer_tank = Some(oxidizer_tank.extract(py).unwrap());
        }

        if let Ok(igniter) = cls.getattr(py, "igniter_dynamics") {
            self.igniter = Some(igniter.extract(py).unwrap());
        }
    }

    pub fn set_ecu(&mut self, ecu: Py<EcuSil>) {
        self.ecu = Some(ecu);
    }

    pub fn set_mission_ctrl(&mut self, mission_ctrl: Py<MissionControl>) {
        self.mission_ctrl = Some(mission_ctrl);
    }

    pub fn set_fuel_tank(&mut self, fuel_tank: Py<dynamics::SilTankDynamics>) {
        self.fuel_tank = Some(fuel_tank);
    }

    pub fn set_oxidizer_tank(&mut self, oxidizer_tank: Py<dynamics::SilTankDynamics>) {
        self.oxidizer_tank = Some(oxidizer_tank);
    }

    pub fn set_igniter(&mut self, igniter: Py<dynamics::igniter::SilIgniterDynamics>) {
        self.igniter = Some(igniter);
    }
}

fn borrow_py<'a, T: PyClass<Frozen = False>>(py: Python<'a>, obj: &'a Option<Py<T>>) -> Option<PyRefMut<'a, T>> {
    if let Some(obj) = obj {
        Some(obj.borrow_mut(py))
    } else {
        None
    }
}