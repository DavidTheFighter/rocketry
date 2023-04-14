#![cfg_attr(all(not(test), arcref), no_std)]
#![deny(unsafe_code)]

pub mod kalman;

use hal::fcu_hal::{FcuDriver, VehicleState};
use mint::Vector3;

pub struct Fcu<'a> {
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
}

impl<'a> Fcu<'a> {
    pub fn new(driver: &'a mut dyn FcuDriver) -> Self {
        Self {
            vehicle_state: VehicleState::Idle,
            driver,
        }
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>) {
        // something
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: Vector3<f32>) {
        // something
    }

    pub fn update_magnetic_field(&mut self, magnetic_field: Vector3<f32>) {
        // something
    }

    pub fn update_barometric_pressure(&mut self, barometric_pressure: f32) {
        // something
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        // something
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Fcu<'_> {}