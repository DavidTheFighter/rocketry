use nalgebra::{Quaternion, SMatrix, SVector, UnitQuaternion, Vector3, Vector4};
use num_traits::Float;
use serde::{Deserialize, Serialize};
use shared::fcu_hal::FcuConfig;
// state_vector = [x, y, z, vx, vy, vz, ax, ay, az, w, i, j, k, avx, avy, avz]
// measure = [x, y, z, by, ax, ay, az, avx, avy, avz]

pub(super) const STATE_LEN: usize = 16;
pub(super) const MEASURE_LEN: usize = 10;
pub(super) const SIGMA_BASE: usize = STATE_LEN;
pub(super) const SIGMA_LEN: usize = 2 * SIGMA_BASE + 1;
pub(super) const SIGMA_LEN_MINUS_1: usize = SIGMA_LEN - 1;

const ALPHA: f32 = 1.0;
const BETA: f32 = 2.0;
const KAPPA: f32 = 0.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalmanFilter {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub position_std_dev: Vector3<f32>,
    pub velocity_std_dev: Vector3<f32>,
    pub acceleration_std_dev: Vector3<f32>,
    pub orientation_std_dev: Vector4<f32>,
    pub angular_velocity_std_dev: Vector3<f32>,
    pub state: SVector<f32, STATE_LEN>,
    pub state_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub process_noise_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub measurement_noise_cov: SMatrix<f32, MEASURE_LEN, MEASURE_LEN>,
    pub wm: SVector<f32, SIGMA_LEN>,
    pub wc: SVector<f32, SIGMA_LEN>,
    pub sigma_scaling: f32,
}

