use hal::fcu_hal::FcuConfig;
use nalgebra::Vector3;
use mint;

use self::{kalman::KalmanFilter, orientation::OrientationFilter};

mod kalman;
pub mod orientation;
pub mod position;

pub struct SensorCalibrationData {
    pub accelerometer: Vector3<f32>,
    pub gyroscope: Vector3<f32>,
    pub magnetometer: Vector3<f32>,
    pub barometric_altitude: f32,
}

pub struct StateVector {
    pub(crate) position: Vector3<f32>,
    pub(crate) position_std_dev: Vector3<f32>,
    pub(crate) velocity: Vector3<f32>,
    pub(crate) velocity_std_dev: Vector3<f32>,
    pub(crate) acceleration: Vector3<f32>,
    pub(crate) acceleration_std_dev: Vector3<f32>,
    pub(crate) orientation: OrientationFilter,
    pub(crate) sensor_calibration: SensorCalibrationData,
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
            orientation: OrientationFilter::new(),
            position_kalman: Self::init_position_kalman(config),
            sensor_calibration: SensorCalibrationData {
                accelerometer: Vector3::new(0.0, 0.0, 0.0),
                gyroscope: Vector3::new(0.0, 0.0, 0.0),
                magnetometer: Vector3::new(0.0, 0.0, 0.0),
                barometric_altitude: 0.0,
            },
        }
    }

    pub fn predict(&mut self, dt: f32) {
        self.predict_position(dt);
        self.orientation.predict(dt);
    }

    pub fn update_config(&mut self, config: &FcuConfig) {
        self.update_config_position(config);
        // self.update_config_orientation(config);
    }

    pub fn update_calibration(&mut self, sensor_calibration: SensorCalibrationData) {
        self.sensor_calibration = sensor_calibration;
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

    pub fn get_acceleration_body_frame(&self) -> mint::Vector3<f32> {
        todo!()
    }

    pub fn get_orientation(&self) -> mint::Quaternion<f32> {
        self.orientation.orientation.into()
    }

    pub fn get_angular_velocity(&self) -> mint::Vector3<f32> {
        self.orientation.angular_velocity.into()
    }

    pub fn get_angular_acceleration(&self) -> mint::Vector3<f32> {
        mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}