use shared::{ecu_hal::EngineState, ControllerFsm, ControllerState};

use crate::Ecu;

pub mod idle;

pub enum EngineFsm {
    Idle(idle::Idle),
}

impl<'a> ControllerFsm<EngineFsm, Ecu<'a>, EngineState> for EngineFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<EngineFsm, Ecu<'a>> {
        match self {
            EngineFsm::Idle(state) => state,
        }
    }

    fn hal_state(&self) -> EngineState {
        match self {
            EngineFsm::Idle(_) => EngineState::Idle,
        }
    }
}
