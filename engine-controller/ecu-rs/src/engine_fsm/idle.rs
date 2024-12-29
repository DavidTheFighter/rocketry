use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuCommand, EngineConfig},
    ControllerState,
};

use crate::{fsm_tanks_pressurized, silprintln, Ecu};

use super::{igniter_startup::IgniterStartup, pump_startup::PumpStartup, EngineFsm};

pub struct Idle {
    engine_config: EngineConfig,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.received_fire_command(packets) {
            if fsm_tanks_pressurized(ecu) {
                ecu.alert_manager
                    .clear_condition(EcuAlert::EngineTankOffNominal);

                if self.engine_config.use_pumps {
                    return Some(PumpStartup::new(self.engine_config.clone()));
                } else {
                    return Some(IgniterStartup::new(self.engine_config.clone()));
                }
            } else {
                ecu.alert_manager
                    .set_condition(EcuAlert::EngineTankOffNominal);
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
    pub fn new(engine_config: EngineConfig) -> EngineFsm {
        EngineFsm::Idle(Self { engine_config })
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
