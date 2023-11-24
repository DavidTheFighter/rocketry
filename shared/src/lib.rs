#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub mod alerts;
pub mod comms_hal;
pub mod comms_manager;
pub mod ecu_hal;
pub mod ecu_mock;
pub mod fcu_hal;
pub mod fcu_mock;
pub mod logger;
pub mod standard_atmosphere;

use comms_hal::{NetworkAddress, Packet};
use serde::{Deserialize, Serialize};

pub const GRAVITY: f32 = 9.80665; // In m/s^2

pub trait ControllerState<S, C> {
    fn update(
        &mut self,
        controller: &mut C,
        dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<S>;
    fn enter_state(&mut self, controller: &mut C);
    fn exit_state(&mut self, controller: &mut C);
}

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
