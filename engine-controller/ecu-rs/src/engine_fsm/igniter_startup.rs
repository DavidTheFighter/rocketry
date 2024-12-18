use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::{EcuAlert, EcuCommand, IgniterState}, ControllerState
};

use crate::{silprintln, Ecu};

use super::{engine_shutdown::EngineShutdown, engine_startup::EngineStartup, EngineFsm};

pub struct IgniterStartup {
    startup_elapsed_time: f32,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for IgniterStartup {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {

        match ecu.igniter_state() {
            IgniterState::Firing => {
                return Some(EngineStartup::new());
            },
            IgniterState::Shutdown => {
                silprintln!("Aborting due to {:?} state", ecu.igniter_state());
                ecu.alert_manager.set_condition(EcuAlert::EngineStartupIgniterAnomaly);
                return Some(EngineShutdown::new());
            }
            _ => {},
        }

        if self.startup_timed_out(ecu) {
            ecu.alert_manager.set_condition(EcuAlert::EngineStartupIgniterAnomaly);
            return Some(EngineShutdown::new());
        }

        // TODO Check stable pump/feed pressure

        self.startup_elapsed_time += dt;

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        silprintln!("Entered engine igniter startup state");
        ecu.enqueue_command(EcuCommand::FireIgniter);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl IgniterStartup {
    pub fn new() -> EngineFsm {
        EngineFsm::IgniterStartup(Self {
            startup_elapsed_time: 0.0,
        })
    }

    fn startup_timed_out(&self, ecu: &mut Ecu) -> bool {
        self.startup_elapsed_time >= ecu.config.engine_config.igniter_startup_timeout_s
    }
}