impl KalmanFilter {
    pub fn new(config: &FcuConfig) -> Self {
        let mut initial_state = SVector::<f32, STATE_LEN>::zeros();
        initial_state[9] = 1.0;

        let mut measurement_noise_cov = SMatrix::<f32, MEASURE_LEN, MEASURE_LEN>::zeros();
        measurement_noise_cov[(0, 0)] = config.gps_noise_std_dev.x.powi(2); // x
        measurement_noise_cov[(1, 1)] = config.gps_noise_std_dev.y.powi(2); // y
        measurement_noise_cov[(2, 2)] = config.gps_noise_std_dev.z.powi(2); // z
        measurement_noise_cov[(3, 3)] = config.barometer_noise_std_dev.powi(2); // baro
        measurement_noise_cov[(4, 4)] = config.accelerometer_noise_std_dev.x.powi(2); // ax
        measurement_noise_cov[(5, 5)] = config.accelerometer_noise_std_dev.y.powi(2); // ay
        measurement_noise_cov[(6, 6)] = config.accelerometer_noise_std_dev.z.powi(2); // az
        measurement_noise_cov[(7, 7)] = config.gyro_noise_std_dev.x.powi(2); // avx
        measurement_noise_cov[(8, 8)] = config.gyro_noise_std_dev.y.powi(2); // avy
        measurement_noise_cov[(9, 9)] = config.gyro_noise_std_dev.z.powi(2); // avz

        let mut wm = SVector::<f32, SIGMA_LEN>::zeros();
        let mut wc = SVector::<f32, SIGMA_LEN>::zeros();

        wm[0] = ALPHA.powi(2) / (ALPHA.powi(2) + (SIGMA_BASE as f32));
        wc[0] = ALPHA.powi(2) / (ALPHA.powi(2) + (SIGMA_BASE as f32));

        for i in 1..SIGMA_LEN {
            wm[i] = 1.0 / (2.0 * (ALPHA.powi(2) + (SIGMA_BASE as f32)));
            wc[i] = 1.0 / (2.0 * (ALPHA.powi(2) + (SIGMA_BASE as f32)));
        }

        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            orientation: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
            position_std_dev: Vector3::zeros(),
            velocity_std_dev: Vector3::zeros(),
            acceleration_std_dev: Vector3::zeros(),
            orientation_std_dev: Vector4::zeros(),
            angular_velocity_std_dev: Vector3::zeros(),
            state: initial_state,
            state_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-1,
            process_noise_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity()
                * config.kalman_process_variance,
            measurement_noise_cov,
            wm,
            wc,
            sigma_scaling: ALPHA.powi(2) * ((STATE_LEN as f32) + KAPPA),
        }
    }

    pub fn predict(&mut self, dt: f32) {
        // println!("kalman.predict({})", dt);
        // println!("\t{:?}", self.state);
        // println!("\t{:?}", self.state_cov.diagonal());

        // Sigma points for update
        let sp_k_k = self.calc_sp(&self.state, &self.state_cov);

        // Apply dynamics model to the sigma points at k given k
        let mut sp_kp1_k = [SVector::<f32, STATE_LEN>::zeros(); SIGMA_LEN];
        for i in 0..SIGMA_LEN {
            sp_kp1_k[i] = self.dynamics_fn(&sp_k_k[i], dt);
        }

        // Obtain the state at time k+1 given k
        let mut x_kp1_k = SVector::<f32, STATE_LEN>::zeros();
        for i in 0..SIGMA_LEN {
            x_kp1_k += self.wm[i] * sp_kp1_k[i];
        }

        // Compute predicted state covariance at k+1 given k
        let mut cov_kp1_k = SMatrix::<f32, STATE_LEN, STATE_LEN>::zeros();
        for i in 0..SIGMA_LEN {
            let diff = sp_kp1_k[i] - x_kp1_k;
            cov_kp1_k += self.wc[i] * diff * diff.transpose();
        }

        // TODO Optimze to add along diagonal?
        cov_kp1_k += &self.process_noise_cov;

        self.state = x_kp1_k;
        self.state_cov = cov_kp1_k;

        // Update outward facing state

        for x in self.state.iter() {
            if x.is_nan() {
                panic!("Kalman state contains NaN {:?}", self.state);
            }
        }

        for x in self.state_cov.iter() {
            if x.is_nan() {
                panic!("Kalman state_cov contains NaN {:?}", self.state_cov);
            }
        }

        // println!("\t->");
        // println!("\t{:?}", self.state);
        // println!("\t{:?}", self.state_cov.diagonal());

        self.position = self.state.fixed_rows::<3>(0).into();
        self.velocity = self.state.fixed_rows::<3>(3).into();
        self.acceleration = self.state.fixed_rows::<3>(6).into();
        self.orientation = UnitQuaternion::from_quaternion(Quaternion::new(
            self.state[9],
            self.state[10],
            self.state[11],
            self.state[12],
        ));
        self.angular_velocity = self.state.fixed_rows::<3>(13).into();

        let errors = self.state_cov.diagonal().map(f32::sqrt);
        self.position_std_dev = errors.fixed_rows::<3>(0).into();
        self.velocity_std_dev = errors.fixed_rows::<3>(3).into();
        self.acceleration_std_dev = errors.fixed_rows::<3>(6).into();
        self.orientation_std_dev = errors.fixed_rows::<4>(9).into();
        self.angular_velocity_std_dev = errors.fixed_rows::<3>(13).into();
    }

    pub fn update(
        &mut self,
        measurement: &SVector<f32, MEASURE_LEN>,
        measurement_model: &SMatrix<f32, MEASURE_LEN, STATE_LEN>,
    ) {
        // println!("kalman.update()");
        // println!("\t{:?}", self.state);
        // println!("\t{:?}", self.state_cov.diagonal());

        // Sigma points for update from measurements at the current step k given k-1
        let sp_k_km1 = self.calc_sp(&self.state, &self.state_cov);

        // Apply measurement model to the sigma points to get predicted measurements at k given k-1
        let mut y_k_km1 = [SVector::<f32, MEASURE_LEN>::zeros(); SIGMA_LEN];
        for i in 0..SIGMA_LEN {
            y_k_km1[i] = measurement_model * &sp_k_km1[i];
        }

        // Combine predicted sigma points to get predicted measurement at k
        let mut y_k = SVector::<f32, MEASURE_LEN>::zeros();
        for i in 0..SIGMA_LEN {
            y_k += self.wm[i] * y_k_km1[i];
        }

        // Estimate covariance for the predicted measurement at k
        let mut cov_y = SMatrix::<f32, MEASURE_LEN, MEASURE_LEN>::zeros();
        for i in 0..SIGMA_LEN {
            let diff = y_k_km1[i] - y_k;
            cov_y += self.wc[i] * diff * diff.transpose();
        }
        cov_y += &self.measurement_noise_cov;

        // Estimate cross covariance between predicted measurement at k and state at k given k-1
        let mut p_xy = SMatrix::<f32, STATE_LEN, MEASURE_LEN>::zeros();
        for i in 1..SIGMA_LEN {
            p_xy += (sp_k_km1[i] - sp_k_km1[0]) * (y_k_km1[i] - y_k).transpose();
        }
        p_xy *= self.wm[1];

        // Calculate Kalman gain
        let kalman_gain = p_xy * cov_y.try_inverse().expect("Failed to invert matrix");

        // Calculate new state and covariance
        self.state = sp_k_km1[0] + kalman_gain * &(measurement - y_k);
        // self.state_cov = (SMatrix::<f32, STATE_LEN, STATE_LEN>::identity()
        //     - kalman_gain * measurement_model)
        //     * &self.state_cov;

        self.state_cov = &self.state_cov - kalman_gain * cov_y * kalman_gain.transpose();

        for x in self.state.iter() {
            if x.is_nan() {
                panic!("Kalman state contains NaN {:?}", self.state);
            }
        }

        for x in self.state_cov.iter() {
            if x.is_nan() {
                panic!("Kalman state_cov contains NaN {:?}", self.state_cov);
            }
        }

        // println!("\t->");
        // println!("\t{:?}", self.state);
        // println!("\t{:?}", self.state_cov.diagonal());

        // println!("{} = {} - {} * {} * {}", self.state_cov, prev_state_cov, kalman_gain, cov_y, kalman_gain.transpose());
    }

    pub fn zero(&mut self, zeroed_orientation: UnitQuaternion<f32>) {
        self.position = Vector3::zeros();
        self.velocity = Vector3::zeros();
        self.acceleration = Vector3::zeros();
        self.orientation = zeroed_orientation;
        self.angular_velocity = Vector3::zeros();
        self.position_std_dev = Vector3::zeros();
        self.velocity_std_dev = Vector3::zeros();
        self.acceleration_std_dev = Vector3::zeros();
        self.orientation_std_dev = Vector4::zeros();
        self.angular_velocity_std_dev = Vector3::zeros();

        self.state = SVector::<f32, STATE_LEN>::zeros();
        self.state[9] = zeroed_orientation.quaternion().w;
        self.state[10] = zeroed_orientation.quaternion().i;
        self.state[11] = zeroed_orientation.quaternion().j;
        self.state[12] = zeroed_orientation.quaternion().k;

        self.state_cov = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4;
    }

    pub fn update_config(&mut self, config: &FcuConfig) {
        silprintln!("Updating kalman config");
        self.process_noise_cov =
            SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * config.kalman_process_variance;
        self.measurement_noise_cov[(0, 0)] = config.gps_noise_std_dev.x.powi(2); // x
        self.measurement_noise_cov[(1, 1)] = config.gps_noise_std_dev.y.powi(2); // y
        self.measurement_noise_cov[(2, 2)] = config.gps_noise_std_dev.z.powi(2); // z
        self.measurement_noise_cov[(3, 3)] = config.barometer_noise_std_dev.powi(2); // baro
        self.measurement_noise_cov[(4, 4)] = config.accelerometer_noise_std_dev.x.powi(2); // ax
        self.measurement_noise_cov[(5, 5)] = config.accelerometer_noise_std_dev.y.powi(2); // ay
        self.measurement_noise_cov[(6, 6)] = config.accelerometer_noise_std_dev.z.powi(2); // az
        self.measurement_noise_cov[(7, 7)] = config.gps_noise_std_dev.x.powi(2); // avx
        self.measurement_noise_cov[(8, 8)] = config.gps_noise_std_dev.y.powi(2); // avy
        self.measurement_noise_cov[(9, 9)] = config.gps_noise_std_dev.z.powi(2);
        // avz
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>) {
        // println!("kalman.update_acceleration({:?})", acceleration);
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(4).copy_from(&acceleration);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(4, 6)] = 1.0;
        measurement_matrix[(5, 7)] = 1.0;
        measurement_matrix[(6, 8)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    pub fn update_barometric_pressure(&mut self, barometric_altitude: f32) {
        // println!("kalman.update_barometric_pressure({:?})", barometric_altitude);
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement[3] = barometric_altitude;

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(3, 1)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        // println!("kalman.update_gps({:?})", gps);
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement.fixed_rows_mut::<3>(0).copy_from(&gps);

        let mut measurement_matrix = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_matrix[(0, 0)] = 1.0;
        measurement_matrix[(1, 1)] = 1.0;
        measurement_matrix[(2, 2)] = 1.0;

        self.update(&measurement, &measurement_matrix);
    }

    pub fn update_gyroscope(&mut self, angular_velocity: Vector3<f32>) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement
            .fixed_rows_mut::<3>(7)
            .copy_from(&angular_velocity);

        let mut measurement_model = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_model[(7, 13)] = 1.0;
        measurement_model[(8, 14)] = 1.0;
        measurement_model[(9, 15)] = 1.0;

        self.update(&measurement, &measurement_model);
    }

    fn dynamics_fn(&self, state: &SVector<f32, STATE_LEN>, dt: f32) -> SVector<f32, STATE_LEN> {
        let mut new_state = state.clone();

        // Update velocity from acceleration
        new_state[3] += state[6] * dt;
        new_state[4] += state[7] * dt;
        new_state[5] += state[8] * dt;

        // Update position from velocity and acceleration
        new_state[0] += state[3] * dt + 0.5 * state[6] * dt * dt;
        new_state[1] += state[4] * dt + 0.5 * state[7] * dt * dt;
        new_state[2] += state[5] * dt + 0.5 * state[8] * dt * dt;

        // Update orientation from angular velocity
        let ang_vel = Vector3::new(state[13], state[14], state[15]);
        let orientation = UnitQuaternion::from_quaternion(Quaternion::new(
            state[9], state[10], state[11], state[12],
        ));
        let new_orientation = integrate_angular_velocity_rk4(orientation, ang_vel, dt);

        new_state[9] = new_orientation.quaternion().w;
        new_state[10] = new_orientation.quaternion().i;
        new_state[11] = new_orientation.quaternion().j;
        new_state[12] = new_orientation.quaternion().k;

        new_state
    }

    fn calc_sp(
        &self,
        state: &SVector<f32, STATE_LEN>,
        state_cov: &SMatrix<f32, STATE_LEN, STATE_LEN>,
    ) -> [SVector<f32, STATE_LEN>; SIGMA_LEN] {
        let mut sp = [SVector::<f32, STATE_LEN>::zeros(); SIGMA_LEN];
        let deltas = self.calc_sp_deltas(state_cov);
        sp[0] = *state;

        for i in 1..SIGMA_LEN {
            sp[i] = sp[0] + deltas[i - 1];
        }

        sp
    }

    fn calc_sp_deltas(
        &self,
        state_cov: &SMatrix<f32, STATE_LEN, STATE_LEN>,
    ) -> [SVector<f32, STATE_LEN>; SIGMA_LEN_MINUS_1] {
        let mut deltas = [SVector::<f32, STATE_LEN>::zeros(); SIGMA_LEN_MINUS_1];

        // println!("sigma_scaling = {}, state_cov = {}", self.sigma_scaling, state_cov.diagonal());
        let lower = (self.sigma_scaling * state_cov)
            .cholesky()
            .expect("Failed to compute cholesky")
            .l();

        for i in 0..STATE_LEN {
            deltas[i].set_column(0, &lower.column(i));
        }

        for i in STATE_LEN..(2 * STATE_LEN) {
            deltas[i].set_column(0, &lower.column(i - STATE_LEN));
            deltas[i] *= -1.0;
        }

        deltas
    }
}

fn integrate_angular_velocity_rk4(
    quat: UnitQuaternion<f32>,
    ang_vel: Vector3<f32>,
    dt: f32,
) -> UnitQuaternion<f32> {
    let quat = quat.quaternion();
    let k1 = q_dot(&quat, ang_vel);
    let k2 = q_dot(&(quat + 0.5 * dt * k1), ang_vel);
    let k3 = q_dot(&(quat + 0.5 * dt * k2), ang_vel);
    let k4 = q_dot(&(quat + dt * k3), ang_vel);

    let q_deriv = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    let result = quat + q_deriv * dt;

    UnitQuaternion::from_quaternion(result)
}

fn q_dot(quat: &Quaternion<f32>, ang_vel: Vector3<f32>) -> Quaternion<f32> {
    0.5 * Quaternion::new(0.0, ang_vel.x, ang_vel.y, ang_vel.z) * quat
}
