use nalgebra::{SMatrix, SVector, Quaternion};

// state_vector = [x, y, z, vx, vy, vz, ax, ay, az, qw, qx, qy, qz, wx, wy, wz]
// measure = [x, y, z, ax, ay, az, wx, wy, wz]

const STATE_LENGTH: usize = 16;
const MEASURE_LENGTH: usize = 12;

pub struct KalmanFilter {
    pub state: SVector<f32, STATE_LENGTH>,
    pub process_noise_cov: SMatrix<f32, STATE_LENGTH, STATE_LENGTH>,
    pub measurement_noise_cov: SMatrix<f32, MEASURE_LENGTH, MEASURE_LENGTH>,
    pub state_cov: SMatrix<f32, STATE_LENGTH, STATE_LENGTH>,
}

impl KalmanFilter {
    pub fn new(
        initial_state: SVector<f32, STATE_LENGTH>,
        process_noise_cov: SMatrix<f32, STATE_LENGTH, STATE_LENGTH>,
        measurement_noise_cov: SMatrix<f32, MEASURE_LENGTH, MEASURE_LENGTH>,
    ) -> Self {
        KalmanFilter {
            state: initial_state,
            process_noise_cov,
            measurement_noise_cov,
            state_cov: SMatrix::<f32, STATE_LENGTH, STATE_LENGTH>::identity(),
        }
    }

    pub fn predict(&mut self, dt: f32) {
        let state_transition = self.calculate_state_transition_matrix(dt);

        self.state = state_transition * &self.state;

        // Normalize the orientation quaternion
        let orientation = Quaternion::new(
            self.state[9],
            self.state[10],
            self.state[11],
            self.state[12],
        ).normalize();

        self.state[9] = orientation.w;
        self.state[10] = orientation.i;
        self.state[11] = orientation.j;
        self.state[12] = orientation.k;

        self.state_cov =
            state_transition
            * &self.state_cov
            * state_transition.transpose()
            + &self.process_noise_cov;
    }

    pub fn update(
        &mut self,
        measurement: &SVector<f32, MEASURE_LENGTH>,
        measurement_model: &SMatrix<f32, MEASURE_LENGTH, STATE_LENGTH>
    ) {
        let kalman_gain =
            &self.state_cov
            * measurement_model.transpose()
            * (&(measurement_model * &self.state_cov * measurement_model.transpose())
            + &self.measurement_noise_cov)
                .try_inverse()
                .unwrap();
        self.state = &self.state + kalman_gain * &(measurement - measurement_model * &self.state);
        self.state_cov = (SMatrix::<f32, STATE_LENGTH, STATE_LENGTH>::identity() - kalman_gain * measurement_model) * &self.state_cov;
    }

    fn calculate_state_transition_matrix(&self, dt: f32) -> SMatrix<f32, STATE_LENGTH, STATE_LENGTH> {
        let mut state = SMatrix::<f32, STATE_LENGTH, STATE_LENGTH>::identity();

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

        // Update orientation from angular velocity
        let w = -0.5 * dt * SMatrix::<f32, 4, 4>::new(
            0.0, -self.state[13], -self.state[14], -self.state[15],
            self.state[13], 0.0, self.state[15], -self.state[14],
            self.state[14], -self.state[15], 0.0, self.state[13],
            self.state[15], self.state[14], -self.state[13], 0.0,
        );

        for x in 9..13 {
            for y in 9..13 {
                state[(x, y)] = w[(x - 9, y - 9)];
            }
        }

        state
    }
}