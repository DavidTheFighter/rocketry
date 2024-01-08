#![cfg_attr(all(not(test), feature = "no_std"), no_std)]
#![forbid(unsafe_code)]

pub mod big_brother;
mod dedupe;
pub(crate) mod forwarding;
pub mod interface;
mod network_map;
pub mod serdes;

pub use crate::big_brother::BigBrother;
