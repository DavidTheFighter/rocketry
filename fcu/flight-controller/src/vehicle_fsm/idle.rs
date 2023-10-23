use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}, GRAVITY};
use crate::{FiniteStateMachine, Fcu};
use super::{Idle, FsmStorage};

impl FiniteStateMachine<VehicleState> for Idle {
    fn update(fcu: &mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<VehicleState> {
        let begun_accelerating = Idle::begun_accelerating(fcu);
        let should_start_calibration = Idle::should_start_calibration(_packets);

        if begun_accelerating {
            return Some(VehicleState::Ascent);
        } else if should_start_calibration {
            return Some(VehicleState::Calibrating);
        }

        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Idle(Idle {});
    }
}

impl Idle {
    fn begun_accelerating(fcu: &mut Fcu) -> bool {
        let acceleration = fcu.state_vector.get_acceleration().magnitude();
        if acceleration - GRAVITY > fcu.config.startup_acceleration_threshold {
            return true;
        }

        false
    }

    fn should_start_calibration(packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::StartCalibration = packet {
                return true;
            }
        }

        false
    }
}