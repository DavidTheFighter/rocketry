pub mod engine;
pub mod igniter;
pub mod pump;
pub mod tank;

use std::{net::TcpStream, sync::Mutex};

use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde::Serialize;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::EcuCommand,
};
use tungstenite::{connect, stream::MaybeTlsStream, WebSocket};

#[pyclass]
pub struct CommandHandler {
    pub websocket: Mutex<WebSocket<MaybeTlsStream<TcpStream>>>,
}

#[pymethods]
impl CommandHandler {
    #[new]
    pub fn new(websocket_url: String) -> PyResult<Self> {
        let (websocket, response) = connect(websocket_url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;

        println!("Websocket HTTP code: {}", response.status());

        Ok(Self {
            websocket: Mutex::new(websocket),
        })
    }
}

impl CommandHandler {
    pub fn send_packet(&self, packet: Packet, destination: NetworkAddress) -> PyResult<()> {
        let packet_with_address = shared::comms_hal::PacketWithAddress {
            address: destination,
            packet,
        };

        let json_str = serde_json::to_string(&packet_with_address).unwrap();

        self.websocket
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?
            .write(tungstenite::Message::Text(json_str))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;

        self.websocket
            .lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?
            .flush()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;

        Ok(())
    }

    pub fn send_packet_and_receive_response<F>(
        &self,
        packet: Packet,
        destination: NetworkAddress,
        callback_fn: F,
    ) -> PyResult<Packet>
    where
        F: Fn(&Packet) -> bool,
    {
        self.send_packet(packet, destination)?;

        loop {
            let message = self
                .websocket
                .lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?
                .read()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))?;

            if let tungstenite::Message::Text(json_str) = message {
                let packet_with_address: shared::comms_hal::PacketWithAddress =
                    serde_json::from_str(&json_str).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e))
                    })?;

                if packet_with_address.address == destination {
                    if callback_fn(&packet_with_address.packet) {
                        return Ok(packet_with_address.packet);
                    }
                }
            }
        }
    }

    pub fn send_ecu_command(&self, ecu_index: u8, command: EcuCommand) -> PyResult<()> {
        self.send_packet(
            Packet::EcuCommand(command),
            NetworkAddress::EngineController(ecu_index),
        )
    }
}

#[pymodule]
fn mission_ctrl_api(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CommandHandler>()?;
    m.add_class::<igniter::Igniter>()?;
    m.add_class::<pump::Pump>()?;
    m.add_class::<tank::Tank>()?;

    Ok(())
}

pub fn list_from_array<T: Serialize>(py: Python, list: T) -> &PyList {
    let binding = serde_json::to_value(&list).expect("Failed to serialize list");
    let values = binding.as_array().unwrap();

    let list = PyList::empty(py);
    for value in values {
        if value.is_object() {
            list.append(dict_from_obj(py, value)).unwrap();
        } else if value.is_array() {
            list.append(list_from_array(py, value)).unwrap();
        } else if value.is_i64() {
            list.append(value.as_i64().unwrap()).unwrap();
        } else if value.is_number() {
            list.append(value.as_f64().unwrap()).unwrap();
        } else if value.is_string() {
            list.append(value.as_str().unwrap()).unwrap();
        } else if value.is_boolean() {
            list.append(value.as_bool().unwrap()).unwrap();
        } else if !value.is_null() {
            panic!("Unsupported type {:?} for list ({:?}", value, list);
        }
    }

    list
}

pub fn dict_from_obj<T: Serialize>(py: Python, obj: T) -> &PyDict {
    let binding = serde_json::to_value(&obj).expect("Failed to serialize object");
    let values = binding.as_object().unwrap();

    let dict = PyDict::new(py);
    for (key, value) in values {
        if value.is_object() {
            dict.set_item(key, dict_from_obj(py, value)).unwrap();
        } else if value.is_array() {
            dict.set_item(key, list_from_array(py, value)).unwrap();
        } else if value.is_i64() {
            dict.set_item(key, value.as_i64().unwrap()).unwrap();
        } else if value.is_number() {
            dict.set_item(key, value.as_f64().unwrap()).unwrap();
        } else if value.is_string() {
            dict.set_item(key, value.as_str().unwrap()).unwrap();
        } else if value.is_boolean() {
            dict.set_item(key, value.as_bool().unwrap()).unwrap();
        } else if !value.is_null() {
            panic!(
                "Unsupported type {:?} for key {:?} ({:?})",
                value, key, dict
            );
        }
    }

    dict
}
