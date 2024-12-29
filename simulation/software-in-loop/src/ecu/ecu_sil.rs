use std::{cell::RefCell, collections::HashMap, rc::Rc};

use big_brother::{
    big_brother::MAX_INTERFACE_COUNT,
    interface::{mock_interface::MockInterface, BigBrotherInterface},
};
use ecu_rs::{ecu::EcuBigBrother, Ecu};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use shared::{
    comms_hal::NetworkAddress,
    ecu_hal::{EcuBinaryOutput, EcuLinearOutput, EcuSensor},
};
use strum::IntoEnumIterator;

use crate::{
    dynamics::{
        engine::SilEngineDynamics, igniter::SilIgniterDynamics, pump::SilPumpDynamics,
        SilTankDynamics, ATMOSPHERIC_PRESSURE_PA,
    },
    network::SilNetworkIface,
    sensors::SensorNoise,
    ser::{dict_from_obj, obj_from_dict},
};

use super::{driver::EcuDriverSil, sensors::initialize_sensors};

#[pyclass(unsendable)]
pub struct EcuSil {
    pub(crate) _driver: Rc<RefCell<EcuDriverSil>>,
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<EcuBigBrother<'static>>>,
    pub(crate) ecu: Ecu<'static>,
    pub(crate) sensors: HashMap<EcuSensor, Box<dyn SensorNoise>>,
    time_since_last_ecu_update: f64,
    ecu_update_interval: f64,
    fuel_tank: Option<Py<SilTankDynamics>>,
    oxidizer_tank: Option<Py<SilTankDynamics>>,
    engine: Option<Py<SilEngineDynamics>>,
    igniter: Option<Py<SilIgniterDynamics>>,
    fuel_pump: Option<Py<SilPumpDynamics>>,
    oxidizer_pump: Option<Py<SilPumpDynamics>>,
}

#[pymethods]
impl EcuSil {
    #[new]
    pub fn new(
        network_ifaces: &PyList,
        ecu_index: u8,
        sensor_configuration: &PyDict,
        ecu_update_interval: f64,
        fuel_tank: Option<Py<SilTankDynamics>>,
        oxidizer_tank: Option<Py<SilTankDynamics>>,
        engine: Option<Py<SilEngineDynamics>>,
        igniter: Option<Py<SilIgniterDynamics>>,
        fuel_pump: Option<Py<SilPumpDynamics>>,
        oxidizer_pump: Option<Py<SilPumpDynamics>>,
    ) -> Self {
        let driver = Rc::new(RefCell::new(EcuDriverSil::new()));
        let driver_ref: &'static mut EcuDriverSil =
            unsafe { std::mem::transmute(&mut *driver.borrow_mut()) };

        let mut big_brother_ifaces = [None, None];
        let mut big_brother_ifaces_ref: [Option<&'static mut dyn BigBrotherInterface>;
            MAX_INTERFACE_COUNT] = [None, None];

        for (i, sil_iface) in network_ifaces.iter().enumerate().take(2) {
            let mut sil_iface = sil_iface
                .extract::<PyRefMut<'_, SilNetworkIface>>()
                .expect("Failed to extract interface");

            big_brother_ifaces[i].replace(Rc::new(RefCell::new(
                sil_iface.iface.take().expect("Failed to take interface"),
            )));

            let bb_iface_ref: &'static mut MockInterface = unsafe {
                std::mem::transmute(&mut *big_brother_ifaces[i].as_ref().unwrap().borrow_mut())
            };

            big_brother_ifaces_ref[i] = Some(bb_iface_ref);
        }

        let big_brother = Rc::new(RefCell::new(EcuBigBrother::new(
            NetworkAddress::EngineController(ecu_index),
            rand::random(),
            NetworkAddress::Broadcast,
            big_brother_ifaces_ref,
        )));
        let big_brother_ref: &'static mut EcuBigBrother<'static> =
            unsafe { std::mem::transmute(&mut *big_brother.borrow_mut()) };

        let ecu = Ecu::new(driver_ref, big_brother_ref);

