use super::{Calibrating, ComponentStateMachine, FsmState, Idle, Armed};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{VehicleState, self},
};

impl ComponentStateMachine<FsmState> for Idle {
    fn update<'a>(
        &mut self,
        fcu: &'a mut Fcu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if let Some(zero) = self.received_start_calibration(packets) {
            return Some(Calibrating::new(fcu, zero));
        } else if self.received_arming_command(packets) {
            return Some(Armed::new());
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
        VehicleState::Idle
    }
}

impl Idle {
    pub fn new() -> FsmState {
        FsmState::Idle(Self {})
    }

    fn received_arming_command(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::ArmVehicle { magic_number } = packet {
                return *magic_number == fcu_hal::IGNITION_MAGIC_NUMBER;
            }
        }

        false
    }

    fn received_start_calibration(&self, packets: &[(NetworkAddress, Packet)]) -> Option<bool> {
        for (_address, packet) in packets {
            if let Packet::StartCalibration { zero } = packet {
                return Some(*zero);
            }
        }

        None
    }
}
