use super::{Calibrating, FsmState, Idle, Armed};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal, ControllerState,
};

impl<'f> ControllerState<FsmState, Fcu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        fcu: & mut Fcu,
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

    fn enter_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }

    fn exit_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }
}

impl Idle {
    pub fn new() -> FsmState {
        FsmState::Idle(Self {})
    }

    fn received_arming_command(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::ArmVehicle { magic_number } = packet {
                return *magic_number == fcu_hal::ARMING_MAGIC_NUMBER;
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

#[cfg(test)]
mod tests {
    use shared::{fcu_hal, comms_hal::NetworkAddress};

    use crate::vehicle_fsm::Idle;

    #[test]
    fn test_no_packets_arming() {
        let state = Idle {};
        let packets = vec![];

        assert_eq!(state.received_arming_command(&packets), false);
    }

    #[test]
    fn test_arming_packet_good() {
        let state = Idle {};
        let packets = vec![(NetworkAddress::MissionControl, shared::comms_hal::Packet::ArmVehicle { magic_number: fcu_hal::ARMING_MAGIC_NUMBER })];

        assert_eq!(state.received_arming_command(&packets), true);
    }

    #[test]
    fn test_arming_packet_bad_magic_number() {
        let state = Idle {};
        let packets = vec![(NetworkAddress::MissionControl, shared::comms_hal::Packet::ArmVehicle { magic_number: 0xdeadbeef })];

        assert_eq!(state.received_arming_command(&packets), false);
    }
}