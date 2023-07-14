use hal::fcu_hal::FcuConfig;
use nalgebra::{Vector3, UnitQuaternion};
use mint;

use self::kalman::KalmanFilter;

mod kalman;
pub mod orientation;
pub mod position;

pub struct StateVector {
    pub(crate) position: Vector3<f32>,
    pub(crate) position_std_dev: Vector3<f32>,
    pub(crate) velocity: Vector3<f32>,
    pub(crate) velocity_std_dev: Vector3<f32>,
    pub(crate) acceleration: Vector3<f32>,
    pub(crate) acceleration_std_dev: Vector3<f32>,
    pub(crate) accelerometer_bias: Vector3<f32>,
    pub(crate) accelerometer_bias_std_dev: Vector3<f32>,
    pub(crate) orientation: UnitQuaternion<f32>,
    pub(crate) angular_velocity: Vector3<f32>,
    pub(crate) angular_acceleration: Vector3<f32>,
    last_angular_velocity_timestamp: f32,
    position_kalman: KalmanFilter<{ position::STATE_LEN }, { position::MEASURE_LEN }>,
}

impl StateVector {
    pub fn new(config: &FcuConfig) -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            position_std_dev: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            velocity_std_dev: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            acceleration_std_dev: Vector3::new(0.0, 0.0, 0.0),
            accelerometer_bias: Vector3::new(0.0, 0.0, 0.0),
            accelerometer_bias_std_dev: Vector3::new(0.0, 0.0, 0.0),
            orientation: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_acceleration: Vector3::new(0.0, 0.0, 0.0),
            last_angular_velocity_timestamp: 0.0,
            position_kalman: Self::init_position_kalman(config),
        }
    }

    pub fn predict(&mut self, dt: f32) {
        self.predict_position(dt);
        self.predict_orientation(dt);
    }

    pub fn update_config(&mut self, config: &FcuConfig) {
        self.update_config_position(config);
        self.update_config_orientation(config);
    }

    pub fn get_position(&self) -> mint::Vector3<f32> {
        self.position.into()
    }

    pub fn get_position_std_dev(&self) -> mint::Vector3<f32> {
        self.position_std_dev.into()
    }

    pub fn get_position_std_dev_scalar(&self) -> f32 {
        self.position_std_dev.norm()
    }

    pub fn get_velocity(&self) -> mint::Vector3<f32> {
        self.velocity.into()
    }

    pub fn get_velocity_std_dev(&self) -> mint::Vector3<f32> {
        self.velocity_std_dev.into()
    }

    pub fn get_velocity_std_dev_scalar(&self) -> f32 {
        self.velocity_std_dev.norm()
    }

    pub fn get_acceleration(&self) -> mint::Vector3<f32> {
        self.acceleration.into()
    }

    pub fn get_acceleration_std_dev(&self) -> mint::Vector3<f32> {
        self.acceleration_std_dev.into()
    }

    pub fn get_acceleration_std_dev_scalar(&self) -> f32 {
        self.acceleration_std_dev.norm()
    }

    pub fn get_accelerometer_bias(&self) -> mint::Vector3<f32> {
        self.accelerometer_bias.into()
    }

    pub fn get_accelerometer_bias_std_dev(&self) -> mint::Vector3<f32> {
        self.accelerometer_bias_std_dev.into()
    }

    pub fn get_acceleration_body_frame(&self) -> mint::Vector3<f32> {
        todo!()
    }

    pub fn get_orientation(&self) -> mint::Quaternion<f32> {
        self.orientation.into()
    }

    pub fn get_angular_velocity(&self) -> mint::Vector3<f32> {
        self.angular_velocity.into()
    }

    pub fn get_angular_acceleration(&self) -> mint::Vector3<f32> {
        self.angular_acceleration.into()
    }
}