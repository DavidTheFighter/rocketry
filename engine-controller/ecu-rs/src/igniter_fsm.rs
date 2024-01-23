use shared::{ecu_hal::IgniterState, ControllerFsm, ControllerState};

use crate::Ecu;

mod firing;
mod idle;
mod shutdown;
mod startup;

pub enum IgniterFsm {
    Idle(idle::Idle),
    Startup(startup::Startup),
    Firing(firing::Firing),
    Shutdown(shutdown::Shutdown),
}

impl<'a> ControllerFsm<IgniterFsm, Ecu<'a>, IgniterState> for IgniterFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<IgniterFsm, Ecu<'a>> {
        match self {
            IgniterFsm::Idle(state) => state,
            IgniterFsm::Startup(state) => state,
            IgniterFsm::Firing(state) => state,
            IgniterFsm::Shutdown(state) => state,
        }
    }

    fn hal_state(&self) -> IgniterState {
        match self {
            IgniterFsm::Idle(_) => IgniterState::Idle,
            IgniterFsm::Startup(_) => IgniterState::Startup,
            IgniterFsm::Firing(_) => IgniterState::Firing,
            IgniterFsm::Shutdown(_) => IgniterState::Shutdown,
        }
    }
}
