use flight_controller::Fcu;
use hal::{fcu_mock::FcuDriverMock};
use mint::Vector3;
use pyo3::prelude::*;
use pyo3::types::PyList;
use static_alloc::Bump;

static mut MOCK: FcuDriverMock = FcuDriverMock::new();

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
        Self {
            name: "FCU".to_string(),
            fcu: Fcu::new(mock),
        }
    }

    pub fn update(&mut self, dt: f32) {
        // something
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
}

#[pymodule]
fn software_in_loop(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SoftwareInLoop>()?;
    Ok(())
}

