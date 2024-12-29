use core::borrow::BorrowMut;

use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuBinaryOutput, IgniterConfig, TankState},
    ControllerState,
};

use crate::Ecu;

use super::{firing::Firing, shutdown::Shutdown, IgniterFsm};

pub struct Startup {
    igniter_config: IgniterConfig,
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

        if !self.tanks_pressurized(ecu) {
            ecu.alert_manager
                .set_condition(EcuAlert::IgniterTankOffNominal);
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        if self.startup_timed_out() {
            ecu.alert_manager
                .set_condition(EcuAlert::IgniterStartupTimeOut);
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        if self.throat_too_hot() {
            ecu.alert_manager
                .set_condition(EcuAlert::IgniterThroatOverheat);
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        if self.achieved_stable_pressure() {
            return Some(Firing::new(self.igniter_config.clone()));
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_binary_valve(EcuBinaryOutput::IgniterFuelValve, true);
        driver.set_binary_valve(EcuBinaryOutput::IgniterOxidizerValve, true);
        driver.set_sparking(true);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Startup {
    pub fn new(igniter_config: IgniterConfig) -> IgniterFsm {
        IgniterFsm::Startup(Self {
            igniter_config,
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
        let startup_pressure_threshold_pa = self.igniter_config.startup_pressure_threshold_pa;

        if ecu.state_vector.sensor_data.igniter_chamber_pressure_pa >= startup_pressure_threshold_pa
        {
            self.stable_pressure_time += dt;
        } else {
            self.stable_pressure_time = 0.0;
        }
    }

    fn startup_timed_out(&self) -> bool {
        self.startup_elapsed_time >= self.igniter_config.startup_timeout_s
    }

    fn throat_too_hot(&self) -> bool {
        let igniter_throat_temp_max = 0.0; // TODO

        igniter_throat_temp_max >= self.igniter_config.max_throat_temp_k
    }

    fn achieved_stable_pressure(&self) -> bool {
        self.stable_pressure_time >= self.igniter_config.startup_stable_time_s
    }
}
