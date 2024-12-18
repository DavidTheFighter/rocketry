use super::{Ascent, Descent, FsmState};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

impl<'f> ControllerState<FsmState, Fcu<'f>> for Ascent {
    fn update(
        &mut self,
        fcu: &mut Fcu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        self.time_since_state_entry += dt;

        if self.begun_falling(fcu) {
            return Some(Descent::new());
        }

        None
    }

    fn enter_state(&mut self, _fcu: &mut Fcu) {
        // Nothing
    }

    fn exit_state(&mut self, _fcu: &mut Fcu) {
        // Nothing
    }
}

impl Ascent {
    pub fn new() -> FsmState {
        FsmState::Ascent(Self {
            time_since_state_entry: 0.0,
        })
    }

    fn begun_falling(&mut self, fcu: &mut Fcu) -> bool {
        if fcu.state_vector.get_position().y < 5.0 {
            return false;
        }

        if fcu.state_vector.get_velocity().y < 0.0 {
            return true;
        }

        false
    }
}
