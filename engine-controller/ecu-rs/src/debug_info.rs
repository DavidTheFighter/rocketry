use shared::ecu_hal::{EcuDebugInfo, EcuDebugInfoVariant};
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
            },
            EcuDebugInfoVariant::SensorData => EcuDebugInfo::SensorData {
                timestamp,
                fuel_tank_pressure_pa: self.fuel_tank_pressure_pa,
                oxidizer_tank_pressure_pa: self.oxidizer_tank_pressure_pa,
                igniter_chamber_pressure_pa: self.igniter_chamber_pressure_pa,
            },
        }
    }

    pub fn generate_debug_info_all_variants(&self, mut callback: impl FnMut(EcuDebugInfo)) {
        for variant in EcuDebugInfoVariant::iter() {
            callback(self.generate_debug_info(variant));
        }
    }
}
