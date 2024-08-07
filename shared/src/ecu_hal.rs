use core::any::Any;

use serde::{Deserialize, Serialize};
use strum::EnumProperty;
use strum_macros::{EnumCount as EnumCountMacro, EnumDiscriminants, EnumIter};

use crate::{SensorConfig, SensorData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum EngineState {
    Idle,
    PumpStartup,
    IgniterStartup,
    EngineStartup,
    Firing,
    EngineShutdown,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum PumpState {
    Idle,
    Pumping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TankType {
    FuelMain,
    OxidizerMain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PumpType {
    FuelMain,
    OxidizerMain,
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
    FireEngine,
    ShutdownEngine,
    SetTankState((TankType, TankState)),
    SetPumpDuty((PumpType, f32)),
    ConfigureSensor {
        sensor: EcuSensor,
        config: SensorConfig,
    },
    ConfigureEcu(EcuConfig),
    GetConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EcuTelemetry {
    Telemetry(EcuTelemetryFrame),
    TankTelemetry(EcuTankTelemetryFrame),
    DebugInfo(EcuDebugInfo),
    DebugSensorMeasurement((EcuSensor, SensorData)),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EcuResponse {
    Config(EcuConfig),
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

#[derive(Debug, Clone, Copy, EnumIter, EnumProperty, PartialEq, Eq)]
pub enum EcuAlert {
    // Debug mode enabled for clarity while testing
    #[strum(props(severity = "-1"))]
    DebugModeEnabled,

    // Igniter won't start/shutdown because tank conditions are not met
    #[strum(props(severity = "1"))]
    IgniterTankOffNominal,

    // Igniter failed to reach stable combustion pressure
    #[strum(props(severity = "1"))]
    IgniterStartupTimeOut,

    // Igniter throat temperature is too high
    #[strum(props(severity = "1"))]
    IgniterThroatOverheat,

    // Engine won't start/shutdown because tank conditions are not met
    #[strum(props(severity = "1"))]
    EngineTankOffNominal,

    // Engine pumps failed to reach stable pressure
    #[strum(props(severity = "1"))]
    EngineStartupPumpTimeout,

    // Engine igniter failed to start in time
    #[strum(props(severity = "1"))]
    EngineStartupIgniterTimeout,

    // Engine igniter aborted during engine startup
    #[strum(props(severity = "1"))]
    EngineStartupIgniterAnomaly,

    // Engine failed to reach stable combustion pressure
    #[strum(props(severity = "1"))]
    EngineStartupTimeout,

    // Engine shutdown because chamber pressure was not within limits
    #[strum(props(severity = "1"))]
    EngineChamberPressureOffNominal,

    // Engine shut down because of a set timer rather than by command
    #[strum(props(severity = "0"))]
    EngineShutdownTimerExpired,
}

impl From<EcuAlert> for u128 {
    fn from(alert: EcuAlert) -> Self {
        alert as u128
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuTelemetryFrame {
    pub timestamp: u64,
    pub engine_state: EngineState,
    pub igniter_state: IgniterState,
    pub fuel_pump_state: PumpState,
    pub oxidizer_pump_state: PumpState,
    pub engine_chamber_pressure_pa: f32,
    pub engine_fuel_injector_pressure_pa: f32,
    pub engine_oxidizer_injector_pressure_pa: f32,
    pub igniter_chamber_pressure_pa: f32,
    pub fuel_pump_outlet_pressure_pa: f32,
    pub oxidizer_pump_outlet_pressure_pa: f32,
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
    PumpInfo {
        timestamp: u64,
        fuel_pump_state: PumpState,
        oxidizer_pump_state: PumpState,
    },
    SensorData {
        timestamp: u64,
        fuel_tank_pressure_pa: f32,
        oxidizer_tank_pressure_pa: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuConfig {
    pub engine_config: EngineConfig,
    pub igniter_config: IgniterConfig,
    pub tanks_config: Option<TanksConfig>,
    pub telemetry_rate_s: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineConfig {
    pub use_pumps: bool,
    pub fuel_injector_pressure_setpoint_pa: f32,
    pub fuel_injector_startup_pressure_tolerance_pa: f32,
    pub fuel_injector_running_pressure_tolerance_pa: f32,
    pub oxidizer_injector_pressure_setpoint_pa: f32,
    pub oxidizer_injector_startup_pressure_tolerance_pa: f32,
    pub oxidizer_injector_running_pressure_tolerance_pa: f32,
    pub engine_target_combustion_pressure_pa: f32,
    pub engine_combustion_pressure_tolerance_pa: f32,
    pub pump_startup_timeout_s: f32,
    pub igniter_startup_timeout_s: f32,
    pub engine_startup_timeout_s: f32,
    pub engine_firing_duration_s: Option<f32>,
    pub engine_shutdown_duration_s: f32,
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
            engine_config: EngineConfig::default(),
            igniter_config: IgniterConfig::default(),
            tanks_config: None,
            telemetry_rate_s: 0.02,
        }
    }
}

impl EngineConfig {
    pub fn default() -> Self {
        Self {
            use_pumps: true,
            fuel_injector_pressure_setpoint_pa: 500.0 * 6894.76, // 1000 PSI to Pascals
            fuel_injector_startup_pressure_tolerance_pa: 25.0 * 6894.76, // 50 PSI to Pascals
            fuel_injector_running_pressure_tolerance_pa: 100.0 * 6894.76, // 10 PSI to Pascals
            oxidizer_injector_pressure_setpoint_pa: 500.0 * 6894.76, // 1000 PSI to Pascals
            oxidizer_injector_startup_pressure_tolerance_pa: 25.0 * 6894.76, // 50 PSI to Pascals
            oxidizer_injector_running_pressure_tolerance_pa: 100.0 * 6894.76, // 10 PSI to Pascals
            engine_target_combustion_pressure_pa: 300.0 * 6894.76, // 1000 PSI to Pascals
            engine_combustion_pressure_tolerance_pa: 200.0 * 6894.76, // 100 PSI to Pascals
            pump_startup_timeout_s: 1.0,
            igniter_startup_timeout_s: 1.0,
            engine_startup_timeout_s: 1.0,
            engine_firing_duration_s: None,
            engine_shutdown_duration_s: 0.5,
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

    fn set_linear_output(&mut self, output: EcuLinearOutput, value: f32);
    fn get_linear_output(&self, output: EcuLinearOutput) -> f32;

    fn as_mut_any(&mut self) -> &mut dyn Any;
}
