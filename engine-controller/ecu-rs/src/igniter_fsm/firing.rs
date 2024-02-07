use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuBinaryValve, TankState},
    ControllerState,
};

use crate::Ecu;

use super::{shutdown::Shutdown, IgniterFsm};

pub struct Firing {
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

        if !self.tanks_pressurized(ecu) || self.firing_ended(ecu) || self.throat_too_hot(ecu) {
            return Some(Shutdown::new());
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver
            .set_binary_valve(EcuBinaryValve::IgniterFuelMain, true);
        ecu.driver
            .set_binary_valve(EcuBinaryValve::IgniterOxidizerMain, true);
        ecu.driver.set_sparking(false);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Firing {
    pub fn new() -> IgniterFsm {
        IgniterFsm::Firing(Self { elapsed_time: 0.0 })
    }

    fn tanks_pressurized(&self, ecu: &Ecu) -> bool {
        ecu.fuel_tank_state()
            .map_or(true, |state| state == TankState::Pressurized)
            && ecu
                .oxidizer_tank_state()
                .map_or(true, |state| state == TankState::Pressurized)
    }

    fn firing_ended(&self, ecu: &mut Ecu) -> bool {
        self.elapsed_time >= ecu.config.igniter_config.test_firing_duration_s
    }

    fn throat_too_hot(&self, ecu: &mut Ecu) -> bool {
        let igniter_throat_temp_max = 0.0; // TODO

        igniter_throat_temp_max >= ecu.config.igniter_config.max_throat_temp_k
    }
}
