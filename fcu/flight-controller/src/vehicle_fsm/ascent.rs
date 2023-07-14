use hal::{fcu_hal::VehicleState, comms_hal::Packet};
use crate::{FiniteStateMachine, Fcu};
use super::{FsmStorage, Ascent};

impl FiniteStateMachine<VehicleState> for Ascent {
    fn update(fcu: &mut Fcu, _dt: f32, _packet: &Option<Packet>) -> Option<VehicleState> {
        let begun_falling = Ascent::begun_falling(fcu);

        if begun_falling {
            return Some(VehicleState::Descent);
        }

        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Ascent(Ascent {});
    }
}

impl Ascent {
    fn begun_falling(fcu: &mut Fcu) -> bool {
        if fcu.state_vector.velocity.y < 0.0 {
            return true;
        }

        false
    }
}