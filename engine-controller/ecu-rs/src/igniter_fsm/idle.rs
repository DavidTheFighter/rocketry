use core::borrow::BorrowMut;

use shared::{
    comms_hal::Packet,
    ecu_hal::{EcuSolenoidValve, FuelTankState, IgniterState, EcuDriver},
};

use crate::{Ecu, FiniteStateMachine};

use super::{FsmStorage, Idle};

impl FiniteStateMachine<IgniterState> for Idle {
    fn update<D: EcuDriver>(ecu: &mut Ecu<D>, _dt: f32, packet: &Option<Packet>) -> Option<IgniterState> {
        if let Some(Packet::FireIgniter) = packet {
            if ecu.fuel_tank_state == FuelTankState::Pressurized {
                return Some(IgniterState::Startup);
            }
        }

        None
    }

    fn setup_state<D: EcuDriver>(ecu: &mut Ecu<D>) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(EcuSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);

        super::reset_igniter_daq_collections(ecu.driver);

        ecu.igniter_fsm_storage = FsmStorage::Idle(Idle {});
    }
}
