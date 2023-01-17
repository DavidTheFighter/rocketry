#![no_std]
#![forbid(unsafe_code)]

pub mod igniter_fsm;

use hal::ecu_hal::{IgniterConfig, IgniterState, FuelTankState};

pub struct EcuState {
    pub(crate) igniter_config: IgniterConfig,
    pub(crate) igniter_state: IgniterState,
    pub(crate) fuel_tank_state: FuelTankState,
}

