use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::{EcuCommand, EcuLinearOutput, PumpType}, ControllerState
};

use super::{pumping::Pumping, PumpFsm};

#[derive(Debug)]
pub struct Idle {
    pump_type: PumpType,
    linear_output: EcuLinearOutput,
}

impl<'f> ControllerState<PumpFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<PumpFsm> {
        if let Some(duty) = self.received_pump_command(packets) {
            if duty > 0.01 {
                return Some(Pumping::new(self.pump_type, self.linear_output, duty));
            }
        }

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
    pub fn new(pump_type: PumpType, linear_output: EcuLinearOutput) -> PumpFsm {
        PumpFsm::Idle(Self {
            pump_type,
            linear_output,
        })
    }

    fn received_pump_command(&self, packets: &[(NetworkAddress, Packet)]) -> Option<f32> {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::SetPumpDuty((pump, duty)) = command {
                    if *pump == self.pump_type {
                        return Some(*duty);
                    }
                }
            }
        }

        None
    }
}
