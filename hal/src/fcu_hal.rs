use core::any::Any;

use mint::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::{
    comms_hal::{NetworkAddress, Packet},
    fcu_log::DataPoint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum VehicleState {
    Idle,
    Calibrating,
    Armed,
    Launching,
    Ascent,
    Descent,
    // DescentDrogueParachute,
    // DescentMainParachute,
    Landed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum OutputChannel {
    OutputChannel0 = 0,
    OutputChannel1 = 1,
    OutputChannel2 = 2,
    OutputChannel3 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum PwmChannel {
    PwmChannel0 = 0,
    PwmChannel1 = 1,
    PwmChannel2 = 2,
    PwmChannel3 = 3,
    PwmChannel4 = 4,
    PwmChannel5 = 5,
}

pub enum FcuAlertEvent {

}

pub enum FcuAlertCondition {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FcuSensorData {
    Accelerometer {
        acceleration: Vector3<f32>,
        raw_data: Vector3<i16>,
    },
    Gyroscope {
        angular_velocity: Vector3<f32>,
        raw_data: Vector3<i16>,
    },
    Magnetometer {
        magnetic_field: Vector3<f32>,
        raw_data: Vector3<i16>,
    },
    Barometer {
        pressure: f32,
        temperature: f32,
        raw_data: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FcuTelemetryFrame {
    pub timestamp: u64,
    pub vehicle_state: VehicleState,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub position_error: f32,     // Standard deviation
    pub velocity_error: f32,     // Standard deviation
    pub acceleration_error: f32, // Standard deviation
    pub output_channels: [bool; OutputChannel::COUNT],
    pub pwm_channels: [f32; PwmChannel::COUNT],
    pub apogee: f32,
    pub battery_voltage: f32,
    pub data_logged_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FcuDebugInfo {
    pub timestamp: u64,
    pub vehicle_state: VehicleState,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub angular_acceleration: Vector3<f32>,
    pub position_error: Vector3<f32>,     // Standard deviation
    pub velocity_error: Vector3<f32>,     // Standard deviation
    pub acceleration_error: Vector3<f32>, // Standard deviation
    pub output_channels: [bool; OutputChannel::COUNT],
    pub pwm_channels: [f32; PwmChannel::COUNT],
    pub apogee: f32,
    pub battery_voltage: f32,
    pub data_logged_bytes: u32,
    pub raw_accelerometer: Vector3<i16>,
    pub raw_gyroscope: Vector3<i16>,
    pub raw_magnetometer: Vector3<i16>,
    pub raw_barometer: u32,
    pub raw_barometric_altitude: f32,
    pub accelerometer_calibration: Vector3<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FcuDevStatsFrame {
    pub timestamp: u64,
    pub cpu_utilization: f32,
    pub fcu_update_latency_avg: f32,
    pub fcu_update_latency_max: f32,
    pub packet_queue_length_avg: f32,
    pub packet_queue_length_max: u32,
    pub fcu_update_elapsed_avg: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FcuConfig {
    pub telemetry_rate: f32,
    pub startup_acceleration_threshold: f32,
    pub position_kalman_process_variance: f32,
    pub calibration_duration: f32,
    pub accelerometer_noise_std_dev: Vector3<f32>,
    pub barometer_noise_std_dev: f32,
    pub gps_noise_std_dev: Vector3<f32>,
    // Add a bitfield to contain all of the eventual bool configs
    // pub log_dev_stats: bool,
    //
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightConfig {}

pub trait FcuDriver {
    fn timestamp(&self) -> f32;

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool);
    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32);

    fn get_output_channel(&self, channel: OutputChannel) -> bool;
    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool;
    fn get_pwm_channel(&self, channel: PwmChannel) -> f32;

    fn log_data_point(&mut self, datapoint: DataPoint);
    // fn retrieve_log_data_point(&mut self) -> Option<DataPoint>;
    fn erase_flash_chip(&mut self);
    fn enable_logging_to_flash(&mut self);
    fn disable_logging_to_flash(&mut self);
    fn retrieve_log_flash_page(&mut self, addr: u32);

    fn send_packet(&mut self, packet: Packet, destination: NetworkAddress);

    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl FcuTelemetryFrame {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            vehicle_state: VehicleState::Idle,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            velocity: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            acceleration: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            orientation: Quaternion {
                s: 0.0,
                v: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            angular_velocity: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            position_error: 0.0,
            velocity_error: 0.0,
            acceleration_error: 0.0,
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: 0.0,
            battery_voltage: 0.0,
            data_logged_bytes: 0,
        }
    }
}

impl FcuDebugInfo {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            vehicle_state: VehicleState::Idle,
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            orientation: Quaternion { v: Vector3 { x: 0.0, y: 0.0, z: 0.0 }, s: 1.0 },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            angular_acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            position_error: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            velocity_error: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            acceleration_error: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: -1.0,
            battery_voltage: 0.0,
            data_logged_bytes: 0,
            raw_accelerometer: Vector3 { x: 0, y: 0, z: 0 },
            raw_gyroscope: Vector3 { x: 0, y: 0, z: 0 },
            raw_magnetometer: Vector3 { x: 0, y: 0, z: 0 },
            raw_barometer: 0,
            raw_barometric_altitude: 0.0,
            accelerometer_calibration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        }
    }
}

impl FcuConfig {
    pub const fn default() -> Self {
        Self {
            telemetry_rate: 0.02,
            startup_acceleration_threshold: 0.1,
            position_kalman_process_variance: 1e-3,
            calibration_duration: 5.0,
            accelerometer_noise_std_dev: Vector3 {
                x: 1e-2,
                y: 1e-2,
                z: 1e-2,
            },
            barometer_noise_std_dev: 1e-4,
            gps_noise_std_dev: Vector3 {
                x: 5.0,
                y: 10.0,
                z: 5.0,
            },
        }
    }
}

impl FcuDevStatsFrame {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            cpu_utilization: 0.0,
            fcu_update_latency_avg: 0.0,
            fcu_update_latency_max: 0.0,
            packet_queue_length_avg: 0.0,
            packet_queue_length_max: 0,
            fcu_update_elapsed_avg: 0.0,
        }
    }
}
