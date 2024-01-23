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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuSensor {
    IgniterFuelInjectorPressure = 0,
    IgniterGOxInjectorPressure = 1,
    IgniterChamberPressure = 2,
    FuelTankPressure = 3,
    ECUBoardTemp = 4,
    IgniterThroatTemp = 5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EcuCommand {
    SetSolenoidValve {
        valve: EcuSolenoidValve,
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
pub enum EcuSolenoidValve {
    IgniterFuelMain,
    IgniterGOxMain,
    FuelPress,
    FuelVent,
    OxidizerPress,
    OxidizerVent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuTelemetryFrame {
    pub timestamp: u64,
    pub engine_state: EngineState,
    pub igniter_state: IgniterState,
    pub fuel_tank_state: TankState,
    pub oxidizer_tank_state: TankState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(FcuDebugInfoVariant))]
#[strum_discriminants(derive(EnumIter))]
pub enum EcuDebugInfo {
    IgniterInfo {
        timestamp: u64,
        igniter_state: IgniterState,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcuConfig {
    pub igniter_config: IgniterConfig,
    pub telemetry_rate_s: f32,
}

impl EcuConfig {
    pub const fn default() -> Self {
        Self {
            igniter_config: IgniterConfig::default(),
            telemetry_rate_s: 0.02,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgniterConfig {
    pub gox_lead: bool,
    pub gox_lead_duration: f32,
    pub startup_timeout: f32,
    pub startup_pressure_threshold: f32, // TODO: This is in PSI, it should be in Pascals
    pub startup_stable_time: f32,
    pub firing_duration: f32,
    pub shutdown_duration: f32,
    pub max_throat_temp: f32, // In Celsius
}

impl IgniterConfig {
    pub const fn default() -> Self {
        Self {
            gox_lead: false,
            gox_lead_duration: 0.25,
            startup_timeout: 1.0,
            startup_pressure_threshold: 30.0,
            startup_stable_time: 0.25,
            firing_duration: 0.75,
            shutdown_duration: 0.5,
            max_throat_temp: 500.0,
        }
    }
}

pub trait EcuDriver {
    fn timestamp(&self) -> f32;

    fn set_solenoid_valve(&mut self, valve: EcuSolenoidValve, state: bool);
    fn set_sparking(&mut self, state: bool);

    fn get_solenoid_valve(&self, valve: EcuSolenoidValve) -> bool;

    // TODO - Make this an option, because sensors will not always be available (configurable!)
    fn get_sensor(&self, sensor: EcuSensor) -> f32;
    fn get_sparking(&self) -> bool;

    fn send_packet(&mut self, packet: Packet, destination: NetworkAddress);

    fn generate_telemetry_frame(&self) -> EcuTelemetryFrame;

    fn configure_sensor(&mut self, sensor: EcuSensor, config: SensorConfig);

    fn as_mut_any(&mut self) -> &mut dyn Any;
}
