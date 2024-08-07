use pyo3::prelude::*;
use shared::ecu_hal::{EcuCommand, PumpType};

use crate::CommandHandler;

#[pyclass]
pub struct Pump {
    #[pyo3(get, set)]
    pub ecu_index: u8,
    pub pump_type: PumpType,
    command_handler: Py<CommandHandler>,
}

#[pymethods]
impl Pump {
    #[new]
    pub fn new(pump_type: String, ecu_index: u8, command_handler: Py<CommandHandler>) -> Self {
        Self {
            pump_type: match pump_type.as_str() {
                "FuelMain" => PumpType::FuelMain,
                "OxidizerMain" => PumpType::OxidizerMain,
                _ => panic!("Invalid pump type"),
            },
            ecu_index,
            command_handler,
        }
    }

    pub fn full(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(self.ecu_index, EcuCommand::SetPumpDuty((self.pump_type, 1.0)))
    }

    pub fn off(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(self.ecu_index, EcuCommand::SetPumpDuty((self.pump_type, 0.0)))
    }

    pub fn set_duty(&mut self, py: Python, duty: f32) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(self.ecu_index, EcuCommand::SetPumpDuty((self.pump_type, duty)))
    }
}
