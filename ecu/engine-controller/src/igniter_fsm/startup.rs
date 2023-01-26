use core::borrow::BorrowMut;

use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuSensor, EcuSolenoidValve, FuelTankState, IgniterState},
};

use crate::{Ecu, FiniteStateMachine};

use super::{FsmStorage, Startup};

impl FiniteStateMachine<IgniterState> for Startup {
    fn update(ecu: &mut Ecu, dt: f32, _packet: Option<Packet>) -> Option<IgniterState> {
        Startup::update_stable_pressure_timer(ecu, dt);

        let invalid_fsm_dependencies = Startup::check_fsm_dependencies(ecu);
        let startup_timed_out = Startup::startup_timed_out(ecu);
        let achieved_stable_pressure = Startup::achieved_stable_pressure(ecu);
        let throat_too_hot = Startup::throat_too_hot(ecu);

        if invalid_fsm_dependencies || startup_timed_out || throat_too_hot {
            return Some(IgniterState::Shutdown);
        }

        if achieved_stable_pressure {
            return Some(IgniterState::Firing);
        }

        None
    }

    fn setup_state(ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::IgniterFuelMain, true);
        driver.set_solenoid_valve(EcuSolenoidValve::IgniterGOxMain, true);
        driver.set_sparking(true);

        super::reset_igniter_daq_collections(ecu.driver);

        ecu.igniter_fsm_storage = FsmStorage::Startup(Startup {
            startup_elapsed_time: 0.0,
            stable_pressure_time: 0.0,
        });
    }
}

impl Startup {
    fn check_fsm_dependencies(ecu: &Ecu) -> bool {
        ecu.fuel_tank_state == FuelTankState::Pressurized
    }

    fn update_stable_pressure_timer(ecu: &mut Ecu, dt: f32) {
        let (_, chamber_pressure_min, _) = ecu
            .driver
            .collect_daq_sensor_data(EcuSensor::IgniterChamberPressure);

        let startup_pressure_threshold = ecu.igniter_config.startup_pressure_threshold;
        let storage = Startup::get_storage(ecu);

        if chamber_pressure_min >= startup_pressure_threshold {
            storage.stable_pressure_time += dt;
        } else {
            storage.stable_pressure_time = 0.0;
        }
    }

    fn startup_timed_out(ecu: &mut Ecu) -> bool {
        Startup::get_storage(ecu).startup_elapsed_time >= ecu.igniter_config.startup_timeout
    }

    fn throat_too_hot(ecu: &mut Ecu) -> bool {
        let (_, _, igniter_throat_temp_max) = ecu
            .driver
            .collect_daq_sensor_data(EcuSensor::IgniterThroatTemp);

        igniter_throat_temp_max >= ecu.igniter_config.max_throat_temp
    }

    fn achieved_stable_pressure(ecu: &mut Ecu) -> bool {
        Startup::get_storage(ecu).stable_pressure_time >= ecu.igniter_config.startup_stable_time
    }

    fn get_storage<'a>(ecu: &'a mut Ecu) -> &'a mut Startup {
        match &mut ecu.igniter_fsm_storage {
            FsmStorage::Startup(storage) => storage,
            _ => unreachable!(),
        }
    }
}
