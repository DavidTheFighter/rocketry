use core::any::Any;

use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount as EnumCountMacro, EnumDiscriminants, EnumIter};

use crate::{
    comms_hal::{NetworkAddress, Packet},
    SensorConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum EngineState {
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum IgniterState {
    Idle,
    Startup,
    Firing,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum TankState {
    Idle,
    Depressurized,
    Pressurized,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumIter, EnumCountMacro, EnumDiscriminants)]
#[strum_discriminants(name(EcuSensorDataVariant))]
#[strum_discriminants(derive(Serialize, Deserialize))]
pub enum EcuSensorData {
    FuelTankPressure {
        pressure_pa: f32,
        raw_data: u16,
    },
    OxidizerTankPressure {
        pressure_pa: f32,
        raw_data: u16,
    },
    IgniterChamberPressure {
        pressure_pa: f32,
        raw_data: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EcuCommand {
    SetBinaryValve {
        valve: EcuBinaryValve,
        state: bool,
    },
    SetSparking(bool),
    FireIgniter,
    SetFuelTank(TankState),
    SetOxidizerTank(TankState),
    ConfigureSensor {
        sensor: EcuSensorDataVariant,
        config: SensorConfig,
    },
    ConfigureEcu(EcuConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuBinaryValve {
    IgniterFuelMain,
    IgniterGOxMain,
    FuelPress,
    FuelVent,
    OxidizerPress,
    OxidizerVent,
}

impl EcuBinaryValve {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuTelemetryFrame {
    pub timestamp: u64,
    pub engine_state: EngineState,
    pub igniter_state: IgniterState,
    pub igniter_chamber_pressure_pa: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuTankTelemetryFrame {
    pub timestamp: u64,
    pub fuel_tank_state: TankState,
    pub oxidizer_tank_state: TankState,
    pub fuel_tank_pressure_pa: f32,
    pub oxidizer_tank_pressure_pa: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(EcuDebugInfoVariant))]
#[strum_discriminants(derive(EnumIter))]
pub enum EcuDebugInfo {
    IgniterInfo {
        timestamp: u64,
        igniter_state: IgniterState,
        sparking: bool,
    },
    SensorData {
        timestamp: u64,
        fuel_tank_pressure_pa: f32,
        oxidizer_tank_pressure_pa: f32,
        igniter_chamber_pressure_pa: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuConfig {
    pub igniter_config: IgniterConfig,
    pub telemetry_rate_s: f32,
}

impl EcuConfig {
    pub fn default() -> Self {
        Self {
            igniter_config: IgniterConfig::default(),
            telemetry_rate_s: 0.02,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgniterConfig {
    pub startup_timeout_s: f32,
    pub startup_pressure_threshold_pa: f32, // TODO: This is in PSI, it should be in Pascals
    pub startup_stable_time_s: f32,
    pub test_firing_duration_s: f32,
    pub shutdown_duration_s: f32,
    pub max_throat_temp_k: f32, // In Celsius
}

impl IgniterConfig {
    pub fn default() -> Self {
        Self {
            startup_timeout_s: 1.0,
            startup_pressure_threshold_pa: 30.0 * 6894.76, // 30 PSI to Pascals
            startup_stable_time_s: 0.25,
            test_firing_duration_s: 0.75,
            shutdown_duration_s: 0.5,
            max_throat_temp_k: 500.0,
        }
    }
}

pub trait EcuDriver {
    fn timestamp(&self) -> f32;

    fn set_sparking(&mut self, state: bool);
    fn get_sparking(&self) -> bool;

    fn set_binary_valve(&mut self, valve: EcuBinaryValve, state: bool);
    fn get_binary_valve(&self, valve: EcuBinaryValve) -> bool;

    fn as_mut_any(&mut self) -> &mut dyn Any;
}
