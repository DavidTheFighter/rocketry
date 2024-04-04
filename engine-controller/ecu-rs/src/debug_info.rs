use shared::ecu_hal::{EcuDebugInfo, EcuDebugInfoVariant, PumpState, TankState};
use strum::IntoEnumIterator;

use crate::Ecu;

impl<'a> Ecu<'a> {
    pub fn generate_debug_info(&self, variant: EcuDebugInfoVariant) -> EcuDebugInfo {
        let timestamp = (self.driver.timestamp() * 1e3) as u64;

        match variant {
            EcuDebugInfoVariant::IgniterInfo => EcuDebugInfo::IgniterInfo {
                timestamp,
                igniter_state: self.igniter_state(),
                sparking: self.driver.get_sparking(),
                igniter_chamber_pressure_pa: self.state_vector.sensor_data.igniter_chamber_pressure_pa,
                igniter_fuel_injector_pressure_pa: self.state_vector.sensor_data.igniter_fuel_injector_pressure_pa,
                igniter_oxidizer_injector_pressure_pa: self.state_vector.sensor_data.igniter_oxidizer_injector_pressure_pa,
            },
            EcuDebugInfoVariant::TankInfo => EcuDebugInfo::TankInfo {
                timestamp,
                fuel_tank_state: self.fuel_tank_state().unwrap_or(TankState::Idle),
                oxidizer_tank_state: self.oxidizer_tank_state().unwrap_or(TankState::Idle),
            },
            EcuDebugInfoVariant::PumpInfo => EcuDebugInfo::PumpInfo {
                timestamp,
                fuel_pump_state: self.fuel_pump.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(PumpState::Idle),
                oxidizer_pump_state: self.oxidizer_pump.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(PumpState::Idle),
            },
            EcuDebugInfoVariant::SensorData => EcuDebugInfo::SensorData {
                timestamp,
                fuel_tank_pressure_pa: self.state_vector.sensor_data.fuel_tank_pressure_pa,
                oxidizer_tank_pressure_pa: self.state_vector.sensor_data.oxidizer_tank_pressure_pa,
            },
        }
    }

    pub fn generate_debug_info_all_variants(&self, mut callback: impl FnMut(EcuDebugInfo)) {
        for variant in EcuDebugInfoVariant::iter() {
            callback(self.generate_debug_info(variant));
        }
    }
}
