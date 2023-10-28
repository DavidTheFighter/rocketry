use super::{ComponentStateMachine, FsmState, Idle, Zeroing};
use crate::Fcu;
use hal::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
};
use nalgebra::{UnitQuaternion, UnitVector3, Vector3};
use num_traits::Float;

impl ComponentStateMachine<FsmState> for Zeroing {
    fn update(
        &mut self,
        fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.zeroing_time_ended(fcu) {
            self.finish_zeroing(fcu);

            return Some(Idle::new());
        }

        self.accumulate_sensor_data(fcu);

        None
    }

    fn enter_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn exit_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn hal_state(&self) -> VehicleState {
        VehicleState::Zeroing
    }
}

impl Zeroing {
    pub fn new() -> FsmState {
        FsmState::Zeroing(Self {
            start_time: 0.0,
            accelerometer: Vector3::zeros(),
        })
    }

    fn zeroing_time_ended(&mut self, fcu: &mut Fcu) -> bool {
        let elapsed_time = fcu.driver.timestamp() - self.start_time;

        elapsed_time >= fcu.config.calibration_duration
    }

    fn accumulate_sensor_data(&mut self, fcu: &mut Fcu) {
        let accelerometer = fcu.state_vector.sensor_data.accelerometer;

        self.accelerometer +=
            Vector3::<f32>::from(accelerometer) + fcu.state_vector.sensor_calibration.accelerometer;
    }

    fn finish_zeroing(&mut self, fcu: &mut Fcu) {
        let down = UnitVector3::new_normalize(Vector3::new(0.0, -1.0, 0.0).normalize());
        let measured_down = UnitVector3::new_normalize(self.accelerometer.normalize());

        let dot = down.dot(&measured_down);
        let angle = dot.acos();

        if angle.abs() < f32::EPSILON {
            let zeroed_orientation = UnitQuaternion::identity();

            fcu.state_vector.position_filter.zero();
            fcu.state_vector.orientation_filter.zero(zeroed_orientation);
        } else {
            let axis = UnitVector3::new_normalize(down.cross(&measured_down));

            let zeroed_orientation = UnitQuaternion::from_axis_angle(&axis, angle);

            fcu.state_vector.position_filter.zero();
            fcu.state_vector.orientation_filter.zero(zeroed_orientation);
        }
    }
}
