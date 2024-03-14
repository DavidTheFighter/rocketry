use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use super::FcuDriverSim;
use crate::network::SilNetworkIface;
use crate::ser::{dict_from_obj, obj_from_dict};
use big_brother::big_brother::MAX_INTERFACE_COUNT;
use big_brother::interface::mock_interface::MockInterface;
use big_brother::interface::BigBrotherInterface;
use fcu_rs::{Fcu, FcuBigBrother};
use mint::Vector3;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use shared::comms_hal::NetworkAddress;
use shared::fcu_hal::{FcuSensorData, OutputChannel};
use shared::logger::DataPointLoggerMock;
use strum::IntoEnumIterator;

#[pyclass(unsendable)]
pub struct FcuSil {
    #[pyo3(get)]
    name: String,
    pub(crate) _driver: Rc<RefCell<FcuDriverSim>>,
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<FcuBigBrother<'static>>>,
    pub(crate) _data_point_logger: Rc<RefCell<DataPointLoggerMock>>,
    pub(crate) fcu: Fcu<'static>,
}

#[pymethods]
impl FcuSil {
    #[new]
    pub fn new(network_ifaces: &PyList) -> Self {
        let driver = Rc::new(RefCell::new(FcuDriverSim::new()));
        let driver_ref: &'static mut FcuDriverSim =
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

        let big_brother = Rc::new(RefCell::new(FcuBigBrother::new(
            NetworkAddress::FlightController,
            rand::random(),
            NetworkAddress::Broadcast,
            big_brother_ifaces_ref,
        )));
        let big_brother_ref: &'static mut FcuBigBrother<'static> =
            unsafe { std::mem::transmute(&mut *big_brother.borrow_mut()) };

        let data_point_logger = Rc::new(RefCell::new(DataPointLoggerMock));
        let data_point_logger_ref: &'static mut DataPointLoggerMock =
            unsafe { std::mem::transmute(&mut *data_point_logger.borrow_mut()) };

        let fcu = Fcu::new(driver_ref, big_brother_ref, data_point_logger_ref);

        Self {
            name: "FCU".to_string(),
            _driver: driver,
            _big_brother_ifaces: big_brother_ifaces,
            _big_brother: big_brother,
            _data_point_logger: data_point_logger,
            fcu,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fcu.update(dt);
    }

    pub fn update_acceleration(&mut self, accel: &PyList) {
        self.fcu.update_sensor_data(FcuSensorData::Accelerometer {
            acceleration: list_to_vec3(accel),
            raw_data: Vector3 {
                x: 42,
                y: 42,
                z: 42,
            },
        });
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: &PyList) {
        self.fcu.update_sensor_data(FcuSensorData::Gyroscope {
            angular_velocity: list_to_vec3(angular_velocity),
            raw_data: Vector3 {
                x: 42,
                y: 42,
                z: 42,
            },
        });
    }

    pub fn update_barometric_altitude(&mut self, altitude: f32) {
        let pressure = shared::standard_atmosphere::convert_altitude_to_pressure(altitude, 20.0);
        self.fcu.update_sensor_data(FcuSensorData::Barometer {
            pressure: pressure,
            temperature: 20.0,
            raw_data: (pressure * 100.0) as u32,
        });
    }

    pub fn update_gps(&mut self, _gps: &PyList) {
        // TOOD
    }

    pub fn update_fcu_config(&mut self, dict: &PyDict) {
        let config = obj_from_dict(dict);

        println!("Updating config: {:?}", config);

        self.fcu.configure_fcu(config);
    }

    pub fn fcu_config(&self, py: Python) -> PyResult<PyObject> {
        Ok(dict_from_obj(py, &self.fcu.get_fcu_config()).into())
    }

    pub fn start_dev_stats_frame(&mut self) {
        // self.send_packet(NetworkAddress::MissionControl, &Packet::StartDevStatsFrame);
        // self.pending_packets.push((NetworkAddress::MissionControl, Packet::StartDevStatsFrame));
    }

    pub fn update_timestamp(&mut self, sim_time: f32) {
        self.fcu
            .driver
            .as_mut_any()
            .downcast_mut::<FcuDriverSim>()
            .expect("Failed to retrieve driver from FCU object")
            .update_timestamp(sim_time);
    }

    pub fn set_output(&mut self, channel: &str, state: bool) -> PyResult<()> {
        let channel = OutputChannel::from_str(channel)
            .map_err(|_| PyTypeError::new_err("Failed to parse output channel string"))?;

        self.fcu.driver.set_output_channel(channel, state);

        Ok(())
    }

    pub fn set_output_continuity(&mut self, channel: &str, state: bool) -> PyResult<()> {
        let channel = OutputChannel::from_str(channel)
            .map_err(|_| PyTypeError::new_err("Failed to parse output channel string"))?;

        self.fcu
            .driver
            .as_mut_any()
            .downcast_mut::<FcuDriverSim>()
            .ok_or(PyTypeError::new_err(
                "Failed to retrieve driver from FCU object",
            ))?
            .set_output_channel_continuity(channel, state);

        Ok(())
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
        self.fcu
            .generate_debug_info_all_variants(debug_info_callback);

        let output_channels = PyDict::new(py);
        for channel in OutputChannel::iter() {
            output_channels.set_item(
                format!("{:?}", channel),
                self.fcu.driver.get_output_channel(channel),
            )?;
        }
        dict.set_item("outputs", output_channels)?;

        let output_channel_continuities = PyDict::new(py);
        for channel in OutputChannel::iter() {
            output_channel_continuities.set_item(
                format!("{:?}", channel),
                self.fcu.driver.get_output_channel_continuity(channel),
            )?;
        }
        dict.set_item("output_continuities", output_channel_continuities)?;

        dict.get_item_with_error(key)
            .map(|value| value.to_object(py))
    }

    // Returns fields related to the state vector of the FCU
    pub(crate) fn state_vector(&self, py: Python) -> PyResult<PyObject> {
        let dict = dict_from_obj(py, &self.fcu.state_vector);

        Ok(dict.into())
    }
}

#[pyfunction]
pub fn convert_altitude_to_pressure(altitude: f32, temperature: f32) -> f32 {
    shared::standard_atmosphere::convert_altitude_to_pressure(altitude, temperature)
}

#[pyfunction]
pub fn convert_pressure_to_altitude(pressure: f32, temperature: f32) -> f32 {
    shared::standard_atmosphere::convert_pressure_to_altitude(pressure, temperature)
}

// fn vec3_to_list(py: Python, vec: nalgebra::Vector3<f32>) -> PyObject {
//     let list = PyList::new(py, &[vec.x, vec.y, vec.z]);
//     list.into()
// }

fn list_to_vec3(list: &PyList) -> Vector3<f32> {
    if list.len() != 3 {
        panic!(
            "Tried converting a pylist of len() != 3 to a vec3: {:?}",
            list
        );
    }

    Vector3 {
        x: list
            .get_item(0)
            .unwrap()
            .extract::<f32>()
            .expect(".x was not a number"),
        y: list
            .get_item(1)
            .unwrap()
            .extract::<f32>()
            .expect(".y was not a number"),
        z: list
            .get_item(2)
            .unwrap()
            .extract::<f32>()
            .expect(".z was not a number"),
    }
}

// fn quat_to_list(py: Python, quat: nalgebra::UnitQuaternion<f32>) -> PyObject {
//     let list = PyList::new(py, &[quat.w, quat.i, quat.j, quat.k]);
//     list.into()
// }
