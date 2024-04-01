use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

use super::PumpFsm;

#[derive(Debug)]
pub struct Idle {
}

impl<'f> ControllerState<PumpFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<PumpFsm> {
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
    pub fn new() -> PumpFsm {
        PumpFsm::Idle(Self {})
    }
}
