use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

use super::PumpFsm;

#[derive(Debug)]
pub struct Pumping {
}

impl<'f> ControllerState<PumpFsm, Ecu<'f>> for Pumping {
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

impl Pumping {
    pub fn new() -> PumpFsm {
        PumpFsm::Pumping(Self {})
    }
}
