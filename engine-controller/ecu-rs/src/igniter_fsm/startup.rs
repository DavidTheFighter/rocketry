use core::borrow::BorrowMut;

use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuBinaryValve, TankState},
    ControllerState,
};

use crate::{silprintln, Ecu};

use super::{firing::Firing, shutdown::Shutdown, IgniterFsm};

pub struct Startup {
    startup_elapsed_time: f32,
    stable_pressure_time: f32,
}

impl<'f> ControllerState<IgniterFsm, Ecu<'f>> for Startup {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<IgniterFsm> {
        self.update_stable_pressure_timer(ecu, dt);
        self.startup_elapsed_time += dt;

        if !self.tanks_pressurized(ecu) || self.startup_timed_out(ecu) || self.throat_too_hot(ecu) {
            return Some(Shutdown::new());
        }

        if self.achieved_stable_pressure(ecu) {
            return Some(Firing::new());
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_binary_valve(EcuBinaryValve::IgniterFuelMain, true);
        driver.set_binary_valve(EcuBinaryValve::IgniterOxidizerMain, true);
        driver.set_sparking(true);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Startup {
    pub fn new() -> IgniterFsm {
        IgniterFsm::Startup(Self {
            startup_elapsed_time: 0.0,
            stable_pressure_time: 0.0,
        })
    }

    fn tanks_pressurized(&self, ecu: &Ecu) -> bool {
        ecu.fuel_tank_state()
            .map_or(true, |state| state == TankState::Pressurized)
            && ecu
                .oxidizer_tank_state()
                .map_or(true, |state| state == TankState::Pressurized)
    }

    fn update_stable_pressure_timer(&mut self, ecu: &mut Ecu, dt: f32) {
        let startup_pressure_threshold_pa = ecu.config.igniter_config.startup_pressure_threshold_pa;

        if ecu.igniter_chamber_pressure_pa >= startup_pressure_threshold_pa {
            self.stable_pressure_time += dt;
        } else {
            self.stable_pressure_time = 0.0;
        }
    }

    fn startup_timed_out(&self, ecu: &mut Ecu) -> bool {
        self.startup_elapsed_time >= ecu.config.igniter_config.startup_timeout_s
    }

    fn throat_too_hot(&self, ecu: &mut Ecu) -> bool {
        let igniter_throat_temp_max = 0.0; // TODO

        igniter_throat_temp_max >= ecu.config.igniter_config.max_throat_temp_k
    }

    fn achieved_stable_pressure(&self, ecu: &mut Ecu) -> bool {
        self.stable_pressure_time >= ecu.config.igniter_config.startup_stable_time_s
    }
}
