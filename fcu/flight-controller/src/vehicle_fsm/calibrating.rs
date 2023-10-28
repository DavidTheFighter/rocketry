use super::{Calibrating, ComponentStateMachine, FsmState, Idle};
use crate::{silprintln, state_vector::SensorCalibrationData, Fcu};
use hal::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
};
use nalgebra::{Vector3, UnitVector3, UnitQuaternion};

impl ComponentStateMachine<FsmState> for Calibrating {
    fn update(
        &mut self,
        fcu: &mut Fcu,
        _dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<FsmState> {
        if self.calibration_time_ended(fcu) {
            return Some(Idle::new());
        }

        self.accumulate_sensor_data(fcu);

        None
    }

    fn enter_state<'a>(&mut self, _fcu: &'a mut Fcu) {
        // Nothing
    }

    fn exit_state<'a>(&mut self, fcu: &'a mut Fcu) {
        let mut accelerometer_avg = self.accelerometer / (self.data_count as f32);
        let down = Vector3::new(0.0, -1.0, 0.0);//accelerometer_avg.normalize();
        let acceleration_by_gravity = down * 9.80665;

        accelerometer_avg -= acceleration_by_gravity;

        silprintln!("Accel calib: {:?}", accelerometer_avg);

        let sensor_calibration = SensorCalibrationData {
            accelerometer: -accelerometer_avg,
            gyroscope: -self.gyroscope / (self.data_count as f32),
            magnetometer: -self.magnetometer / (self.data_count as f32),
            barometer_pressure: -self.barometer_pressure / (self.data_count as f32),
        };

        fcu.state_vector.update_calibration(sensor_calibration);

        if self.zero {
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

    fn hal_state(&self) -> VehicleState {
        VehicleState::Calibrating
    }
}

impl Calibrating {
    pub fn new<'a>(fcu: &'a mut Fcu, zero: bool) -> FsmState {
        FsmState::Calibrating(Self {
            start_time: fcu.driver.timestamp(),
            accelerometer: Vector3::zeros(),
            gyroscope: Vector3::zeros(),
            magnetometer: Vector3::zeros(),
            barometer_pressure: 0.0,
            data_count: 0,
            zero,
        })
    }

    fn calibration_time_ended(&mut self, fcu: &mut Fcu) -> bool {
        let elapsed_time = fcu.driver.timestamp() - self.start_time;

        elapsed_time >= fcu.config.calibration_duration
    }

    fn accumulate_sensor_data(&mut self, fcu: &mut Fcu) {
        let accelerometer = fcu.state_vector.sensor_data.accelerometer;
        let gyroscope = fcu.state_vector.sensor_data.gyroscope;
        let magnetometer = fcu.state_vector.sensor_data.magnetometer;
        let barometer_pressure = fcu.state_vector.sensor_data.barometer_pressure;

        self.accelerometer += Vector3::<f32>::from(accelerometer);
        self.gyroscope += Vector3::<f32>::from(gyroscope);
        self.magnetometer += Vector3::<f32>::from(magnetometer);
        self.barometer_pressure += barometer_pressure;

        self.data_count += 1;
    }
}
