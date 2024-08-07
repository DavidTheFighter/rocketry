use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::{EcuAlert, EcuCommand}, ControllerState
};

use crate::{fsm_tanks_pressurized, silprintln, Ecu};

use super::{igniter_startup::IgniterStartup, pump_startup::PumpStartup, EngineFsm};

pub struct Idle;

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.received_fire_command(packets) {
            if fsm_tanks_pressurized(ecu) {
                ecu.alert_manager.clear_condition(EcuAlert::EngineTankOffNominal);

                if ecu.config.engine_config.use_pumps {
                    return Some(PumpStartup::new());
                } else {
                    return Some(IgniterStartup::new());
                }
            } else {
                ecu.alert_manager.set_condition(EcuAlert::EngineTankOffNominal);
            }
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

    fn received_fire_command(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::FireEngine = command {
                    return true;
                }
            }
        }

        false
    }
}
