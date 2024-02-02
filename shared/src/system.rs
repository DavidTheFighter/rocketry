use serde::{Deserialize, Serialize};

use crate::{comms_hal::{NetworkAddress, Packet}, ecu_hal, fcu_hal};

pub const MAX_ENGINE_COUNT: usize = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedSystemState {
    pub fcu: Option<fcu_hal::FcuTelemetryFrame>,
    pub engine_states: [Option<ecu_hal::EcuTelemetryFrame>; MAX_ENGINE_COUNT],
}

impl SharedSystemState {
    pub fn new() -> Self {
        Self {
            fcu: None,
            engine_states: [None; MAX_ENGINE_COUNT],
        }
    }

    pub fn update(&mut self, packet: &Packet, source: NetworkAddress) {
        match packet {
            Packet::FcuTelemetry(frame) => self.fcu = Some(frame.clone()),
            Packet::EcuTelemetry(frame) => {
                if let NetworkAddress::EngineController(index) = source {
                    self.engine_states[index as usize] = Some(frame.clone());
                }
            }
            _ => {}
        }
    }
}
