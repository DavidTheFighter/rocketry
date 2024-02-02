use core::borrow::BorrowMut;

use super::{idle::Idle, IgniterFsm};
use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::EcuBinaryValve,
    ControllerState,
};

pub struct Shutdown {
    elapsed_time: f32,
}

impl<'f> ControllerState<IgniterFsm, Ecu<'f>> for Shutdown {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<IgniterFsm> {
        self.elapsed_time += dt;

        if self.shutdown_time_elapsed(ecu) {
            return Some(Idle::new());
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_binary_valve(EcuBinaryValve::IgniterFuelMain, false);
        driver.set_binary_valve(EcuBinaryValve::IgniterGOxMain, true);
        driver.set_sparking(false);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Shutdown {
    pub fn new() -> IgniterFsm {
        IgniterFsm::Shutdown(Self { elapsed_time: 0.0 })
    }

    fn shutdown_time_elapsed(&self, ecu: &mut Ecu) -> bool {
        self.elapsed_time >= ecu.config.igniter_config.shutdown_duration
    }
}
