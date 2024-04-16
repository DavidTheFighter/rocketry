use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::EcuCommand, ControllerState
};

use crate::{fsm_tanks_pressurized, silprintln, Ecu};

use super::{pump_startup::PumpStartup, EngineFsm};

pub struct Idle;

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.received_fire_pump_fed(packets) && fsm_tanks_pressurized(ecu) {
            return Some(PumpStartup::new());
        }

        None
    }

    fn enter_state(&mut self, _ecu: &mut Ecu) {
        silprintln!("Entered engine idle state");
        // Nothing
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Idle {
    pub fn new() -> EngineFsm {
        EngineFsm::Idle(Self {})
    }

    fn received_fire_pump_fed(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::FireEnginePumpFed = command {
                    return true;
                }
            }
        }

        false
    }
}
