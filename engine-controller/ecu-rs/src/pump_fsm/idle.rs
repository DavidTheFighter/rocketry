use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::{EcuCommand, EcuLinearOutput}, ControllerState
};

use super::PumpFsm;

#[derive(Debug)]
pub struct Idle {
    linear_output: EcuLinearOutput,
}

impl<'f> ControllerState<PumpFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<PumpFsm> {
        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver.set_linear_output(self.linear_output, 0.0);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Idle {
    pub fn new(linear_output: EcuLinearOutput) -> PumpFsm {
        PumpFsm::Idle(Self {
            linear_output,
        })
    }

    fn received_pump_command(&self, packets: &[(NetworkAddress, Packet)]) -> Option<f32> {
        // for (source, packet) in packets {
        //     if let Packet::EcuCommand(command) = packet {
        //         if let EcuCommand::Set
        //     }
        // }
        None
    }
}
