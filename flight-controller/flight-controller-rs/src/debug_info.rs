use strum::{EnumCount, IntoEnumIterator};

use shared::fcu_hal::{FcuDebugInfo, PwmChannel, FcuDebugInfoVariant};

use crate::Fcu;

impl<'a> Fcu<'a> {
    pub fn generate_debug_info(&self, variant: FcuDebugInfoVariant) -> FcuDebugInfo {
        let timestamp = (self.driver.timestamp() * 1e3) as u64;

        match variant {
            FcuDebugInfoVariant::VehicleState => FcuDebugInfo::VehicleState {
                timestamp,
                vehicle_state: self.vehicle_state,
                position: self.state_vector.get_position().into(),
                velocity: self.state_vector.get_velocity().into(),
                acceleration: self.state_vector.get_acceleration().into(),
                orientation: self.state_vector.get_orientation().into(),
                angular_velocity: self.state_vector.get_angular_velocity().into(),
                angular_acceleration: self.state_vector.get_angular_acceleration(),
                position_error: self.state_vector.get_position_std_dev().into(),
                velocity_error: self.state_vector.get_velocity_std_dev().into(),
                acceleration_error: self.state_vector.get_acceleration_std_dev().into(),
                output_channels_bitmask: 0,
                output_channels_continuity_bitmask: self.get_output_channels_continuity_bitmask() as u32,
                pwm_channels: [0.0; PwmChannel::COUNT],
            },
            FcuDebugInfoVariant::SensorData => FcuDebugInfo::SensorData {
                timestamp,
                battery_voltage: 11.1169875,
                raw_accelerometer: self.state_vector.sensor_data.accelerometer_raw.into(),
                raw_gyroscope: self.state_vector.sensor_data.gyroscope_raw.into(),
                raw_magnetometer: self.state_vector.sensor_data.magnetometer_raw.into(),
                raw_barometer: self.state_vector.sensor_data.barometer_raw,
                accelerometer_calibration: self.state_vector.sensor_calibration.accelerometer.into(),
                barometric_altitude: self.state_vector.sensor_data.barometer_altitude,
                barometer_calibration: self.state_vector.sensor_calibration.barometeric_altitude,
            },
            FcuDebugInfoVariant::Stats => FcuDebugInfo::Stats {
                timestamp,
                apogee: self.apogee,
                data_logged_bytes: self.data_logger.get_bytes_logged(),
                cpu_utilization: self.driver.hardware_data().cpu_utilization as u32,
            },
        }
    }

    pub fn generate_debug_info_all_variants(&self, mut callback: impl FnMut(FcuDebugInfo)) {
        for variant in FcuDebugInfoVariant::iter() {
            callback(self.generate_debug_info(variant));
        }
    }
}
