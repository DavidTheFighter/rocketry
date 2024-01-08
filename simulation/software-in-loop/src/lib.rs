pub mod driver;
pub mod dynamics;
pub mod logging;
pub mod ser;

use std::str::FromStr;
use std::cell::RefCell;
use std::rc::Rc;

use big_brother::big_brother::UDP_PORT;
use big_brother::interface::mock_interface::MockInterface;
use driver::FcuDriverSim;
use dynamics::Dynamics;
use flight_controller_rs::{Fcu, FcuBigBrother};
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
    _ip_interface: Rc<RefCell<MockInterface>>,
    _big_brother: Rc<RefCell<FcuBigBrother<'static>>>,
    _data_point_logger: Rc<RefCell<DataPointLoggerMock>>,
    temp_fcu_packet_counter: u32,
    fcu: Fcu<'static>,
}

#[pymethods]
impl FcuSil {
    #[new]
    pub fn new() -> Self {
        let driver = Rc::new(RefCell::new(FcuDriverSim::new()));
        let driver_ref: &'static mut FcuDriverSim = unsafe {
            std::mem::transmute(&mut *driver.borrow_mut())
        };

        let ip_interface = Rc::new(RefCell::new(MockInterface::new()));
        let ip_interface_ref: &'static mut MockInterface = unsafe {
            std::mem::transmute(&mut *ip_interface.borrow_mut())
        };

        let big_brother = Rc::new(RefCell::new(FcuBigBrother::new(
            NetworkAddress::FlightController,
            rand::random(),
            NetworkAddress::Broadcast,
            [Some(ip_interface_ref), None],
        )));
        let big_brother_ref: &'static mut FcuBigBrother<'static> = unsafe {
            std::mem::transmute(&mut *big_brother.borrow_mut())
        };

        let data_point_logger = Rc::new(RefCell::new(DataPointLoggerMock));
        let data_point_logger_ref: &'static mut DataPointLoggerMock = unsafe {
            std::mem::transmute(&mut *data_point_logger.borrow_mut())
        };

        let fcu = Fcu::new(driver_ref, big_brother_ref, data_point_logger_ref);

        Self {
            name: "FCU".to_string(),
            _driver: driver,
            _ip_interface: ip_interface,
            _big_brother: big_brother,
            _data_point_logger: data_point_logger,
            temp_fcu_packet_counter: 0,
            fcu,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fcu.update(dt);
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

        self.send_packet(NetworkAddress::MissionControl, &Packet::ConfigureFcu(config));
        // self.pending_packets.push((NetworkAddress::MissionControl, Packet::ConfigureFcu(config)));
    }

    pub fn fcu_config(&self, py: Python) -> PyResult<PyObject> {
        Ok(dict_from_obj(py, &self.fcu.get_fcu_config()).into())
    }

    pub fn start_dev_stats_frame(&mut self) {
        self.send_packet(NetworkAddress::MissionControl, &Packet::StartDevStatsFrame);
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
            .ok_or(PyTypeError::new_err("Failed to retrieve driver from FCU object"))?
            .set_output_channel_continuity(channel, state);

        Ok(())
    }

    pub fn reset_telemetry(&mut self) {
        self.send_packet(NetworkAddress::MissionControl, &Packet::FcuTelemetry(FcuTelemetryFrame::default()));
    }

    pub fn send_arm_vehicle_packet(&mut self) {
        self.send_packet(NetworkAddress::MissionControl, &Packet::ArmVehicle { magic_number: fcu_hal::ARMING_MAGIC_NUMBER });
        // self.pending_packets.push((NetworkAddress::MissionControl, Packet::ArmVehicle { magic_number: fcu_hal::ARMING_MAGIC_NUMBER }));
    }

    pub fn send_ignite_solid_motor_packet(&mut self) {
        self.send_packet(NetworkAddress::MissionControl, &Packet::IgniteSolidMotor { magic_number: fcu_hal::IGNITION_MAGIC_NUMBER });
        // self.pending_packets.push((NetworkAddress::MissionControl, Packet::IgniteSolidMotor { magic_number: fcu_hal::IGNITION_MAGIC_NUMBER }));
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

impl FcuSil {
    fn send_packet(&mut self, from_addr: NetworkAddress, packet: &Packet) {
        println!("From {:?} sending packet: {:?}", from_addr, packet);

        self.fcu
            .comms
            .interfaces[0]
            .as_mut()
            .expect("Failed to get interface")
            .as_mut_any()
            .expect("Failed to get interface as any")
            .downcast_mut::<MockInterface>()
            .expect("Failed to downcast interface")
            .add_recv_packet(
                NetworkAddress::FlightController,
                from_addr,
                [127, 0, 0, 1],
                UDP_PORT,
                self.temp_fcu_packet_counter,
                packet,
            )
            .expect("Failed to add packet to mock interface");

        self.temp_fcu_packet_counter += 1;
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