pub mod driver;
pub mod dynamics;
pub mod logging;
pub mod ser;

use std::str::FromStr;
use std::cell::RefCell;
use std::rc::Rc;

use driver::FcuDriverSim;
use dynamics::Dynamics;
use flight_controller_rs::Fcu;
use shared::comms_hal::{Packet, NetworkAddress};
use shared::fcu_hal::{self, FcuTelemetryFrame, FcuSensorData, OutputChannel};
use logging::{Logger, load_logs_from_file};
use mint::Vector3;
use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyList, PyDict};
use ser::{dict_from_obj, obj_from_dict};
use shared::logger::DataPointLoggerMock;
use strum::IntoEnumIterator;

#[pyclass(unsendable)]
pub struct FcuSil {
    #[pyo3(get)]
    name: String,
    _driver: Rc<RefCell<FcuDriverSim>>,
    _data_point_logger: Rc<RefCell<DataPointLoggerMock>>,
    fcu: Fcu<'static>,
    pending_packets: Vec<(NetworkAddress, Packet)>,
}

#[pymethods]
impl FcuSil {
    #[new]
    pub fn new() -> Self {
        let driver = Rc::new(RefCell::new(FcuDriverSim::new()));
        let driver_ref: &'static mut FcuDriverSim = unsafe {
            std::mem::transmute(&mut *driver.borrow_mut())
        };

        let data_point_logger = Rc::new(RefCell::new(DataPointLoggerMock));
        let data_point_logger_ref: &'static mut DataPointLoggerMock = unsafe {
            std::mem::transmute(&mut *data_point_logger.borrow_mut())
        };

        let fcu = Fcu::new(driver_ref, data_point_logger_ref);

        Self {
            name: "FCU".to_string(),
            _driver: driver,
            _data_point_logger: data_point_logger,
            fcu,
            pending_packets: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fcu.update(dt, &self.pending_packets);
        self.pending_packets.clear();
    }

    pub fn update_acceleration(&mut self, accel: &PyList) {
        self.fcu.update_sensor_data(FcuSensorData::Accelerometer {
            acceleration: list_to_vec3(accel),
            raw_data: Vector3 { x: 42, y: 42, z: 42 },
        });
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: &PyList) {
        self.fcu.update_sensor_data(FcuSensorData::Gyroscope{
            angular_velocity: list_to_vec3(angular_velocity),
            raw_data: Vector3 { x: 42, y: 42, z: 42 },
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

        self.pending_packets.push((NetworkAddress::MissionControl, Packet::ConfigureFcu(config)));
    }

    pub fn fcu_config(&self, py: Python) -> PyResult<PyObject> {
        Ok(dict_from_obj(py, &self.fcu.get_fcu_config()).into())
    }

    pub fn start_dev_stats_frame(&mut self) {
        self.pending_packets.push((NetworkAddress::MissionControl, Packet::StartDevStatsFrame));
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
            .ok_or(PyTypeError::new_err("Failed to retrieve driver from FCU object"))?
            .set_output_channel_continuity(channel, state);

        Ok(())
    }

    pub fn reset_telemetry(&mut self) {
        self.fcu.driver.send_packet(
            Packet::FcuTelemetry(FcuTelemetryFrame::default()),
            NetworkAddress::MissionControl,
        );
    }

    pub fn send_arm_vehicle_packet(&mut self) {
        self.pending_packets.push((NetworkAddress::MissionControl, Packet::ArmVehicle { magic_number: fcu_hal::ARMING_MAGIC_NUMBER }));
    }

    pub fn send_ignite_solid_motor_packet(&mut self) {
        self.pending_packets.push((NetworkAddress::MissionControl, Packet::IgniteSolidMotor { magic_number: fcu_hal::IGNITION_MAGIC_NUMBER }));
    }

    // Returns general and widely needed fields from the FCU
    fn __getitem__(&self, key: &str, py: Python) -> PyResult<PyObject> {
        let debug_info = self.fcu.generate_debug_info();
        let dict = dict_from_obj(py, &debug_info);

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

        dict.get_item_with_error(key).map(|value| value.to_object(py))
    }

    // Returns fields related to the state vector of the FCU
    fn state_vector(&self, py: Python) -> PyResult<PyObject> {
        let dict = dict_from_obj(py, &self.fcu.state_vector);

        Ok(dict.into())
    }
}

#[pyfunction]
fn convert_altitude_to_pressure(altitude: f32, temperature: f32) -> f32 {
    shared::standard_atmosphere::convert_altitude_to_pressure(altitude, temperature)
}

#[pyfunction]
fn convert_pressure_to_altitude(pressure: f32, temperature: f32) -> f32 {
    shared::standard_atmosphere::convert_pressure_to_altitude(pressure, temperature)
}

#[pymodule]
fn software_in_loop(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<FcuSil>()?;
    m.add_class::<Dynamics>()?;
    m.add_class::<Logger>()?;
    m.add_function(wrap_pyfunction!(load_logs_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(convert_altitude_to_pressure, m)?)?;
    m.add_function(wrap_pyfunction!(convert_pressure_to_altitude, m)?)?;
    Ok(())
}

// fn vec3_to_list(py: Python, vec: nalgebra::Vector3<f32>) -> PyObject {
//     let list = PyList::new(py, &[vec.x, vec.y, vec.z]);
//     list.into()
// }

fn list_to_vec3(list: &PyList) -> Vector3<f32> {
    if list.len() != 3 {
        panic!("Tried converting a pylist of len() != 3 to a vec3: {:?}", list);
    }

    Vector3 {
        x: list.get_item(0).unwrap().extract::<f32>().expect(".x was not a number"),
        y: list.get_item(1).unwrap().extract::<f32>().expect(".y was not a number"),
        z: list.get_item(2).unwrap().extract::<f32>().expect(".z was not a number"),
    }
}

// fn quat_to_list(py: Python, quat: nalgebra::UnitQuaternion<f32>) -> PyObject {
//     let list = PyList::new(py, &[quat.w, quat.i, quat.j, quat.k]);
//     list.into()
// }