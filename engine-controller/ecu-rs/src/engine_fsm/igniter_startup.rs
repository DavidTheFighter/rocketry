use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuCommand, EngineConfig, IgniterState},
    ControllerState,
};

use crate::{silprintln, Ecu};

use super::{engine_shutdown::EngineShutdown, engine_startup::EngineStartup, EngineFsm};

pub struct IgniterStartup {
    engine_config: EngineConfig,
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
                return Some(EngineStartup::new(self.engine_config.clone()));
            }
            IgniterState::Shutdown => {
                silprintln!("Aborting due to {:?} state", ecu.igniter_state());
                ecu.alert_manager
                    .set_condition(EcuAlert::EngineStartupIgniterAnomaly);
                return Some(EngineShutdown::new(self.engine_config.clone()));
            }
            _ => {}
        }

        if self.startup_timed_out() {
            ecu.alert_manager
                .set_condition(EcuAlert::EngineStartupIgniterAnomaly);
            return Some(EngineShutdown::new(self.engine_config.clone()));
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
    pub fn new(engine_config: EngineConfig) -> EngineFsm {
        EngineFsm::IgniterStartup(Self {
            engine_config,
            startup_elapsed_time: 0.0,
        })
    }

    fn startup_timed_out(&self) -> bool {
        self.startup_elapsed_time >= self.engine_config.igniter_startup_timeout_s
    }
}
