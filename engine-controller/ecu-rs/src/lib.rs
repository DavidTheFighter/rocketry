#![cfg_attr(not(any(test, feature = "sil")), no_std)]
#![deny(unsafe_code)]

pub mod debug_info;
pub mod ecu;
pub mod engine_fsm;
pub mod igniter_fsm;
pub mod pump_fsm;
pub mod state_vector;
pub mod tank_fsm;

pub use ecu::Ecu;

#[cfg(any(test, feature = "sil"))]
macro_rules! silprintln {
    () => { println!() };
    ($($arg:tt)*) => { println!($($arg)*) };
}

#[cfg(not(any(test, feature = "sil")))]
macro_rules! silprintln {
    () => {};
    ($($arg:tt)*) => {};
}

use shared::ecu_hal::TankState;
pub(crate) use silprintln;

pub(crate) fn fsm_tanks_pressurized(ecu: &Ecu) -> bool {
    let fuel_tank_pressurized = ecu
        .fuel_tank_state()
        .map_or(true, |state| state == TankState::Pressurized);

    let oxidizer_tank_pressurized = ecu
        .oxidizer_tank_state()
        .map_or(true, |state| state == TankState::Pressurized);

    fuel_tank_pressurized && oxidizer_tank_pressurized
}
