use pyo3::prelude::*;
use shared::ecu_hal::{EcuCommand, TankState, TankType};

use crate::CommandHandler;

#[pyclass]
pub struct Tank {
    pub tank_type: TankType,
    #[pyo3(get, set)]
    pub ecu_index: u8,
    command_handler: Py<CommandHandler>,
}

#[pymethods]
impl Tank {
    #[new]
    pub fn new(tank_type: String, ecu_index: u8, command_handler: Py<CommandHandler>) -> Self {
        Self {
            tank_type: match tank_type.as_str() {
                "FuelMain" => TankType::FuelMain,
                "OxidizerMain" => TankType::OxidizerMain,
                _ => panic!("Invalid tank type"),
            },
            ecu_index,
            command_handler,
        }
    }

    pub fn press(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(
            self.ecu_index,
            EcuCommand::SetTankState((self.tank_type, TankState::Pressurized)),
        )
    }

    pub fn idle(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(
            self.ecu_index,
            EcuCommand::SetTankState((self.tank_type, TankState::Idle)),
        )
    }

    pub fn depress(&mut self, py: Python) -> PyResult<()> {
        self.command_handler.borrow(py).send_ecu_command(
            self.ecu_index,
            EcuCommand::SetTankState((self.tank_type, TankState::Depressurized)),
        )
    }
}
