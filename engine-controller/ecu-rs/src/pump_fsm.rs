use shared::{
    ecu_hal::PumpState,
    ControllerFsm, ControllerState,
};

use crate::Ecu;

pub mod idle;
pub mod pumping;

#[derive(Debug)]
pub enum PumpFsm {
    Idle(idle::Idle),
    Pumping(pumping::Pumping),
}

impl<'a> ControllerFsm<PumpFsm, Ecu<'a>, PumpState> for PumpFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<PumpFsm, Ecu<'a>> {
        match self {
            PumpFsm::Idle(state) => state,
            PumpFsm::Pumping(state) => state,
        }
    }

    fn hal_state(&self) -> PumpState {
        match self {
            PumpFsm::Idle(_) => PumpState::Idle,
            PumpFsm::Pumping(_) => PumpState::Pumping,
        }
    }
}
