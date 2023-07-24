use hal::{fcu_hal::VehicleState, comms_hal::Packet};
use crate::{FiniteStateMachine, Fcu};
use super::{FsmStorage, Descent};

impl FiniteStateMachine<VehicleState> for Descent {
    fn update(fcu: &mut Fcu, _dt: f32, _packets: &[Packet]) -> Option<VehicleState> {
        let has_landed = Descent::has_landed(fcu);

        if has_landed {
            return Some(VehicleState::Landed);
        }

        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Descent(Descent {});
    }
}

impl Descent {
    fn has_landed(fcu: &mut Fcu) -> bool {
        if fcu.state_vector.position.y < 1e-3 {
            return true;
        }

        false
    }
}