        Self {
            _driver: driver,
            _big_brother_ifaces: big_brother_ifaces,
            _big_brother: big_brother,
            ecu,
            sensors: initialize_sensors(sensor_configuration),
            time_since_last_ecu_update: 0.0,
            ecu_update_interval,
            fuel_tank,
            oxidizer_tank,
            engine,
            igniter,
            fuel_pump,
            oxidizer_pump,
        }
    }

    pub fn update(&mut self, py: Python, dt: f64) {
        self.time_since_last_ecu_update += dt;

        if self.time_since_last_ecu_update >= self.ecu_update_interval {
            self.update_ecu_loop(py, self.ecu_update_interval);
            self.time_since_last_ecu_update -= self.ecu_update_interval;
        }

        if self.time_since_last_ecu_update >= self.ecu_update_interval {
            eprintln!(
                "Too few updates per frame, skipping update {} {} {}",
                self.time_since_last_ecu_update, dt, self.ecu_update_interval
            );
        }
    }

    pub fn post_update(&mut self) {}

    pub fn update_ecu_config(&mut self, dict: &PyDict) {
        let config = obj_from_dict(dict);

        println!("Updating ECU config: {:?}", config);

        self.ecu.configure_ecu(config);
    }

    pub fn update_timestamp(&mut self, sim_time: f32) {
        self.ecu
            .driver
            .as_mut_any()
            .downcast_mut::<EcuDriverSil>()
            .expect("Failed to retrieve driver from ECU object")
            .update_timestamp(sim_time);
    }

    fn update_ecu_loop(&mut self, py: Python, dt: f64) {
        self.ecu.update(dt as f32);
        self.update_sensors(py, dt);

        if let Some(engine) = self.engine.as_ref() {
            let mut engine = engine.borrow_mut(py);
            engine.fuel_inlet.borrow_mut(py).new_state.closed = !self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::EngineFuelValve);
            engine.oxidizer_inlet.borrow_mut(py).new_state.closed = !self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::EngineOxidizerValve);

            if let Some(igniter) = self.igniter.as_ref() {
                let igniter = igniter.borrow_mut(py);
                engine.new_state.has_ignition_source =
                    igniter.chamber_pressure_pa() > ATMOSPHERIC_PRESSURE_PA;
            }
        }

        if let Some(igniter) = self.igniter.as_ref() {
            let mut igniter = igniter.borrow_mut(py);
            igniter.new_state.has_ignition_source = self.ecu.driver.get_sparking();
            igniter.fuel_inlet.borrow_mut(py).new_state.closed = !self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::IgniterFuelValve);
            igniter.oxidizer_inlet.borrow_mut(py).new_state.closed = !self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::IgniterOxidizerValve);
        }

        if let Some(fuel_tank) = self.fuel_tank.as_ref() {
            let mut fuel_tank = fuel_tank.borrow_mut(py);
            fuel_tank.new_state.press_valve_open = self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::FuelPressValve);
            fuel_tank.new_state.vent_valve_open = self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::FuelVentValve);
        }

        if let Some(oxidizer_tank) = self.oxidizer_tank.as_ref() {
            let mut oxidizer_tank = oxidizer_tank.borrow_mut(py);
            oxidizer_tank.new_state.press_valve_open = self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::OxidizerPressValve);
            oxidizer_tank.new_state.vent_valve_open = self
                .ecu
                .driver
                .get_binary_valve(EcuBinaryOutput::OxidizerVentValve);
        }

        if let Some(fuel_pump) = self.fuel_pump.as_ref() {
            fuel_pump.borrow_mut(py).new_state.motor_duty_cycle =
                self.ecu.driver.get_linear_output(EcuLinearOutput::FuelPump) as f64;
        }

        if let Some(oxidizer_pump) = self.oxidizer_pump.as_ref() {
            oxidizer_pump.borrow_mut(py).new_state.motor_duty_cycle =
                self.ecu
                    .driver
                    .get_linear_output(EcuLinearOutput::OxidizerPump) as f64;
        }
    }

    // Returns general and widely needed fields from the FCU
    fn __getitem__(&self, key: &str, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);

        let debug_info_callback = |debug_info_variant| {
            let debug_info_dict = dict_from_obj(py, &debug_info_variant);

            for value in debug_info_dict.values() {
                for (key, value) in value.downcast::<PyDict>().unwrap().iter() {
                    dict.set_item(key, value)
                        .expect("Failed to set item in dict");
                }
            }
        };
        self.ecu
            .generate_debug_info_all_variants(debug_info_callback);

        let binary_valves = PyDict::new(py);
        for valve in EcuBinaryOutput::iter() {
            binary_valves.set_item(
                format!("{:?}", valve),
                self.ecu.driver.get_binary_valve(valve),
            )?;
        }
        dict.set_item("binary_valves", binary_valves)?;

        let sensors = PyDict::new(py);
        for sensor in EcuSensor::iter() {
            sensors.set_item(
                format!("{:?}", sensor),
                self.get_direct_sensor_value(py, sensor),
            )?;
        }
        dict.set_item("sensors", sensors)?;

        dict.get_item_with_error(key)
            .map(|value| value.to_object(py))
    }
}

