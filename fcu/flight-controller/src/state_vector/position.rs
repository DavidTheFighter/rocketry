use hal::fcu_hal::FcuConfig;
use nalgebra::{SMatrix, SVector, Vector3};
use num_traits::float::Float;

// state_vector = [x, y, z, vx, vy, vz, ax, ay, az]
// measure = [x, y, z, by, ax, ay, az]

pub(super) const STATE_LEN: usize = 9;
pub(super) const MEASURE_LEN: usize = 7;

pub struct PositionFilter {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub position_std_dev: Vector3<f32>,
    pub velocity_std_dev: Vector3<f32>,
    pub acceleration_std_dev: Vector3<f32>,
    pub state: SVector<f32, STATE_LEN>,
    pub state_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub process_noise_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub measurement_noise_cov: SMatrix<f32, MEASURE_LEN, MEASURE_LEN>,
}

impl PositionFilter {
    pub fn new(config: &FcuConfig) -> Self {
        let mut measurement_noise_cov = SMatrix::<f32, MEASURE_LEN, MEASURE_LEN>::zeros();
        measurement_noise_cov[(0, 0)] = config.gps_noise_std_dev.x.powi(2); // x
        measurement_noise_cov[(1, 1)] = config.gps_noise_std_dev.y.powi(2); // y
        measurement_noise_cov[(2, 2)] = config.gps_noise_std_dev.z.powi(2); // z
        measurement_noise_cov[(3, 3)] = config.barometer_noise_std_dev.powi(2); // baro
        measurement_noise_cov[(4, 4)] = config.accelerometer_noise_std_dev.x.powi(2); // ax
        measurement_noise_cov[(5, 5)] = config.accelerometer_noise_std_dev.y.powi(2); // ay
        measurement_noise_cov[(6, 6)] = config.accelerometer_noise_std_dev.z.powi(2); // az

        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            position_std_dev: Vector3::new(0.0, 0.0, 0.0),
            velocity_std_dev: Vector3::new(0.0, 0.0, 0.0),
            acceleration_std_dev: Vector3::new(0.0, 0.0, 0.0),
            state: SVector::<f32, STATE_LEN>::zeros(),
            state_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4,
            process_noise_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity()
                * config.position_kalman_process_variance,
            measurement_noise_cov,
        }
    }

    pub fn predict(&mut self, dt: f32) {
        let state_transition = self.calculate_state_transition_matrix(dt);

        self.state = state_transition * &self.state;

        self.state_cov = state_transition * &self.state_cov * state_transition.transpose()
            + &self.process_noise_cov;

        self.position = self.state.fixed_rows::<3>(0).into();
        self.velocity = self.state.fixed_rows::<3>(3).into();
        self.acceleration = self.state.fixed_rows::<3>(6).into();

        let errors = self.state_cov.diagonal().map(f32::sqrt);
        self.position_std_dev = errors.fixed_rows::<3>(0).into();
        self.velocity_std_dev = errors.fixed_rows::<3>(3).into();
        self.acceleration_std_dev = errors.fixed_rows::<3>(6).into();
    }

    pub fn update(
        &mut self,
        measurement: &SVector<f32, MEASURE_LEN>,
        measurement_model: &SMatrix<f32, MEASURE_LEN, STATE_LEN>,
    ) {
        let kalman_gain = &self.state_cov
            * measurement_model.transpose()
            * (&(measurement_model * &self.state_cov * measurement_model.transpose())
                + &self.measurement_noise_cov)
                .try_inverse()
                .expect("Failed to invert matrix");
        self.state = &self.state + kalman_gain * &(measurement - measurement_model * &self.state);
        self.state_cov = (SMatrix::<f32, STATE_LEN, STATE_LEN>::identity()
            - kalman_gain * measurement_model)
            * &self.state_cov;
    }

    pub fn zero(&mut self) {
        self.position = Vector3::new(0.0, 0.0, 0.0);
        self.velocity = Vector3::new(0.0, 0.0, 0.0);
        self.acceleration = Vector3::new(0.0, 0.0, 0.0);
        self.position_std_dev = Vector3::new(0.0, 0.0, 0.0);
        self.velocity_std_dev = Vector3::new(0.0, 0.0, 0.0);
        self.acceleration_std_dev = Vector3::new(0.0, 0.0, 0.0);

        self.state = SVector::<f32, STATE_LEN>::zeros();
        self.state_cov = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4;
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(4).copy_from(&acceleration);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(4, 6)] = 1.0;
        measurement_matrix[(5, 7)] = 1.0;
        measurement_matrix[(6, 8)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    pub fn update_barometric_pressure(&mut self, barometric_altitude: f32) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement[3] = barometric_altitude;

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(3, 1)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(0).copy_from(&gps);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(0, 0)] = 1.0;
        measurement_matrix[(1, 1)] = 1.0;
        measurement_matrix[(2, 2)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    fn calculate_state_transition_matrix(&self, dt: f32) -> SMatrix<f32, STATE_LEN, STATE_LEN> {
        let mut state = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity();

        // Update position from velocity and acceleration
        state[(0, 3)] = dt;
        state[(0, 6)] = 0.5 * dt * dt;
        state[(1, 4)] = dt;
        state[(1, 7)] = 0.5 * dt * dt;
        state[(2, 5)] = dt;
        state[(2, 8)] = 0.5 * dt * dt;

        // Update velocity from acceleration
        state[(3, 6)] = dt;
        state[(4, 7)] = dt;
        state[(5, 8)] = dt;

        state
    }
}
