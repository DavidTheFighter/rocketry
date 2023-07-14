pub mod driver;
pub mod dynamics;
pub mod logging;
pub mod ser;

use driver::FcuDriverSim;
use dynamics::Dynamics;
use flight_controller::Fcu;
use hal::comms_hal::{Packet, NetworkAddress};
use hal::fcu_hal::FcuTelemetryFrame;
use logging::Logger;
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fcu.update(dt, None);
    }

    pub fn update_acceleration(&mut self, accel: &PyList) {
        if accel.len() != 3 {
            panic!("acceleration must be a list of length 3");
        }

        self.fcu.update_acceleration(Vector3 {
            x: accel.get_item(0).unwrap().extract::<f32>().unwrap(),
            y: accel.get_item(1).unwrap().extract::<f32>().unwrap(),
            z: accel.get_item(2).unwrap().extract::<f32>().unwrap(),
        });
    }

    pub fn update_barometric_altitude(&mut self, altitude: f32) {
        self.fcu.update_barometric_pressure(altitude);
    }

    pub fn update_angular_velocity(&mut self, ang_vel: &PyList) {
        if ang_vel.len() != 3 {
            panic!("angular velocity must be a list of length 3");
        }

        self.fcu.update_angular_velocity(Vector3 {
            x: ang_vel.get_item(0).unwrap().extract::<f32>().unwrap(),
            y: ang_vel.get_item(1).unwrap().extract::<f32>().unwrap(),
            z: ang_vel.get_item(2).unwrap().extract::<f32>().unwrap(),
        });
    }

    pub fn update_gps(&mut self, gps: &PyList) {
        if gps.len() != 3 {
            panic!("gps must be a list of length 3");
        }

        self.fcu.update_gps(Vector3 {
            x: gps.get_item(0).unwrap().extract::<f32>().unwrap(),
            y: gps.get_item(1).unwrap().extract::<f32>().unwrap(),
            z: gps.get_item(2).unwrap().extract::<f32>().unwrap(),
        });
    }

    pub fn get_fcu_position(&self, py: Python) -> PyResult<PyObject> {
        let pos = self.fcu.state_vector.get_position();
        Ok(vec3_to_list(py, pos).into())
    }

    pub fn get_fcu_velocity(&self, py: Python) -> PyResult<PyObject> {
        let vel = self.fcu.state_vector.get_velocity();
        Ok(vec3_to_list(py, vel).into())
    }

    pub fn get_fcu_acceleration(&self, py: Python) -> PyResult<PyObject> {
        let accel = self.fcu.state_vector.get_acceleration();
        Ok(vec3_to_list(py, accel).into())
    }

    pub fn get_fcu_orientation(&self, py: Python) -> PyResult<PyObject> {
        let orientation = self.fcu.state_vector.get_orientation();
        Ok(quat_to_list(py, orientation).into())
    }

    pub fn get_fcu_angular_velocity(&self, py: Python) -> PyResult<PyObject> {
        let ang_vel = self.fcu.state_vector.get_angular_velocity();
        Ok(vec3_to_list(py, ang_vel).into())
    }

    pub fn get_fcu_angular_acceleration(&self, py: Python) -> PyResult<PyObject> {
        let ang_accel = self.fcu.state_vector.get_angular_acceleration();
        Ok(vec3_to_list(py, ang_accel).into())
    }

    pub fn get_fcu_telemetry(&mut self, py: Python) -> PyResult<PyObject> {
        let driver = self.fcu.driver.as_mut_any().downcast_mut::<FcuDriverSim>().unwrap();
        let telem = &driver.last_telem_packet;

        if telem.is_none() {
            return Ok(PyDict::new(py).into());
        }

        Ok(dict_from_obj(py, telem.as_ref().unwrap()).into())
    }

    pub fn update_fcu_config(&mut self, dict: &PyDict) {
        let config = obj_from_dict(dict);

        println!("Updating config: {:?}", config);

        self.fcu.configure_fcu(config);
    }

    pub fn get_fcu_detailed_state(&mut self, py: Python) -> PyResult<PyObject> {
        let state = self.fcu.generate_detailed_state_frame();

        Ok(dict_from_obj(py, state).into())
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
    Ok(())
}

fn vec3_to_list(py: Python, vec: Vector3<f32>) -> PyObject {
    let list = PyList::new(py, &[vec.x, vec.y, vec.z]);
    list.into()
}

fn quat_to_list(py: Python, quat: mint::Quaternion<f32>) -> PyObject {
    let list = PyList::new(py, &[quat.s, quat.v.x, quat.v.y, quat.v.z]);
    list.into()
}