impl EcuSil {
    pub fn get_direct_sensor_value(&self, py: Python, sensor: EcuSensor) -> f64 {
        match sensor {
            EcuSensor::FuelTankPressure => self
                .fuel_tank
                .as_ref()
                .map(|tank| tank.borrow(py).tank_pressure_pa() as f64)
                .unwrap_or(0.0),
            EcuSensor::OxidizerTankPressure => self
                .oxidizer_tank
                .as_ref()
                .map(|tank| tank.borrow(py).tank_pressure_pa() as f64)
                .unwrap_or(0.0),
            EcuSensor::IgniterChamberPressure => self
                .igniter
                .as_ref()
                .map(|igniter| igniter.borrow(py).chamber_pressure_pa() as f64)
                .unwrap_or(0.0),
            EcuSensor::IgniterFuelInjectorPressure => self
                .igniter
                .as_ref()
                .map(|igniter| {
                    igniter
                        .borrow(py)
                        .fuel_inlet
                        .borrow(py)
                        .outlet_pressure_pa() as f64
                })
                .unwrap_or(0.0),
            EcuSensor::IgniterOxidizerInjectorPressure => self
                .igniter
                .as_ref()
                .map(|igniter| {
                    igniter
                        .borrow(py)
                        .oxidizer_inlet
                        .borrow(py)
                        .outlet_pressure_pa() as f64
                })
                .unwrap_or(0.0),
            EcuSensor::IgniterThroatTemperature => 0.0,
            EcuSensor::EngineChamberPressure => self
                .engine
                .as_ref()
                .map(|engine| engine.borrow(py).chamber_pressure_pa() as f64)
                .unwrap_or(0.0),
            EcuSensor::EngineFuelInjectorPressure => self
                .engine
                .as_ref()
                .map(|engine| engine.borrow(py).fuel_inlet.borrow(py).outlet_pressure_pa() as f64)
                .unwrap_or(0.0),
            EcuSensor::EngineOxidizerInjectorPressure => self
                .engine
                .as_ref()
                .map(|engine| {
                    engine
                        .borrow(py)
                        .oxidizer_inlet
                        .borrow(py)
                        .outlet_pressure_pa() as f64
                })
                .unwrap_or(0.0),
            EcuSensor::EngineThroatTemperature => 0.0,
            EcuSensor::FuelPumpOutletPressure => self
                .fuel_pump
                .as_ref()
                .map(|pump| pump.borrow(py).state.pressure_pa as f64)
                .unwrap_or(0.0),
            EcuSensor::FuelPumpInletPressure => 0.0,
            EcuSensor::FuelPumpInducerPressure => 0.0,
            EcuSensor::OxidizerPumpOutletPressure => self
                .oxidizer_pump
                .as_ref()
                .map(|pump| pump.borrow(py).state.pressure_pa as f64)
                .unwrap_or(0.0),
            EcuSensor::OxidizerPumpInletPressure => 0.0,
            EcuSensor::OxidizerPumpInducerPressure => 0.0,
        }
    }
}
