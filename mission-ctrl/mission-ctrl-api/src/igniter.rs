use pyo3::{prelude::*, types::PyDict};
use serde_json::Value;
use shared::{comms_hal::Packet, ecu_hal::{EcuCommand, EcuResponse}};

use crate::{dict_from_obj, CommandHandler};

#[pyclass]
pub struct Igniter {
    #[pyo3(get, set)]
    pub ecu_index: u8,
    command_handler: Py<CommandHandler>,
}

#[pymethods]
impl Igniter {
    #[new]
    pub fn new(ecu_index: u8, command_handler: Py<CommandHandler>) -> Self {
        Self { ecu_index, command_handler }
    }

    pub fn fire(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(self.ecu_index, EcuCommand::FireIgniter)
    }

    pub fn config(&mut self, py: Python) -> PyResult<PyObject> {
        let config = self.command_handler.borrow(py).send_packet_and_receive_response(
            Packet::EcuCommand(EcuCommand::GetConfig),
            shared::comms_hal::NetworkAddress::EngineController(self.ecu_index),
            |packet: &shared::comms_hal::Packet| {
                matches!(packet, Packet::EcuResponse(EcuResponse::Config(_)))
            },
        )?;

        if let Packet::EcuResponse(EcuResponse::Config(config)) = config {
            let dict = dict_from_obj(py, config.igniter_config);

            Ok(dict.into())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyException, _>("Failed to get config"))
        }
    }
}
