use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuCommand, EcuLinearOutput, PumpType},
    ControllerState,
};

use super::{idle::Idle, PumpFsm};

#[derive(Debug)]
pub struct Pumping {
    pump_type: PumpType,
    linear_output: EcuLinearOutput,
    duty_cycle: f32,
}

impl<'f> ControllerState<PumpFsm, Ecu<'f>> for Pumping {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<PumpFsm> {
        if let Some(duty) = self.received_pump_command(packets) {
            if duty > 0.01 {
                self.duty_cycle = duty;
                ecu.driver
                    .set_linear_output(self.linear_output, self.duty_cycle);
            } else {
                return Some(Idle::new(self.pump_type, self.linear_output));
            }
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver
            .set_linear_output(self.linear_output, self.duty_cycle);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Pumping {
    pub fn new(pump_type: PumpType, linear_output: EcuLinearOutput, duty_cycle: f32) -> PumpFsm {
        PumpFsm::Pumping(Self {
            pump_type,
            linear_output,
            duty_cycle,
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
