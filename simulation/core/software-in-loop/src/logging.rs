use hal::fcu_hal::{FcuTelemetryFrame, FcuDetailedStateFrame};
use pyo3::{prelude::*, types::PyDict};
use serde::{Serialize, Deserialize};
use std::io::Write;

use crate::{SoftwareInLoop, driver::FcuDriverSim, ser::dict_from_obj};

type Scalar = f64;

#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
pub struct Logger {
    #[pyo3(get, set)]
    pub dt: Scalar,
    pub telemetry: Vec<FcuTelemetryFrame>,
    pub detailed_state: Vec<FcuDetailedStateFrame>,
    #[pyo3(get, set)]
    pub position: Vec<Vec<Scalar>>,
    #[pyo3(get, set)]
    pub velocity: Vec<Vec<Scalar>>,
    #[pyo3(get, set)]
    pub acceleration: Vec<Vec<Scalar>>,
    #[pyo3(get, set)]
    pub orientation: Vec<Vec<Scalar>>,
    #[pyo3(get, set)]
    pub angular_velocity: Vec<Vec<Scalar>>,
    #[pyo3(get, set)]
    pub angular_acceleration: Vec<Vec<Scalar>>,
}

#[pymethods]
impl Logger {
    #[new]
    pub fn new() -> Self {
        Self {
            dt: 0.0,
            telemetry: Vec::new(),
            detailed_state: Vec::new(),
            position: Vec::new(),
            velocity: Vec::new(),
            acceleration: Vec::new(),
            orientation: Vec::new(),
            angular_velocity: Vec::new(),
            angular_acceleration: Vec::new(),
        }
    }

    pub fn log_telemetry(&mut self, fcu: &mut SoftwareInLoop) {
        let driver = fcu
            .fcu
            .driver
            .as_mut_any()
            .downcast_mut::<FcuDriverSim>()
            .expect("Failed to retrieve driver from FCU object");

        if let Some(packet) = &driver.last_telem_packet {
            self.telemetry.push(packet.clone());
        }
    }

    pub fn log_detailed_state(&mut self, fcu: &mut SoftwareInLoop) {
        let state = fcu.fcu.generate_detailed_state_frame();

        self.detailed_state.push(state);
    }

    pub fn log_position(&mut self, vec: Vec<Scalar>) {
        self.position.push(vec);
    }

    pub fn log_velocity(&mut self, vec: Vec<Scalar>) {
        self.velocity.push(vec);
    }

    pub fn log_acceleration(&mut self, vec: Vec<Scalar>) {
        self.acceleration.push(vec);
    }

    pub fn log_orientation(&mut self, vec: Vec<Scalar>) {
        self.orientation.push(vec);
    }

    pub fn log_angular_velocity(&mut self, vec: Vec<Scalar>) {
        self.angular_velocity.push(vec);
    }

    pub fn log_angular_acceleration(&mut self, vec: Vec<Scalar>) {
        self.angular_acceleration.push(vec);
    }

    pub fn grab_timestep_frame(&self, py: Python, i: usize) -> PyResult<PyObject> {
        let dict = PyDict::new(py);

        dict.set_item("position", self.position[i].clone())?;
        dict.set_item("velocity", self.velocity[i].clone())?;
        dict.set_item("acceleration", self.acceleration[i].clone())?;
        dict.set_item("orientation", self.orientation[i].clone())?;
        dict.set_item("angular_velocity", self.angular_velocity[i].clone())?;
        dict.set_item("angular_acceleration", self.angular_acceleration[i].clone())?;

        dict.set_item("telemetry", dict_from_obj(py, &self.telemetry[i]))?;
        dict.set_item("detailed_state", dict_from_obj(py, &self.detailed_state[i]))?;

        Ok(dict.into())
    }

    pub fn num_timesteps(&self) -> PyResult<usize> {
        Ok(self.position.len())
    }

    pub fn dump_to_file(&self) {
        let mut file = std::fs::File::create("latest-sim.json").unwrap();
        let json = serde_json::to_string_pretty(&self).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}