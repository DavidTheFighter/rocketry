use core::borrow::BorrowMut;

use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuSolenoidValve, FuelTankState, IgniterState},
};

use crate::{Ecu, FiniteStateMachine};

use super::{FsmStorage, Idle};

impl FiniteStateMachine<IgniterState> for Idle {
    fn update(ecu: &mut Ecu, _dt: f32, packet: Option<Packet>) -> Option<IgniterState> {
        if let Some(Packet::FireIgniter) = packet {
            if ecu.fuel_tank_state == FuelTankState::Pressurized {
                return Some(IgniterState::Startup);
            }
        }

        None
    }

    fn setup_state(ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(EcuSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);

        super::reset_igniter_daq_collections(ecu.driver);

        ecu.igniter_fsm_storage = FsmStorage::Idle(Idle {});
    }
}
