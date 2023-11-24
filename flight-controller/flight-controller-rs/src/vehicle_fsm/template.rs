use shared::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use crate::Fcu;
use super::{ComponentStateMachine, FsmState, STATE};

impl<'f> ControllerState<FsmState, Fcu<'f>> for STATE {
    fn update<'a>(&mut self, fcu: & mut Fcu, dt: f32, packets: &[(NetworkAddress, Packet)]) -> Option<FsmState> {
        None
    }

    fn enter_state(&mut self, fcu: & mut Fcu) {
        todo!()
    }

    fn exit_state(&mut self, _fcu: & mut Fcu) {
        todo!()
    }

    fn hal_state(&self) -> VehicleState {
        todo!()
    }
}

impl STATE {
    pub fn new() -> FsmState {
        todo!()
    }
}
