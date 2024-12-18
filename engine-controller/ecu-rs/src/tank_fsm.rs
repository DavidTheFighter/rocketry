use shared::{
    ecu_hal::{EcuBinaryOutput, TankState, TankType},
    ControllerFsm, ControllerState,
};

use crate::Ecu;

pub mod filling;
pub mod idle;
pub mod pressurized;
pub mod venting;

#[derive(Debug)]
pub enum TankFsm {
    Idle(idle::Idle),
    Venting(venting::Venting),
    Pressurized(pressurized::Pressurized),
    Filling(filling::Filling),
}

impl<'a> ControllerFsm<TankFsm, Ecu<'a>, TankState> for TankFsm {
    fn to_controller_state(&mut self) -> &mut dyn ControllerState<TankFsm, Ecu<'a>> {
        match self {
            TankFsm::Idle(state) => state,
            TankFsm::Venting(state) => state,
            TankFsm::Pressurized(state) => state,
            TankFsm::Filling(state) => state,
        }
    }

    fn hal_state(&self) -> TankState {
        match self {
            TankFsm::Idle(_) => TankState::Idle,
            TankFsm::Venting(_) => TankState::Venting,
            TankFsm::Pressurized(_) => TankState::Pressurized,
            TankFsm::Filling(_) => TankState::Filling,
        }
    }
}

fn new_state_from_command(
    state: TankState,
    tank_type: TankType,
    press_valve: Option<EcuBinaryOutput>,
    fill_valve: Option<EcuBinaryOutput>,
    vent_valve: Option<EcuBinaryOutput>,
) -> TankFsm {
    match state {
        TankState::Idle => idle::Idle::new(tank_type, press_valve, fill_valve, vent_valve),
        TankState::Venting => venting::Venting::new(tank_type, press_valve, fill_valve, vent_valve),
        TankState::Pressurized => pressurized::Pressurized::new(tank_type, press_valve, fill_valve, vent_valve),
        TankState::Filling => filling::Filling::new(tank_type, press_valve, fill_valve, vent_valve),
    }
}
