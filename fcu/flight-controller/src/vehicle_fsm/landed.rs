use super::{ComponentStateMachine, FsmState, Landed};
use crate::Fcu;
use hal::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
};

impl ComponentStateMachine<FsmState> for Landed {
    fn update(
        &mut self,
        _fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        None
    }

    fn enter_state<'a>(&mut self, fcu: &'a mut Fcu) {
        fcu.state_vector.set_landed(true);
    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn hal_state(&self) -> VehicleState {
        VehicleState::Landed
    }
}

impl Landed {
    pub fn new() -> FsmState {
        FsmState::Landed(Self {})
    }
}
