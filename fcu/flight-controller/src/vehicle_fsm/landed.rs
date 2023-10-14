use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use crate::{FiniteStateMachine, Fcu};
use super::{FsmStorage, Landed};

impl FiniteStateMachine<VehicleState> for Landed {
    fn update(_fcu: &mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<VehicleState> {
        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Landed(Landed {});
    }
}