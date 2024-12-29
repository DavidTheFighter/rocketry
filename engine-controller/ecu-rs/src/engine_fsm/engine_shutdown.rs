use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuBinaryOutput, EcuCommand, EngineConfig, PumpType},
    ControllerState,
};

use crate::{silprintln, Ecu};

use super::{idle::Idle, EngineFsm};

pub struct EngineShutdown {
    engine_config: EngineConfig,
    time_since_state_transition: f32,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for EngineShutdown {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.time_since_state_transition >= self.engine_config.engine_shutdown_duration_s {
            return Some(Idle::new(self.engine_config.clone()));
        }

        self.time_since_state_transition += dt;

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        silprintln!("Entered engine shutdown state");
        ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::FuelMain, 0.0)));
        ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::OxidizerMain, 0.0)));
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::EngineFuelValve, false);
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::EngineOxidizerValve, false);
        // Nothing
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl EngineShutdown {
    pub fn new(engine_config: EngineConfig) -> EngineFsm {
        EngineFsm::EngineShutdown(Self {
            engine_config,
            time_since_state_transition: 0.0,
        })
    }
}
