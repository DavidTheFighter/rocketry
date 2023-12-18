#![cfg_attr(all(not(test), not(feature = "stdtcp")), no_std)]
#![forbid(unsafe_code)]

pub mod big_brother;
pub mod interface;
mod network_map;
pub mod serdes;

pub use big_brother::BigBrother;
