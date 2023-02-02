use serde::{Serialize, Deserialize};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::comms_hal::{Packet, NetworkAddress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum VehicleState {
    Idle = 0,
    Ascent = 1,
    Descent = 2,
    Landed = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum PyroChannel {
    PyroChannel0 = 0,
    PyroChannel1 = 1,
    PyroChannel2 = 2,
    PyroChannel3 = 3,
}

pub trait FcuDriver {
    fn set_pyro_channel(&mut self, channel: PyroChannel, state: bool);

    fn send_packet(&mut self, packet: Packet, destination: NetworkAddress);
}