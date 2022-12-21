#![forbid(unsafe_code)]
#![no_std]

pub mod comms_hal;
pub mod ecu_hal;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
}

impl SensorConfig {
    pub const fn default() -> Self {
        Self {
            premin: 0.0,
            premax: 1.0,
            postmin: 0.0,
            postmax: 1.0,
        }
    }

    pub fn apply(&self, val: f32) -> f32 {
        let lerp = (val - self.premin) / (self.premax - self.premin);

        lerp * (self.postmax - self.postmin) + self.postmin
    }
}
