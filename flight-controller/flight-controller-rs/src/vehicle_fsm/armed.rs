use shared::{fcu_hal::{VehicleState, self}, comms_hal::{Packet, NetworkAddress}};
use crate::Fcu;
use super::{ComponentStateMachine, FsmState, Armed, Ignition};

impl ComponentStateMachine<FsmState> for Armed {
    fn update<'a>(&mut self, _fcu: &'a mut Fcu, _dt: f32, packets: &[(NetworkAddress, Packet)]) -> Option<FsmState> {
        if self.received_ignition_command(packets) {
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
        todo!()
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
}
