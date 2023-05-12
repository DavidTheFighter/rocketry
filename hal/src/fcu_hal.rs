use mint::{Quaternion, Vector3};
use serde::{Serialize, Deserialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::comms_hal::{Packet, NetworkAddress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum VehicleState {
    Idle = 0,
    Ascent = 1,
    Descent = 2,
    Landed = 3,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcuTelemetryFrame {
    pub timestamp: u64,
    pub vehicle_state: VehicleState,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub angular_acceleration: Vector3<f32>,
    pub output_channels: [bool; OutputChannel::COUNT],
    pub pwm_channels: [f32; PwmChannel::COUNT],
    pub battery_voltage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcuConfig {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightConfig {

}

pub trait FcuDriver {
    fn set_output_channel(&mut self, channel: OutputChannel, state: bool);
    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32);

    fn get_output_channel(&self, channel: OutputChannel) -> bool;
    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool;
    fn get_pwm_channel(&self, channel: PwmChannel) -> f32;

    fn send_packet(&mut self, packet: Packet, destination: NetworkAddress);
}

impl FcuTelemetryFrame {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            vehicle_state: VehicleState::Idle,
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            orientation: Quaternion { s: 0.0, v: Vector3 { x: 0.0, y: 0.0, z: 0.0 } },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            angular_acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            battery_voltage: 0.0,
        }
    }
}