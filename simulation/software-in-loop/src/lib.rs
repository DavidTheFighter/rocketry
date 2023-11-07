pub mod driver;
pub mod dynamics;
pub mod logging;
pub mod ser;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Mutex, Arc};

use driver::FcuDriverSim;
use dynamics::Dynamics;
use flight_controller_rs::Fcu;
use shared::comms_hal::{Packet, NetworkAddress};
use shared::fcu_hal::{FcuTelemetryFrame, FcuDevStatsFrame, FcuSensorData, FcuDriver};
use logging::{Logger, load_logs_from_file};
use mint::Vector3;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyDict};
use ser::{dict_from_obj, obj_from_dict};

#[pyclass(unsendable)]
pub struct SoftwareInLoop {
    #[pyo3(get)]
    name: String,
    driver: Rc<RefCell<FcuDriverSim>>,
    fcu: Fcu<'static>,
    pending_packets: Vec<(NetworkAddress, Packet)>,
}

#[pymethods]
impl SoftwareInLoop {
    #[new]
    pub fn new() -> Self {
        let driver = Rc::new(RefCell::new(FcuDriverSim::new()));
        let driver_ref: &'static mut FcuDriverSim = unsafe {
            std::mem::transmute(&mut *driver.borrow_mut())
        };

        Self {
            name: "FCU".to_string(),
            driver,
            fcu: Fcu::new(driver_ref),
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

    pub fn reset_telemetry(&mut self) {
        self.fcu.driver.send_packet(
            Packet::FcuTelemetry(FcuTelemetryFrame::default()),
            NetworkAddress::MissionControl,
        );
    }

    // Returns general and widely needed fields from the FCU
    fn __getitem__(&self, key: &str, py: Python) -> PyResult<PyObject> {
        let debug_info = self.fcu.generate_debug_info();
        let dict = dict_from_obj(py, &debug_info);

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
    m.add_class::<SoftwareInLoop>()?;
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