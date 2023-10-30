use super::{Ascent, ComponentStateMachine, Descent, FsmState};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
};

impl ComponentStateMachine<FsmState> for Ascent {
    fn update(
        &mut self,
        fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.begun_falling(fcu) {
            return Some(Descent::new());
        }

        None
    }

    fn enter_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn hal_state(&self) -> VehicleState {
        VehicleState::Ascent
    }
}

impl Ascent {
    pub fn new() -> FsmState {
        FsmState::Ascent(Self {})
    }

    fn begun_falling(&mut self, fcu: &mut Fcu) -> bool {
        if fcu.state_vector.get_velocity().y < 0.0 {
            return true;
        }

        false
    }
}
