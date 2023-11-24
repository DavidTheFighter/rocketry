use super::{FsmState, Landed};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState,
};

impl<'f> ControllerState<FsmState, Fcu<'f>> for Landed {
    fn update(
        &mut self,
        _fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        None
    }

    fn enter_state(&mut self, fcu: & mut Fcu) {
        fcu.state_vector.set_landed(true);
    }

    fn exit_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }
}

impl Landed {
    pub fn new() -> FsmState {
        FsmState::Landed(Self {})
    }
}
