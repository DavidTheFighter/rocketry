use shared::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use crate::Fcu;
use super::{ComponentStateMachine, FsmState, Ignition, Ascent};

impl ComponentStateMachine<FsmState> for Ignition {
    fn update<'a>(&mut self, fcu: &'a mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<FsmState> {
        if self.begun_accelerating(fcu) {
            fcu.state_vector.set_landed(false);
            return Some(Ascent::new());
        }

        None
    }

    fn enter_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // TODO Ignite motor
    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        todo!()
    }

    fn hal_state(&self) -> VehicleState {
        todo!()
    }
}

impl Ignition {
    pub fn new() -> FsmState {
        FsmState::Ignition(Ignition { })
    }

    fn begun_accelerating(&self, fcu: &mut Fcu) -> bool {
        let acceleration = fcu.state_vector.get_acceleration().magnitude();
        // silprintln!("Idle: Time {} - landed? {} - accel {:?} - accel sense {:?}", fcu.driver.timestamp(), fcu.state_vector.get_landed(), fcu.state_vector.get_acceleration(), fcu.state_vector.sensor_data.accelerometer);
        if acceleration > fcu.config.startup_acceleration_threshold {
            return true;
        }

        false
    }
}
