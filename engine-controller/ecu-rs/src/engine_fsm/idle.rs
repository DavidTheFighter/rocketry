use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

use crate::Ecu;

use super::EngineFsm;

pub struct Idle;

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        None
    }

    fn enter_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Idle {
    pub fn new() -> EngineFsm {
        EngineFsm::Idle(Self {})
    }
}
