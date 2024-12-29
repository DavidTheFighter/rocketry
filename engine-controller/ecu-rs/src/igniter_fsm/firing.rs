use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuBinaryOutput, IgniterConfig, TankState},
    ControllerState,
};

use crate::Ecu;

use super::{shutdown::Shutdown, IgniterFsm};

pub struct Firing {
    igniter_config: IgniterConfig,
    elapsed_time: f32,
}

impl<'f> ControllerState<IgniterFsm, Ecu<'f>> for Firing {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<IgniterFsm> {
        self.elapsed_time += dt;

        if !self.tanks_pressurized(ecu) {
            ecu.alert_manager
                .set_condition(EcuAlert::IgniterTankOffNominal);
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        if self.throat_too_hot() {
            ecu.alert_manager
                .set_condition(EcuAlert::IgniterThroatOverheat);
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        if self.firing_ended() {
            return Some(Shutdown::new(self.igniter_config.clone()));
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::IgniterFuelValve, true);
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::IgniterOxidizerValve, true);
        ecu.driver.set_sparking(false);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Firing {
    pub fn new(igniter_config: IgniterConfig) -> IgniterFsm {
        IgniterFsm::Firing(Self {
            igniter_config,
            elapsed_time: 0.0,
        })
    }

    fn tanks_pressurized(&self, ecu: &Ecu) -> bool {
        ecu.fuel_tank_state()
            .map_or(true, |state| state == TankState::Pressurized)
            && ecu
                .oxidizer_tank_state()
                .map_or(true, |state| state == TankState::Pressurized)
    }

    fn firing_ended(&self) -> bool {
        self.elapsed_time >= self.igniter_config.test_firing_duration_s
    }

    fn throat_too_hot(&self) -> bool {
        let igniter_throat_temp_max = 0.0; // TODO

        igniter_throat_temp_max >= self.igniter_config.max_throat_temp_k
    }
}
