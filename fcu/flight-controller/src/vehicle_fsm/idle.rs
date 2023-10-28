use super::{Calibrating, ComponentStateMachine, FsmState, Idle, Ascent};
use crate::{silprintln, Fcu};
use hal::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
    GRAVITY,
};

impl ComponentStateMachine<FsmState> for Idle {
    fn update<'a>(
        &mut self,
        fcu: &'a mut Fcu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.begun_accelerating(fcu) {
            fcu.state_vector.set_landed(false);
            return Some(Ascent::new());
        } else if let Some(zero) = self.received_start_calibration(packets) {
            return Some(Calibrating::new(fcu, zero));
        }

        None
    }

    fn enter_state<'a>(&mut self, _fcu: &'a mut Fcu) {

    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        silprintln!("vehicle_fms: Idle -> Exit");
    }

    fn hal_state(&self) -> VehicleState {
        VehicleState::Idle
    }
}

impl Idle {
    pub fn new() -> FsmState {
        FsmState::Idle(Self {})
    }

    fn begun_accelerating(&self, fcu: &mut Fcu) -> bool {
        let acceleration = fcu.state_vector.get_acceleration().magnitude();
        // silprintln!("Idle: Time {} - landed? {} - accel {:?} - accel sense {:?}", fcu.driver.timestamp(), fcu.state_vector.get_landed(), fcu.state_vector.get_acceleration(), fcu.state_vector.sensor_data.accelerometer);
        if acceleration > fcu.config.startup_acceleration_threshold {
            return true;
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
