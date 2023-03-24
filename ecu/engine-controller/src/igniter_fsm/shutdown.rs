use core::borrow::BorrowMut;

use super::{FsmStorage, Shutdown};
use crate::{Ecu, FiniteStateMachine};
use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuSolenoidValve, IgniterState, EcuDriver},
};

impl FiniteStateMachine<IgniterState> for Shutdown {
    fn update<D: EcuDriver>(ecu: &mut Ecu<D>, dt: f32, _packet: &Option<Packet>) -> Option<IgniterState> {
        Shutdown::update_state_duration(ecu, dt);

        let shutdown_time_elapsed = Shutdown::shutdown_time_elapsed(ecu);

        if shutdown_time_elapsed {
            return Some(IgniterState::Idle);
        }

        None
    }

    fn setup_state<D: EcuDriver>(ecu: &mut Ecu<D>) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(EcuSolenoidValve::IgniterGOxMain, true);
        driver.set_sparking(false);

        super::reset_igniter_daq_collections(ecu.driver);

        ecu.igniter_fsm_storage = FsmStorage::Shutdown(Shutdown { elapsed_time: 0.0 });
    }
}

impl Shutdown {
    fn update_state_duration<D: EcuDriver>(ecu: &mut Ecu<D>, dt: f32) {
        Shutdown::get_storage(ecu).elapsed_time += dt;
    }

    fn shutdown_time_elapsed<D: EcuDriver>(ecu: &mut Ecu<D>) -> bool {
        Shutdown::get_storage(ecu).elapsed_time >= ecu.igniter_config.shutdown_duration
    }

    fn get_storage<'a, D: EcuDriver>(ecu: &'a mut Ecu<D>) -> &'a mut Shutdown {
        match &mut ecu.igniter_fsm_storage {
            FsmStorage::Shutdown(storage) => storage,
            _ => unreachable!(),
        }
    }
}
