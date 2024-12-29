use core::borrow::BorrowMut;

use super::{idle::Idle, IgniterFsm};
use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuBinaryOutput, IgniterConfig},
    ControllerState,
};

pub struct Shutdown {
    igniter_config: IgniterConfig,
    elapsed_time: f32,
}

impl<'f> ControllerState<IgniterFsm, Ecu<'f>> for Shutdown {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<IgniterFsm> {
        self.elapsed_time += dt;

        if self.shutdown_time_elapsed() {
            return Some(Idle::new(self.igniter_config.clone()));
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_binary_valve(EcuBinaryOutput::IgniterFuelValve, false);
        driver.set_binary_valve(EcuBinaryOutput::IgniterOxidizerValve, true);
        driver.set_sparking(false);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Shutdown {
    pub fn new(igniter_config: IgniterConfig) -> IgniterFsm {
        IgniterFsm::Shutdown(Self {
            igniter_config,
            elapsed_time: 0.0,
        })
    }

    fn shutdown_time_elapsed(&self) -> bool {
        self.elapsed_time >= self.igniter_config.shutdown_duration_s
    }
}
