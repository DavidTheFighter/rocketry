use super::{Calibrating, FsmState, Idle};
use crate::{state_vector::SensorCalibrationData, Fcu};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ControllerState, standard_atmosphere::convert_pressure_to_altitude,
};
use nalgebra::{Vector3, UnitVector3, UnitQuaternion};

#[allow(unused_imports)]
use num_traits::Float;

impl<'f> ControllerState<FsmState, Fcu<'f>> for Calibrating {
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

    fn enter_state(&mut self, _fcu: & mut Fcu) {
        // Nothing
    }

    fn exit_state(&mut self, fcu: & mut Fcu) {
        let mut accelerometer_avg = self.accelerometer / (self.data_count as f32);
        let down = accelerometer_avg.normalize();
        let acceleration_by_gravity = down * 9.80665;

        accelerometer_avg -= acceleration_by_gravity;

        silprintln!("Accel calib: {:?}", accelerometer_avg);

        let sensor_calibration = SensorCalibrationData {
            accelerometer: -accelerometer_avg,
            gyroscope: -self.gyroscope / (self.data_count as f32),
            magnetometer: -self.magnetometer / (self.data_count as f32),
            barometeric_altitude: -self.barometric_altitude / (self.data_count as f32),
        };

        fcu.state_vector.update_calibration(sensor_calibration);

        if self.zero {
            let up = UnitVector3::new_normalize(Vector3::new(0.0, 1.0, 0.0).normalize());
            let measured_up = UnitVector3::new_normalize(down);

            let dot = measured_up.dot(&up);
            let angle = dot.acos();

            if angle.abs() < f32::EPSILON {
                let zeroed_orientation = UnitQuaternion::identity();

                fcu.state_vector.position_filter.zero();
                fcu.state_vector.orientation_filter.zero(zeroed_orientation);
            } else {
                let axis = UnitVector3::new_normalize(measured_up.cross(&up));

                let zeroed_orientation = UnitQuaternion::from_axis_angle(&axis, angle);

                fcu.state_vector.position_filter.zero();
                fcu.state_vector.orientation_filter.zero(zeroed_orientation);
            }
        }
    }
}

impl Calibrating {
    pub fn new<'a>(fcu: & mut Fcu, zero: bool) -> FsmState {
        FsmState::Calibrating(Self {
            start_time: fcu.driver.timestamp(),
            accelerometer: Vector3::zeros(),
            gyroscope: Vector3::zeros(),
            magnetometer: Vector3::zeros(),
            barometric_altitude: 0.0,
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
        let baro_altitude = fcu.state_vector.sensor_data.barometer_altitude;

        self.accelerometer += Vector3::<f32>::from(accelerometer);
        self.gyroscope += Vector3::<f32>::from(gyroscope);
        self.magnetometer += Vector3::<f32>::from(magnetometer);
        self.barometric_altitude += baro_altitude;

        self.data_count += 1;
    }
}
