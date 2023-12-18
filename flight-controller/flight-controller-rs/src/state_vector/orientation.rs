use nalgebra::{Quaternion, SMatrix, SVector, UnitQuaternion, Vector3};
use serde::Serialize;

// state_vector = [w, i, j, k, avx, avy, avz]
// measure = [avx, avy, avz]

pub(super) const STATE_LEN: usize = 7;
pub(super) const MEASURE_LEN: usize = 3;

#[derive(Debug, Clone, Serialize)]
pub struct OrientationFilter {
    pub orientation: UnitQuaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub state: SVector<f32, STATE_LEN>,
    pub state_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub process_noise_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub measurement_noise_cov: SMatrix<f32, MEASURE_LEN, MEASURE_LEN>,
}

impl OrientationFilter {
    pub fn new() -> Self {
        let mut initial_state = SVector::<f32, STATE_LEN>::zeros();
        initial_state[0] = 1.0;

        Self {
            orientation: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            state: initial_state,
            state_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4,
            process_noise_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-3,
            measurement_noise_cov: SMatrix::<f32, MEASURE_LEN, MEASURE_LEN>::identity() * 1e-3,
        }
    }

    pub fn predict(&mut self, dt: f32) {
        let mut quat = Quaternion::new(self.state[0], self.state[1], self.state[2], self.state[3]);
        quat = quat / quat.norm();

        let angular_velocity = Vector3::new(self.state[4], self.state[5], self.state[6]);

        let k1 = q_dot(&quat, angular_velocity);
        let k2 = q_dot(&(quat + 0.5 * dt * k1), angular_velocity);
        let k3 = q_dot(&(quat + 0.5 * dt * k2), angular_velocity);
        let k4 = q_dot(&(quat + dt * k3), angular_velocity);

        let q_deriv = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        quat = quat + q_deriv * dt;
        quat = quat / quat.norm();

        self.state[0] = quat.w;
        self.state[1] = quat.i;
        self.state[2] = quat.j;
        self.state[3] = quat.k;

        let mut state_transition = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity();
        state_transition[(0, 4)] = 0.5 * dt * -angular_velocity.x;
        state_transition[(0, 5)] = 0.5 * dt * -angular_velocity.y;
        state_transition[(0, 6)] = 0.5 * dt * -angular_velocity.z;

        state_transition[(1, 4)] = 0.5 * dt * angular_velocity.x;
        state_transition[(1, 5)] = 0.5 * dt * -angular_velocity.y;
        state_transition[(1, 6)] = 0.5 * dt * angular_velocity.z;

        state_transition[(2, 4)] = 0.5 * dt * angular_velocity.y;
        state_transition[(2, 5)] = 0.5 * dt * angular_velocity.x;
        state_transition[(2, 6)] = 0.5 * dt * -angular_velocity.z;

        state_transition[(3, 4)] = 0.5 * dt * angular_velocity.z;
        state_transition[(3, 5)] = 0.5 * dt * angular_velocity.y;
        state_transition[(3, 6)] = 0.5 * dt * angular_velocity.x;

        self.state_cov = state_transition * &self.state_cov * state_transition.transpose()
            + &self.process_noise_cov;

        // Update our outward facing values
        self.orientation = UnitQuaternion::from_quaternion(quat);

        self.angular_velocity = Vector3::new(self.state[4], self.state[5], self.state[6]);
    }

    fn update(
        &mut self,
        measurement: SVector<f32, MEASURE_LEN>,
        measurement_model: SMatrix<f32, MEASURE_LEN, STATE_LEN>,
    ) {
        let y = measurement - measurement_model * &self.state;
        let s = measurement_model * &self.state_cov * measurement_model.transpose()
            + &self.measurement_noise_cov;
        let k = &self.state_cov
            * measurement_model.transpose()
            * s.try_inverse()
                .expect("Failed to invert s matrix for orientation");

        let ident = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity();

        self.state = &self.state + k * y;
        self.state_cov = (ident - k * measurement_model) * &self.state_cov;
    }

    pub fn zero(&mut self, zeroed_orientation: UnitQuaternion<f32>) {
        self.orientation = zeroed_orientation;
        self.angular_velocity = Vector3::new(0.0, 0.0, 0.0);

        self.state[0] = zeroed_orientation.w;
        self.state[1] = zeroed_orientation.i;
        self.state[2] = zeroed_orientation.j;
        self.state[3] = zeroed_orientation.k;
        self.state[4] = 0.0;
        self.state[5] = 0.0;
        self.state[6] = 0.0;

        self.state_cov = SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() * 1e-4;
    }

    pub fn update_gyroscope(&mut self, angular_velocity: Vector3<f32>) {
        let mut measurement = SVector::<f32, MEASURE_LEN>::zeros();
        measurement
            .fixed_rows_mut::<3>(0)
            .copy_from(&angular_velocity);

        let mut measurement_model = SMatrix::<f32, MEASURE_LEN, STATE_LEN>::zeros();
        measurement_model[(0, 4)] = 1.0;
        measurement_model[(1, 5)] = 1.0;
        measurement_model[(2, 6)] = 1.0;

        self.update(measurement, measurement_model);
    }

    pub fn update_magnetic_field(&mut self, _magnetic_field: Vector3<f32>) {
        // something
    }
}

fn q_dot(quat: &Quaternion<f32>, ang_vel: Vector3<f32>) -> Quaternion<f32> {
    0.5 * Quaternion::new(0.0, ang_vel.x, ang_vel.y, ang_vel.z) * quat
}
