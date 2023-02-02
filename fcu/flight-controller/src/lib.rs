#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

use hal::fcu_hal::{FcuDriver, VehicleState};

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
}
