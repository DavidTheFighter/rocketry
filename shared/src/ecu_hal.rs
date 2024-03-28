use core::any::Any;

use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount as EnumCountMacro, EnumDiscriminants, EnumIter};

use crate::SensorConfig;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter, Hash)]
pub enum EcuSensor {
    FuelTankPressure,
    OxidizerTankPressure,

    IgniterChamberPressure,
    IgniterFuelInjectorPressure,
    IgniterOxidizerInjectorPressure,
    IgniterThroatTemperature,

    EngineChamberPressure,
    EngineFuelInjectorPressure,
    EngineOxidizerInjectorPressure,
    EngineThroatTemperature,

    FuelPumpOutletPressure,
    FuelPumpInletPressure,
    FuelPumpInducerPressure,
    OxidizerPumpOutletPressure,
    OxidizerPumpInletPressure,
    OxidizerPumpInducerPressure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EcuCommand {
    SetBinaryValve {
        valve: EcuBinaryOutput,
        state: bool,
    },
    SetSparking(bool),
    FireIgniter,
    SetFuelTank(TankState),
    SetOxidizerTank(TankState),
    ConfigureSensor {
        sensor: EcuSensor,
        config: SensorConfig,
    },
    ConfigureEcu(EcuConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuBinaryOutput {
    IgniterFuelValve,
    IgniterOxidizerValve,
    FuelPressValve,
    FuelVentValve,
    OxidizerPressValve,
    OxidizerVentValve,
    EngineFuelValve,
    EngineOxidizerValve,
    FuelPurgeValve,
}

impl EcuBinaryOutput {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuLinearOutput {
    FuelPump,
    OxidizerPump,
}

impl EcuLinearOutput {
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
        igniter_chamber_pressure_pa: f32,
        igniter_fuel_injector_pressure_pa: Option<f32>,
        igniter_oxidizer_injector_pressure_pa: Option<f32>,
    },
    TankInfo {
        timestamp: u64,
        fuel_tank_state: TankState,
        oxidizer_tank_state: TankState,
    },
    SensorData {
        timestamp: u64,
        fuel_tank_pressure_pa: f32,
        oxidizer_tank_pressure_pa: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuConfig {
    pub igniter_config: IgniterConfig,
    pub tanks_config: Option<TanksConfig>,
    pub telemetry_rate_s: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgniterConfig {
    pub startup_timeout_s: f32,
    pub startup_pressure_threshold_pa: f32,
    pub startup_stable_time_s: f32,
    pub test_firing_duration_s: f32,
    pub shutdown_duration_s: f32,
    pub max_throat_temp_k: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TanksConfig {
    pub target_fuel_pressure_pa: f32,
    pub target_oxidizer_pressure_pa: f32,
}

impl EcuConfig {
    pub fn default() -> Self {
        Self {
            igniter_config: IgniterConfig::default(),
            tanks_config: None,
            telemetry_rate_s: 0.02,
        }
    }
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

    fn set_binary_valve(&mut self, valve: EcuBinaryOutput, state: bool);
    fn get_binary_valve(&self, valve: EcuBinaryOutput) -> bool;

    fn as_mut_any(&mut self) -> &mut dyn Any;
}
