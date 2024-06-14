use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal::{EcuAlert, EcuCommand}, ControllerState
};

use crate::{silprintln, Ecu};

use super::{engine_shutdown::EngineShutdown, EngineFsm};

pub struct Firing {
    startup_elapsed_time: f32,
}

impl<'f> ControllerState<EngineFsm, Ecu<'f>> for Firing {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<EngineFsm> {
        if self.chamber_pressure_degraded(ecu) {
            ecu.alert_manager.set_condition(EcuAlert::EngineChamberPressureOffNominal);

            return Some(EngineShutdown::new());
        }

        if self.engine_firing_timer_expired(ecu) {
            ecu.alert_manager.set_condition(EcuAlert::EngineShutdownTimerExpired);

            return Some(EngineShutdown::new());
        }

        if self.received_shutdown_command(packets) {
            return Some(EngineShutdown::new());
        }

        self.startup_elapsed_time += dt;

        None
    }

    fn enter_state(&mut self, _ecu: &mut Ecu) {
        silprintln!("Entered engine firing state");
        // Nothing
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Firing {
    pub fn new() -> EngineFsm {
        EngineFsm::Firing(Self {
            startup_elapsed_time: 0.0,
        })
    }

    fn chamber_pressure_degraded(&self, ecu: &Ecu) -> bool {
        (ecu.state_vector.sensor_data.engine_chamber_pressure_pa - ecu.config.engine_config.engine_target_combustion_pressure_pa).abs() > ecu.config.engine_config.engine_combustion_pressure_tolerance_pa
    }

    fn received_shutdown_command(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::ShutdownEngine = command {
                    return true;
                }
            }
        }

        false
    }

    fn engine_firing_timer_expired(&self, ecu: &Ecu) -> bool {
        self.startup_elapsed_time >= ecu.config.engine_config.engine_firing_duration_s.unwrap_or(f32::INFINITY)
    }
}
