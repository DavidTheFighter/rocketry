use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuBinaryOutput, EcuCommand, EngineConfig, PumpType},
    ControllerState,
};

use crate::{silprintln, Ecu};

use super::{firing::Firing, idle::Idle, EngineFsm};

pub struct EngineStartup {
    engine_config: EngineConfig,
    startup_elapsed_time: f32,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for EngineStartup {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.achieved_stable_pressure(ecu) {
            return Some(Firing::new(self.engine_config.clone()));
        }

        if self.startup_timed_out() {
            ecu.driver
                .set_binary_valve(EcuBinaryOutput::EngineFuelValve, false);
            ecu.driver
                .set_binary_valve(EcuBinaryOutput::EngineOxidizerValve, false);
            ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::FuelMain, 0.0)));
            ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::OxidizerMain, 0.0)));

            ecu.alert_manager
                .set_condition(EcuAlert::EngineStartupTimeout);

            return Some(Idle::new(self.engine_config.clone()));
        }

        self.startup_elapsed_time += dt;

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        silprintln!("Entered engine startup state");
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::EngineFuelValve, true);
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::EngineOxidizerValve, true);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl EngineStartup {
    pub fn new(engine_config: EngineConfig) -> EngineFsm {
        EngineFsm::EngineStartup(Self {
            engine_config,
            startup_elapsed_time: 0.0,
        })
    }

    fn achieved_stable_pressure(&self, ecu: &Ecu) -> bool {
        (ecu.state_vector.sensor_data.engine_chamber_pressure_pa
            - self.engine_config.engine_target_combustion_pressure_pa)
            .abs()
            < self.engine_config.engine_combustion_pressure_tolerance_pa
    }

    fn startup_timed_out(&self) -> bool {
        self.startup_elapsed_time >= self.engine_config.engine_startup_timeout_s
    }
}
