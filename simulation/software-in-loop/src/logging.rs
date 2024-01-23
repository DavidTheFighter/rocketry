use big_brother::{
    big_brother::BigBrotherPacket,
    interface::{mock_interface::MockPayload, mock_topology::MockPhysicalNet},
    serdes::{self, PacketMetadata},
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde::{Deserialize, Serialize};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{FcuDebugInfo, FcuDevStatsFrame, FcuTelemetryFrame},
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread,
};

use crate::{dynamics::SilVehicleDynamics, fcu::FcuSil, network::SilNetwork, ser::dict_from_obj};

type Scalar = f64;

#[pyclass]
#[derive(Clone, Serialize, Deserialize)]
pub struct Logger {
    #[pyo3(get, set)]
    pub dt: Scalar,
    #[serde(skip)]
    networks: Vec<Arc<Mutex<MockPhysicalNet>>>,
    num_timesteps: usize,
    // Per timestep data
    pub fcu_telemetry: Vec<FcuTelemetryFrame>,
    pub fcu_debug_info: Vec<Vec<FcuDebugInfo>>,
    pub dev_stats: Vec<FcuDevStatsFrame>,
    pub network_packets: Vec<Vec<(PacketMetadata<NetworkAddress>, Packet)>>,
    pub network_payloads: Vec<Vec<MockPayload>>,
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
    pub fn new(sil_networks: &PyList) -> Self {
        let mut networks = Vec::new();

        for sil_network in sil_networks.iter() {
            let sil_network = sil_network
                .extract::<PyRefMut<'_, SilNetwork>>()
                .expect("Failed to extract network");

            networks.push(sil_network.network.clone());
        }

        Self {
            dt: 0.0,
            networks,
            num_timesteps: 0,
            fcu_telemetry: Vec::new(),
            fcu_debug_info: Vec::new(),
            dev_stats: Vec::new(),
            network_packets: Vec::new(),
            network_payloads: Vec::new(),
            position: Vec::new(),
            velocity: Vec::new(),
            acceleration: Vec::new(),
            orientation: Vec::new(),
            angular_velocity: Vec::new(),
            angular_acceleration: Vec::new(),
        }
    }

    pub fn log_common_data(&mut self) {
        let mut packets = Vec::new();
        let mut payloads = Vec::new();

        for network in &self.networks {
            let mut network = network.lock().unwrap();
            let network_payloads = network.take_payload_log();

            for payload in network_payloads {
                let metadata = serdes::deserialize_metadata(&payload.data.as_slice()).unwrap();
                let bb_packet: BigBrotherPacket<Packet> =
                    serdes::deserialize_packet(&payload.data.as_slice()).unwrap();

                if let BigBrotherPacket::UserPacket(packet) = bb_packet {
                    packets.push((metadata, packet));
                }
                payloads.push(payload);
            }
        }

        self.network_packets.push(packets);
        self.network_payloads.push(payloads);
        self.num_timesteps += 1;
    }

    pub fn log_fcu_data(&mut self, fcu: &mut FcuSil) {
        if let Some(frame) = &fcu.fcu.last_telemetry_frame {
            self.fcu_telemetry.push(frame.clone());
        }

        let mut debug_infos = Vec::new();
        let debug_info_callback = |debug_info_variant| {
            debug_infos.push(debug_info_variant);
        };
        fcu.fcu
            .generate_debug_info_all_variants(debug_info_callback);
        self.fcu_debug_info.push(debug_infos);
    }

    pub fn log_dynamics_data(&mut self, dynamics: &mut SilVehicleDynamics) {
        self.position.push(dynamics.get_position().unwrap());
        self.velocity.push(dynamics.get_velocity().unwrap());
        self.acceleration
            .push(dynamics.get_acceleration_world_frame().unwrap());
        self.orientation.push(dynamics.get_orientation().unwrap());
        self.angular_velocity
            .push(dynamics.get_angular_velocity().unwrap());
        self.angular_acceleration
            .push(dynamics.get_angular_acceleration().unwrap());
    }

    pub fn grab_timestep_frame(&self, py: Python, i: usize) -> PyResult<PyObject> {
        let dict = PyDict::new(py);

        if self.position.len() > 0 {
            dict.set_item("position", self.position[i].clone())?;
            dict.set_item("velocity", self.velocity[i].clone())?;
            dict.set_item("acceleration", self.acceleration[i].clone())?;
            dict.set_item("orientation", self.orientation[i].clone())?;
            dict.set_item("angular_velocity", self.angular_velocity[i].clone())?;
            dict.set_item("angular_acceleration", self.angular_acceleration[i].clone())?;
        }

        if self.fcu_telemetry.len() > 0 {
            dict.set_item("fcu_telemetry", dict_from_obj(py, &self.fcu_telemetry[i]))?;

            let debug_info_dict = PyDict::new(py);
            for variant in &self.fcu_debug_info[i] {
                for value in dict_from_obj(py, variant).values() {
                    for (key, value) in value.downcast::<PyDict>().unwrap().iter() {
                        debug_info_dict.set_item(key, value)?;
                    }
                }
            }
            dict.set_item("fcu_debug_info", debug_info_dict)?;
        }

        Ok(dict.into())
    }

    pub fn get_network_packet_bytes(&mut self, py: Python, i: usize) -> PyResult<PyObject> {
        let packet_list = PyList::empty(py);

        for payload in &self.network_payloads[i] {
            packet_list.append(PyList::new(py, payload.data.iter()))?;
        }

        Ok(packet_list.into())
    }

    pub fn get_dev_stat_frames(&self, py: Python) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        for frame in &self.dev_stats {
            list.append(dict_from_obj(py, frame))?;
        }

        Ok(list.into())
    }

    pub fn num_timesteps(&self) -> PyResult<usize> {
        Ok(self.num_timesteps)
    }

    pub fn dump_to_file(&self) {
        let data = self.clone();

        thread::spawn(move || {
            let mut file = std::fs::File::create("last-sim.json").unwrap();
            let json = serde_json::to_string(&data).unwrap();
            file.write_all(json.as_bytes()).unwrap();
            file.flush().unwrap();
            println!("Finished saving simulation data to file!");
        });
    }
}

#[pyfunction]
pub fn load_logs_from_file(file: &str) -> PyResult<Logger> {
    let file = std::fs::File::open(file).expect("1");
    let data: Logger = serde_json::from_reader(file).expect("2");

    Ok(data)
}
