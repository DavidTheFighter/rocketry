use shared::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use crate::Fcu;
use super::{ComponentStateMachine, FsmState, Calibrating};

impl ComponentStateMachine<FsmState> for STATE {
    fn update<'a>(&mut self, fcu: &'a mut Fcu, dt: f32, packets: &[(NetworkAddress, Packet)]) -> Option<FsmState> {
        None
    }

    fn enter_state<'a>(&mut self, fcu: &'a mut Fcu) {
        todo!()
    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        todo!()
    }

    fn hal_state(&self) -> VehicleState {
        todo!()
    }
}

impl Idle {
    pub fn new() -> FsmState {
        todo!()
    }
}
