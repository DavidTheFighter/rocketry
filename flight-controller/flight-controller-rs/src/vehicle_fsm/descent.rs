use super::{Descent, FsmState, Landed};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

impl<'f> ControllerState<FsmState, Fcu<'f>> for Descent {
    fn update(
        &mut self,
        fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.has_landed(fcu) {
            return Some(Landed::new());
        }

        None
    }

    fn enter_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }

    fn exit_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }
}

impl Descent {
    pub fn new() -> FsmState {
        FsmState::Descent(Self {})
    }

    fn has_landed(&mut self, fcu: &mut Fcu) -> bool {
        fcu.state_vector.get_position().y < 1e-3
    }
}
