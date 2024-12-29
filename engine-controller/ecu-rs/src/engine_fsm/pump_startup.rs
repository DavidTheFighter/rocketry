use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuCommand, EngineConfig, PumpState, PumpType},
    ControllerState,
};

use crate::{engine_fsm::idle::Idle, silprintln, Ecu};

use super::{igniter_startup::IgniterStartup, EngineFsm};

pub struct PumpStartup {
    engine_config: EngineConfig,
    startup_elapsed_time: f32,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for PumpStartup {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        _packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.achieved_startup_pump_pressure(ecu) {
            return Some(IgniterStartup::new(self.engine_config.clone()));
        }

        if self.startup_timed_out() {
            ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::FuelMain, 0.0)));
            ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::OxidizerMain, 0.0)));

            ecu.alert_manager
                .set_condition(EcuAlert::EngineStartupPumpTimeout);

            return Some(Idle::new(self.engine_config.clone()));
        }

        self.startup_elapsed_time += dt;

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        silprintln!("Entered engine pump startup state");
        ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::FuelMain, 1.0)));
        ecu.enqueue_command(EcuCommand::SetPumpDuty((PumpType::OxidizerMain, 1.0)));
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl PumpStartup {
    pub fn new(engine_config: EngineConfig) -> EngineFsm {
        EngineFsm::PumpStartup(Self {
            engine_config,
            startup_elapsed_time: 0.0,
        })
    }

    fn startup_timed_out(&self) -> bool {
        self.startup_elapsed_time >= self.engine_config.pump_startup_timeout_s
    }

    fn achieved_startup_pump_pressure(&self, ecu: &Ecu) -> bool {
        ecu.fuel_pump
            .as_ref()
            .map(|pump| pump.hal_state() == PumpState::Pumping)
            .unwrap_or(false)
            && ecu
                .oxidizer_pump
                .as_ref()
                .map(|pump| pump.hal_state() == PumpState::Pumping)
                .unwrap_or(false)
            && (ecu.state_vector.sensor_data.fuel_pump_outlet_pressure_pa
                - self.engine_config.fuel_injector_pressure_setpoint_pa)
                .abs()
                < self
                    .engine_config
                    .fuel_injector_startup_pressure_tolerance_pa
            && (ecu
                .state_vector
                .sensor_data
                .oxidizer_pump_outlet_pressure_pa
                - self.engine_config.oxidizer_injector_pressure_setpoint_pa)
                .abs()
                < self
                    .engine_config
                    .oxidizer_injector_startup_pressure_tolerance_pa
    }
}
