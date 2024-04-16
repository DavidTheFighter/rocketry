use shared::{ecu_hal::EngineState, ControllerFsm, ControllerState};

use crate::Ecu;

pub mod idle;
pub mod pump_startup;
pub mod igniter_startup;
pub mod engine_startup;
pub mod firing;
pub mod engine_shutdown;

pub enum EngineFsm {
    Idle(idle::Idle),
    PumpStartup(pump_startup::PumpStartup),
    IgniterStartup(igniter_startup::IgniterStartup),
    EngineStartup(engine_startup::EngineStartup),
    Firing(firing::Firing),
    EngineShutdown(engine_shutdown::EngineShutdown),
}

impl<'a> ControllerFsm<EngineFsm, Ecu<'a>, EngineState> for EngineFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<EngineFsm, Ecu<'a>> {
        match self {
            EngineFsm::Idle(state) => state,
            EngineFsm::PumpStartup(state) => state,
            EngineFsm::IgniterStartup(state) => state,
            EngineFsm::EngineStartup(state) => state,
            EngineFsm::Firing(state) => state,
            EngineFsm::EngineShutdown(state) => state,
        }
    }

    fn hal_state(&self) -> EngineState {
        match self {
            EngineFsm::Idle(_) => EngineState::Idle,
            EngineFsm::PumpStartup(_) => EngineState::PumpStartup,
            EngineFsm::IgniterStartup(_) => EngineState::IgniterStartup,
            EngineFsm::EngineStartup(_) => EngineState::EngineStartup,
            EngineFsm::Firing(_) => EngineState::Firing,
            EngineFsm::EngineShutdown(_) => EngineState::EngineShutdown,
        }
    }
}
