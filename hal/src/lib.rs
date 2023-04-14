#![forbid(unsafe_code)]
#![no_std]

pub mod comms_hal;
pub mod ecu_hal;
pub mod fcu_hal;

pub mod ecu_mock;
pub mod fcu_mock;

use serde::{Deserialize, Serialize};

// Describes a polynomial calibration curve for a sensor.
// Given in the form: y = x0 + x1 * x + x2 * x^2 + x3 * x^3
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub x0: f32,
    pub x1: f32,
    pub x2: f32,
    pub x3: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
    pub calibration: Option<SensorCalibration>,
}

impl SensorConfig {
    pub const fn default() -> Self {
        Self {
            premin: 0.0,
            premax: 1.0,
            postmin: 0.0,
            postmax: 1.0,
            calibration: None,
        }
    }

    pub fn apply(&self, val: f32) -> f32 {
        let lerp = (val - self.premin) / (self.premax - self.premin);
        let mut value = lerp * (self.postmax - self.postmin) + self.postmin;

        if let Some(curve) = self.calibration {
            let x = value;
            value += curve.x0 + curve.x1 * x + curve.x2 * x * x + curve.x3 * x * x * x;
        }

        value
    }
}
