#![forbid(unsafe_code)]
#![no_std]

pub mod ecu_hal;
pub mod comms_hal;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
}
