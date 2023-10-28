pub mod driver;
pub mod dynamics;
pub mod logging;
pub mod ser;

use driver::FcuDriverSim;
use dynamics::Dynamics;
use flight_controller::Fcu;
use shared::comms_hal::{Packet, NetworkAddress};
use shared::fcu_hal::{FcuTelemetryFrame, FcuDevStatsFrame, FcuSensorData};
use logging::{Logger, load_logs_from_file};
use mint::Vector3;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyDict};
use ser::{dict_from_obj, obj_from_dict};

static mut MOCK: FcuDriverSim = FcuDriverSim::new();

#[pyclass]
pub struct SoftwareInLoop {
    #[pyo3(get)]
    name: String,
    fcu: Fcu<'static>,
    pending_packets: Vec<(NetworkAddress, Packet)>,
}

#[pymethods]
impl SoftwareInLoop {
    #[new]
    pub fn new() -> Self {
        let mock = unsafe { &mut MOCK };
        mock.init();
        Self {
            name: "FCU".to_string(),
            fcu: Fcu::new(mock),
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

        self.fcu.configure_fcu(config);
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
}

#[pymodule]
fn software_in_loop(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SoftwareInLoop>()?;
    m.add_class::<Dynamics>()?;
    m.add_class::<Logger>()?;
    m.add_function(wrap_pyfunction!(load_logs_from_file, m)?)?;
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