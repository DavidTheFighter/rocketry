use std::{rc::Rc, cell::RefCell};

use big_brother::{interface::{mock_interface::MockInterface, BigBrotherInterface}, big_brother::MAX_INTERFACE_COUNT};
use ecu_rs::{Ecu, ecu::EcuBigBrother};
use pyo3::{prelude::*, types::{PyList, PyDict}};
use shared::{comms_hal::NetworkAddress, ecu_hal::{EcuBinaryOutput, EcuSensor}, PressureData};
use strum::IntoEnumIterator;

use crate::{network::SilNetworkIface, ser::{obj_from_dict, dict_from_obj}};

use super::driver::EcuDriverSil;

#[pyclass(unsendable)]
pub struct EcuSil {
    pub(crate) _driver: Rc<RefCell<EcuDriverSil>>,
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<EcuBigBrother<'static>>>,
    pub(crate) ecu: Ecu<'static>,
}

#[pymethods]
impl EcuSil {
    #[new]
    pub fn new(network_ifaces: &PyList, ecu_index: u8) -> Self {
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.ecu.update(dt);
    }

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

    pub fn update_fuel_tank_pressure(&mut self, pressure_pa: f32) {
        self.ecu.update_sensor_data(&EcuSensor::FuelTankPressure(PressureData {
            pressure_pa,
            raw_data: 0,
        }));
    }

    pub fn update_oxidizer_tank_pressure(&mut self, pressure_pa: f32) {
        self.ecu.update_sensor_data(&EcuSensor::OxidizerTankPressure(PressureData {
            pressure_pa,
            raw_data: 0,
        }));
    }

    pub fn update_igniter_chamber_pressure(&mut self, pressure_pa: f32) {
        self.ecu.update_sensor_data(&EcuSensor::IgniterChamberPressure(PressureData {
            pressure_pa,
            raw_data: 0,
        }));
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

        dict.get_item_with_error(key)
            .map(|value| value.to_object(py))
    }
}