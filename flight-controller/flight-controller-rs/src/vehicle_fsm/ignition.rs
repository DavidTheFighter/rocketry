use super::{Ascent, FsmState, Ignition};
use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::OutputChannel,
    ControllerState,
};

impl<'f> ControllerState<FsmState, Fcu<'f>> for Ignition {
    fn update<'a>(
        &mut self,
        fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.begun_accelerating(fcu) {
            fcu.state_vector.set_landed(false);
            return Some(Ascent::new());
        }

        None
    }

    fn enter_state(&mut self, fcu: &mut Fcu) {
        fcu.driver
            .set_output_channel(OutputChannel::SolidMotorIgniter, true);
    }

    fn exit_state(&mut self, fcu: &mut Fcu) {
        fcu.driver
            .set_output_channel(OutputChannel::SolidMotorIgniter, false);
    }
}

impl Ignition {
    pub fn new() -> FsmState {
        FsmState::Ignition(Ignition {})
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
