use nalgebra::{SMatrix, SVector};

pub struct KalmanFilter<const STATE_LEN: usize, const MEASURE_LEN: usize> {
    pub state: SVector<f32, STATE_LEN>,
    pub process_noise_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
    pub measurement_noise_cov: SMatrix<f32, MEASURE_LEN, MEASURE_LEN>,
    pub state_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
}

impl<const STATE_LEN: usize, const MEASURE_LEN: usize> KalmanFilter<STATE_LEN, MEASURE_LEN> {
    pub fn new(
        initial_state: SVector<f32, STATE_LEN>,
        process_noise_cov: SMatrix<f32, STATE_LEN, STATE_LEN>,
        measurement_noise_cov: SMatrix<f32, MEASURE_LEN, MEASURE_LEN>,
        initial_state_cov: SMatrix::<f32, STATE_LEN, STATE_LEN>,
    ) -> Self {
        KalmanFilter {
            state: initial_state,
            process_noise_cov,
            measurement_noise_cov,
            state_cov: initial_state_cov,
        }
    }

    pub fn predict(&mut self, dt: f32) {
        let state_transition = self.calculate_state_transition_matrix(dt);

        self.state = state_transition * &self.state;

        self.state_cov =
            state_transition
            * &self.state_cov
            * state_transition.transpose()
            + &self.process_noise_cov;
    }

    pub fn update(
        &mut self,
        measurement: &SVector<f32, MEASURE_LEN>,
        measurement_model: &SMatrix<f32, MEASURE_LEN, STATE_LEN>
    ) {
        let kalman_gain =
            &self.state_cov
            * measurement_model.transpose()
            * (&(measurement_model * &self.state_cov * measurement_model.transpose())
            + &self.measurement_noise_cov)
                .try_inverse()
                .expect("Failed to invert matrix");
        self.state = &self.state + kalman_gain * &(measurement - measurement_model * &self.state);
        self.state_cov = (SMatrix::<f32, STATE_LEN, STATE_LEN>::identity() - kalman_gain * measurement_model) * &self.state_cov;
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

        // Update accelerometer bias
        state[(9, 9)] = 1.0;
        state[(10, 10)] = 1.0;
        state[(11, 11)] = 1.0;

        state
    }
}