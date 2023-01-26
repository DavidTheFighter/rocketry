use core::any::Any;

use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::SensorConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuSensor {
    IgniterFuelInjectorPressure = 0,
    IgniterGOxInjectorPressure = 1,
    IgniterChamberPressure = 2,
    FuelTankPressure = 3,
    ECUBoardTemp = 4,
    IgniterThroatTemp = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumCountMacro, EnumIter)]
pub enum EcuSolenoidValve {
    IgniterFuelMain = 0,
    IgniterGOxMain = 1,
    FuelPress = 2,
    FuelVent = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum IgniterState {
    Idle = 0,
    Startup = 1,
    Firing = 2,
    Shutdown = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum FuelTankState {
    Idle = 0,
    Depressurized = 1,
    Pressurized = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcuTelemetryFrame {
    pub timestamp: u64,
    pub igniter_state: IgniterState,
    pub fuel_tank_state: FuelTankState,
    pub sensors: [f32; EcuSensor::COUNT],
    pub solenoid_valves: [bool; EcuSolenoidValve::COUNT],
    pub sparking: bool,
    pub cpu_utilization: u32,
}

impl EcuTelemetryFrame {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            igniter_state: IgniterState::Idle,
            fuel_tank_state: FuelTankState::Idle,
            sensors: [0_f32; EcuSensor::COUNT],
            solenoid_valves: [false; EcuSolenoidValve::COUNT],
            sparking: false,
            cpu_utilization: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct EcuDAQFrame {
    pub sensor_values: [u16; EcuSensor::COUNT],
}

impl EcuDAQFrame {
    pub const fn default() -> Self {
        Self {
            sensor_values: [0_u16; EcuSensor::COUNT],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn set_solenoid_valve(&mut self, valve: EcuSolenoidValve, state: bool);
    fn set_sparking(&mut self, state: bool);

    fn get_solenoid_valve(&self, valve: EcuSolenoidValve) -> bool;

    // TODO - Make this an option, because sensors will not always be available (configurable!)
    fn get_sensor(&self, sensor: EcuSensor) -> f32;
    fn get_sparking(&self) -> bool;

    fn generate_telemetry_frame(&self) -> EcuTelemetryFrame;

    /// Collects the data the DAQ has measured since the last time this was called. This is meant
    /// so that the DAQ can run independently of the ECU loop. Each call to this resets the stored
    /// min/max values for this particular sensor so the DAQ can update them until the next ECU loop.
    ///
    /// Returns the current sensor value, minimum value since the last call, and the maximum
    /// value since the last call.
    fn collect_daq_sensor_data(&mut self, sensor: EcuSensor) -> (f32, f32, f32);

    fn configure_sensor(&mut self, sensor: EcuSensor, config: SensorConfig);

    fn as_mut_any(&mut self) -> &mut dyn Any;
}
