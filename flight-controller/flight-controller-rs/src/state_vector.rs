use serde::Serialize;
use shared::{fcu_hal::{FcuConfig, FcuSensorData}, GRAVITY};
use nalgebra::{UnitQuaternion, Vector3};

use shared::standard_atmosphere::convert_pressure_to_altitude;

use self::{orientation::OrientationFilter, position::PositionFilter};

pub mod orientation;
pub mod position;

#[derive(Debug, Clone, Serialize)]
pub struct SensorCalibrationData {
    pub accelerometer: Vector3<f32>,
    pub gyroscope: Vector3<f32>,
    pub magnetometer: Vector3<f32>,
    pub barometer_pressure: f32,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct StateVector {
    pub(crate) position_filter: PositionFilter,
    pub(crate) orientation_filter: OrientationFilter,
    pub(crate) sensor_calibration: SensorCalibrationData,
    pub(crate) sensor_data: SensorData,
    pub landed: bool,
}

impl StateVector {
    pub fn new(config: &FcuConfig) -> Self {
        Self {
            position_filter: PositionFilter::new(config),
            orientation_filter: OrientationFilter::new(),
            sensor_calibration: SensorCalibrationData {
                accelerometer: Vector3::new(0.0, 0.0, 0.0),
                gyroscope: Vector3::new(0.0, 0.0, 0.0),
                magnetometer: Vector3::new(0.0, 0.0, 0.0),
                barometer_pressure: 0.0,
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
            },
            landed: true,
        }
    }

    pub fn predict(&mut self, dt: f32) {
        self.position_filter.predict(dt);
        self.orientation_filter.predict(dt);
    }

    pub fn update_config(&mut self, _config: &FcuConfig) {
        // TODO
    }

    pub fn update_calibration(&mut self, sensor_calibration: SensorCalibrationData) {
        self.sensor_calibration = sensor_calibration;
    }

    pub fn update_sensor_data(&mut self, data: FcuSensorData) {
        match data {
            FcuSensorData::Accelerometer {
                acceleration,
                raw_data,
            } => {
                self.sensor_data.accelerometer = acceleration.into();
                self.sensor_data.accelerometer_raw = raw_data.into();

                let mut acceleration = acceleration.into();
                acceleration += self.sensor_calibration.accelerometer;
                acceleration = self
                    .orientation_filter
                    .orientation
                    .transform_vector(&acceleration);

                if self.landed {
                    acceleration.y += GRAVITY;
                }

                self.position_filter.update_acceleration(acceleration);
            }
            FcuSensorData::Gyroscope {
                angular_velocity,
                raw_data,
            } => {
                self.sensor_data.gyroscope = angular_velocity.into();
                self.sensor_data.gyroscope_raw = raw_data.into();

                let mut angular_velocity = angular_velocity.into();
                angular_velocity += self.sensor_calibration.gyroscope;

                self.orientation_filter.update_gyroscope(angular_velocity);
            }
            FcuSensorData::Magnetometer {
                magnetic_field,
                raw_data,
            } => {
                self.sensor_data.magnetometer = magnetic_field.into();
                self.sensor_data.magnetometer_raw = raw_data.into();

                let mut magnetic_field = magnetic_field.into();
                magnetic_field += self.sensor_calibration.magnetometer;

                self.orientation_filter
                    .update_magnetic_field(magnetic_field);
            }
            FcuSensorData::Barometer {
                pressure,
                temperature,
                raw_data,
            } => {
                self.sensor_data.barometer_pressure = pressure;
                self.sensor_data.barometer_altitude =
                    convert_pressure_to_altitude(pressure, temperature);
                self.sensor_data.barometer_raw = raw_data;

                // let pressure = pressure + self.sensor_calibration.barometer_pressure;
                // let altitude = convert_pressure_to_altitude(pressure, temperature);

                // self.position_filter.update_barometric_pressure(altitude);
            }
        }
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.position_filter.position
    }

    pub fn get_position_std_dev(&self) -> Vector3<f32> {
        self.position_filter.position_std_dev
    }

    pub fn get_velocity(&self) -> Vector3<f32> {
        self.position_filter.velocity
    }

    pub fn get_velocity_std_dev(&self) -> Vector3<f32> {
        self.position_filter.velocity_std_dev
    }

    pub fn get_acceleration(&self) -> Vector3<f32> {
        self.position_filter.acceleration
    }

    pub fn get_acceleration_std_dev(&self) -> Vector3<f32> {
        self.position_filter.acceleration_std_dev
    }

    pub fn get_orientation(&self) -> UnitQuaternion<f32> {
        self.orientation_filter.orientation
    }

    pub fn get_angular_velocity(&self) -> Vector3<f32> {
        self.orientation_filter.angular_velocity
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
