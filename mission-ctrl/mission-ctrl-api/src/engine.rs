use pyo3::prelude::*;
use shared::ecu_hal::EcuCommand;

use crate::CommandHandler;

#[pyclass]
pub struct Engine {
    #[pyo3(get, set)]
    pub ecu_index: u8,
    command_handler: Py<CommandHandler>,
}

#[pymethods]
impl Engine {
    #[new]
    pub fn new(ecu_index: u8, command_handler: Py<CommandHandler>) -> Self {
        Self {
            ecu_index,
            command_handler,
        }
    }

    pub fn fire(&mut self, py: Python) -> PyResult<()> {
        self.command_handler
            .borrow(py)
            .send_ecu_command(self.ecu_index, EcuCommand::FireEngine)
    }
}
