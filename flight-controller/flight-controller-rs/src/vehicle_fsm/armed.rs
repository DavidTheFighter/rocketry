use shared::{fcu_hal::{VehicleState, self, OutputChannel}, comms_hal::{Packet, NetworkAddress}};
use crate::Fcu;
use super::{ComponentStateMachine, FsmState, Armed, Ignition};

impl ComponentStateMachine<FsmState> for Armed {
    fn update<'a>(&mut self, fcu: &'a mut Fcu, _dt: f32, packets: &[(NetworkAddress, Packet)]) -> Option<FsmState> {
        if self.received_ignition_command(packets) && self.igniter_has_continuity(fcu) {
            return Some(Ignition::new());
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
        VehicleState::Armed
    }
}

impl Armed {
    pub fn new() -> FsmState {
        FsmState::Armed(Armed { })
    }

    fn received_ignition_command(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::IgniteSolidMotor { magic_number } = packet {
                return *magic_number == fcu_hal::IGNITION_MAGIC_NUMBER;
            }
        }

        false
    }

    fn igniter_has_continuity(&self, fcu: &mut Fcu) -> bool {
        fcu.driver.get_output_channel_continuity(OutputChannel::SolidMotorIgniter)
    }
}

#[cfg(test)]
mod tests {
    use shared::{comms_hal::NetworkAddress, fcu_hal};

    use crate::vehicle_fsm::Armed;

    #[test]
    fn test_no_packets_start_arming() {
        let state = Armed {};
        let packets = vec![];

        assert_eq!(state.received_ignition_command(&packets), false);
    }

    #[test]
    fn test_ignition_packet_good() {
        let state = Armed {};
        let packets = vec![(NetworkAddress::MissionControl, shared::comms_hal::Packet::IgniteSolidMotor { magic_number: fcu_hal::IGNITION_MAGIC_NUMBER })];

        assert_eq!(state.received_ignition_command(&packets), true);
    }

    #[test]
    fn test_ignition_packet_bad_magic_number() {
        let state = Armed {};
        let packets = vec![(NetworkAddress::MissionControl, shared::comms_hal::Packet::ArmVehicle { magic_number: 0xdeadbeef })];

        assert_eq!(state.received_ignition_command(&packets), false);
    }
}
