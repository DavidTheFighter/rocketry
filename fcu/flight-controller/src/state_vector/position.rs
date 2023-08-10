use hal::fcu_hal::FcuConfig;
use nalgebra::{Vector3, SMatrix, SVector};
use num_traits::float::Float;

use super::{StateVector, kalman::KalmanFilter};

// state_vector = [x, y, z, vx, vy, vz, ax, ay, az]
// measure = [x, y, z, by, ax, ay, az]

pub(super) const STATE_LEN: usize = 9;
pub(super) const MEASURE_LEN: usize = 7;

impl StateVector {
    pub(super) fn predict_position(&mut self, dt: f32) {
        // self.velocity += self.acceleration * dt;
        // self.position += self.velocity * dt;

        self.position_kalman.predict(dt);

        self.position = self.position_kalman.state.fixed_rows::<3>(0).into();
        self.velocity = self.position_kalman.state.fixed_rows::<3>(3).into();
        self.acceleration = self.position_kalman.state.fixed_rows::<3>(6).into();

        let errors = self.position_kalman.state_cov.diagonal().map(f32::sqrt);
        self.position_std_dev = errors.fixed_rows::<3>(0).into();
        self.velocity_std_dev = errors.fixed_rows::<3>(3).into();
        self.acceleration_std_dev = errors.fixed_rows::<3>(6).into();
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>) {
        let acceleration =  self.orientation.orientation.transform_vector(&acceleration);

        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(4).copy_from(&acceleration);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(4, 6)] = 1.0;
        measurement_matrix[(5, 7)] = 1.0;
        measurement_matrix[(6, 8)] = 1.0;

        self.position_kalman.update(&measurement, &measurement_matrix);
    }

    pub fn update_barometric_pressure(&mut self, barometric_pressure: f32) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement[3] = barometric_pressure;

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(3, 1)] = 1.0;

        self.position_kalman.update(&measurement, &measurement_matrix);
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(0).copy_from(&gps);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(0, 0)] = 1.0;
        measurement_matrix[(1, 1)] = 1.0;
        measurement_matrix[(2, 2)] = 1.0;

        self.position_kalman.update(&measurement, &measurement_matrix);
    }

    pub(super) fn update_config_position(&mut self, config: &FcuConfig) {
        let old_state = self.position_kalman.state;

        self.position_kalman = Self::init_position_kalman(config);
        self.position_kalman.state = old_state;
    }

    pub(super) fn init_position_kalman(config: &FcuConfig) -> KalmanFilter<STATE_LEN, MEASURE_LEN> {
        let mut measurement_noise = SMatrix::<f32, MEASURE_LEN, MEASURE_LEN>::zeros();
        measurement_noise[(0, 0)] = config.gps_noise_std_dev.x.powi(2);                 // x
        measurement_noise[(1, 1)] = config.gps_noise_std_dev.y.powi(2);                 // y
        measurement_noise[(2, 2)] = config.gps_noise_std_dev.z.powi(2);                 // z
        measurement_noise[(3, 3)] = config.barometer_noise_std_dev.powi(2);             // baro
        measurement_noise[(4, 4)] = config.accelerometer_noise_std_dev.x.powi(2);       // ax
        measurement_noise[(5, 5)] = config.accelerometer_noise_std_dev.y.powi(2);       // ay
        measurement_noise[(6, 6)] = config.accelerometer_noise_std_dev.z.powi(2);       // az

        KalmanFilter::new(
            SMatrix::<f32, STATE_LEN, 1>::zeros(),
            SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * config.position_kalman_process_variance,
            measurement_noise,
            SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4,
        )
    }
}