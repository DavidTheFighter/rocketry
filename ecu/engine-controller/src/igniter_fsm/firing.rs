use core::borrow::BorrowMut;

use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuSensor, EcuSolenoidValve, FuelTankState, IgniterState, EcuDriver},
};

use crate::{Ecu, FiniteStateMachine};

use super::{Firing, FsmStorage};

impl FiniteStateMachine<IgniterState> for Firing {
    fn update<D: EcuDriver>(ecu: &mut Ecu<D>, dt: f32, _packet: &Option<Packet>) -> Option<IgniterState> {
        Firing::update_firing_duration(ecu, dt);

        let invalid_fsm_dependencies = Firing::check_fsm_dependencies(ecu);
        let firing_ended = Firing::firing_ended(ecu);
        let throat_too_hot = Firing::throat_too_hot(ecu);

        if invalid_fsm_dependencies || firing_ended || throat_too_hot {
            return Some(IgniterState::Shutdown);
        }

        None
    }

    fn setup_state<D: EcuDriver>(ecu: &mut Ecu<D>) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::IgniterFuelMain, true);
        driver.set_solenoid_valve(EcuSolenoidValve::IgniterGOxMain, true);
        driver.set_sparking(false);

        super::reset_igniter_daq_collections(ecu.driver);

        ecu.igniter_fsm_storage = FsmStorage::Firing(Firing { elapsed_time: 0.0 });
    }
}

impl Firing {
    fn check_fsm_dependencies<D: EcuDriver>(ecu: &Ecu<D>) -> bool {
        ecu.fuel_tank_state == FuelTankState::Pressurized
    }

    fn update_firing_duration<D: EcuDriver>(ecu: &mut Ecu<D>, dt: f32) {
        Firing::get_storage(ecu).elapsed_time += dt;
    }

    fn firing_ended<D: EcuDriver>(ecu: &mut Ecu<D>) -> bool {
        Firing::get_storage(ecu).elapsed_time >= ecu.igniter_config.firing_duration
    }

    fn throat_too_hot<D: EcuDriver>(ecu: &mut Ecu<D>) -> bool {
        let (_, _, igniter_throat_temp_max) = ecu
            .driver
            .collect_daq_sensor_data(EcuSensor::IgniterThroatTemp);

        igniter_throat_temp_max >= ecu.igniter_config.max_throat_temp
    }

    fn get_storage<'a, D: EcuDriver>(ecu: &'a mut Ecu<D>) -> &'a mut Firing {
        match &mut ecu.igniter_fsm_storage {
            FsmStorage::Firing(storage) => storage,
            _ => unreachable!(),
        }
    }
}