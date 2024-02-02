#![cfg_attr(not(any(test, feature = "sil")), no_std)]
#![deny(unsafe_code)]

pub mod debug_info;
pub mod ecu;
pub mod engine_fsm;
pub mod igniter_fsm;
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

pub(crate) use silprintln;
