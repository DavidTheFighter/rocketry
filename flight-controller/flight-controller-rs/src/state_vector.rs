use serde::Serialize;
use shared::{fcu_hal::{FcuConfig, FcuSensorData}, GRAVITY};
use nalgebra::{UnitQuaternion, Vector3};

use shared::standard_atmosphere::convert_pressure_to_altitude;

use self::kalman::KalmanFilter;

pub mod kalman;

#[derive(Debug, Clone, Serialize)]
pub struct SensorCalibrationData {
    pub accelerometer: Vector3<f32>,
    pub gyroscope: Vector3<f32>,
    pub magnetometer: Vector3<f32>,
    pub barometeric_altitude: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensorData {
    pub accelerometer: Vector3<f32>,
    pub accelerometer_raw: Vector3<i16>,
    pub gyroscope: Vector3<f32>,
    pub gyroscope_raw: Vector3<i16>,
    pub magnetometer: Vector3<f32>,
    pub magnetometer_raw: Vector3<i16>,
    pub barometer_pressure: f32,
    pub barometer_altitude: f32,
    pub barometer_raw: u32,
    pub barometer_temperature: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateVector {
    pub(crate) kalman: KalmanFilter,
    pub(crate) sensor_calibration: SensorCalibrationData,
    pub(crate) sensor_data: SensorData,
    pub landed: bool,
}

impl StateVector {
    pub fn new(config: &FcuConfig) -> Self {
        Self {
            kalman: KalmanFilter::new(config),
            sensor_calibration: SensorCalibrationData {
                accelerometer: Vector3::new(0.0, 0.0, 0.0),
                gyroscope: Vector3::new(0.0, 0.0, 0.0),
                magnetometer: Vector3::new(0.0, 0.0, 0.0),
                barometeric_altitude: 0.0,
            },
            sensor_data: SensorData {
                accelerometer: Vector3::new(0.0, 0.0, 0.0),
                accelerometer_raw: Vector3::new(0, 0, 0),
                gyroscope: Vector3::new(0.0, 0.0, 0.0),
                gyroscope_raw: Vector3::new(0, 0, 0),
                magnetometer: Vector3::new(0.0, 0.0, 0.0),
                magnetometer_raw: Vector3::new(0, 0, 0),
                barometer_pressure: 0.0,
                barometer_altitude: 0.0,
                barometer_raw: 0,
                barometer_temperature: 0.0,
            },
            landed: true,
        }
    }

    pub fn predict(&mut self, dt: f32) {
        self.kalman.predict(dt);
    }

    pub fn update_config(&mut self, config: &FcuConfig) {
        self.kalman.update_config(config);
    }

    pub fn update_calibration(&mut self, sensor_calibration: SensorCalibrationData) {
        self.sensor_calibration = sensor_calibration;
    }

    pub fn update_sensor_data(&mut self, data: &FcuSensorData) {
        match *data {
            FcuSensorData::Accelerometer {
                acceleration,
                raw_data,
            } => {
                self.sensor_data.accelerometer = acceleration.into();
                self.sensor_data.accelerometer_raw = raw_data.into();

                let mut acceleration = acceleration.into();
                acceleration += self.sensor_calibration.accelerometer;
                acceleration = self
                    .kalman
                    .orientation
                    .transform_vector(&acceleration);

                if self.landed {
                    acceleration.y += GRAVITY;
                }

                self.kalman.update_acceleration(acceleration);
            }
            FcuSensorData::Gyroscope {
                angular_velocity,
                raw_data,
            } => {
                self.sensor_data.gyroscope = angular_velocity.into();
                self.sensor_data.gyroscope_raw = raw_data.into();

                let mut angular_velocity = angular_velocity.into();
                angular_velocity += self.sensor_calibration.gyroscope;

                self.kalman.update_gyroscope(angular_velocity);
            }
            FcuSensorData::Magnetometer {
                magnetic_field,
                raw_data,
            } => {
                // self.sensor_data.magnetometer = magnetic_field.into();
                // self.sensor_data.magnetometer_raw = raw_data.into();

                // let mut magnetic_field = magnetic_field.into();
                // magnetic_field += self.sensor_calibration.magnetometer;

                // self.kalman
                //     .update_magnetic_field(magnetic_field);
            }
            FcuSensorData::Barometer {
                pressure,
                temperature,
                raw_data,
            } => {
                self.sensor_data.barometer_pressure = pressure;
                self.sensor_data.barometer_temperature = temperature;
                self.sensor_data.barometer_raw = raw_data;

                self.sensor_data.barometer_altitude = convert_pressure_to_altitude(pressure, temperature);

                let altitude = self.sensor_data.barometer_altitude + self.sensor_calibration.barometeric_altitude;

                self.kalman.update_barometric_pressure(altitude);
            }
        }
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.kalman.position
    }

    pub fn get_position_std_dev(&self) -> Vector3<f32> {
        self.kalman.position_std_dev
    }

    pub fn get_velocity(&self) -> Vector3<f32> {
        self.kalman.velocity
    }

    pub fn get_velocity_std_dev(&self) -> Vector3<f32> {
        self.kalman.velocity_std_dev
    }

    pub fn get_acceleration(&self) -> Vector3<f32> {
        self.kalman.acceleration
    }

    pub fn get_acceleration_std_dev(&self) -> Vector3<f32> {
        self.kalman.acceleration_std_dev
    }

    pub fn get_orientation(&self) -> UnitQuaternion<f32> {
        self.kalman.orientation
    }

    pub fn get_angular_velocity(&self) -> Vector3<f32> {
        self.kalman.angular_velocity
    }

    pub fn get_angular_acceleration(&self) -> mint::Vector3<f32> {
        mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn set_landed(&mut self, landed: bool) {
        self.landed = landed;
    }

    pub fn get_landed(&self) -> bool {
        self.landed
    }
}
