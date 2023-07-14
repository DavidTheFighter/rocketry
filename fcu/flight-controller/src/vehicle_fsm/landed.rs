use hal::{fcu_hal::VehicleState, comms_hal::Packet};
use crate::{FiniteStateMachine, Fcu};
use super::{FsmStorage, Landed};

impl FiniteStateMachine<VehicleState> for Landed {
    fn update(_fcu: &mut Fcu, _dt: f32, _packet: &Option<Packet>) -> Option<VehicleState> {
        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Landed(Landed {});
    }
}