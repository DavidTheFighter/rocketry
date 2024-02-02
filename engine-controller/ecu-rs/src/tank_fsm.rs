use shared::{
    ecu_hal::{EcuBinaryValve, TankState},
    ControllerFsm, ControllerState,
};

use crate::Ecu;

pub mod depressurized;
pub mod idle;
pub mod pressurized;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TankType {
    Fuel,
    Oxidizer,
}

#[derive(Debug)]
pub enum TankFsm {
    Idle(idle::Idle),
    Depressurized(depressurized::Depressurized),
    Pressurized(pressurized::Pressurized),
}

impl<'a> ControllerFsm<TankFsm, Ecu<'a>, TankState> for TankFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<TankFsm, Ecu<'a>> {
        match self {
            TankFsm::Idle(state) => state,
            TankFsm::Depressurized(state) => state,
            TankFsm::Pressurized(state) => state,
        }
    }

    fn hal_state(&self) -> TankState {
        match self {
            TankFsm::Idle(_) => TankState::Idle,
            TankFsm::Depressurized(_) => TankState::Depressurized,
            TankFsm::Pressurized(_) => TankState::Pressurized,
        }
    }
}

fn new_state_from_command(
    state: TankState,
    tank_type: TankType,
    press_valve: EcuBinaryValve,
    vent_valve: EcuBinaryValve,
) -> TankFsm {
    match state {
        TankState::Idle => idle::Idle::new(tank_type, press_valve, vent_valve),
        TankState::Depressurized => depressurized::Depressurized::new(tank_type, press_valve, vent_valve),
        TankState::Pressurized => pressurized::Pressurized::new(tank_type, press_valve, vent_valve),
    }
}
