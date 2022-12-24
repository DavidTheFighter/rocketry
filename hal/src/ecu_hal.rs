use serde::{Deserialize, Serialize};

use crate::SensorConfig;

pub const MAX_ECU_SENSORS: usize = 6;
pub const MAX_ECU_VALVES: usize = 4;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ECUSensor {
    IgniterFuelInjectorPressure = 0,
    IgniterGOxInjectorPressure = 1,
    IgniterChamberPressure = 2,
    FuelTankPressure = 3,
    ECUBoardTemp = 4,
    IgniterThroatTemp = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ECUSolenoidValve {
    IgniterFuelMain = 0,
    IgniterGOxMain = 1,
    FuelPress = 2,
    FuelVent = 3,
}

pub const ECU_SENSORS: [ECUSensor; MAX_ECU_SENSORS] = [
    ECUSensor::IgniterFuelInjectorPressure,
    ECUSensor::IgniterGOxInjectorPressure,
    ECUSensor::IgniterChamberPressure,
    ECUSensor::FuelTankPressure,
    ECUSensor::ECUBoardTemp,
    ECUSensor::IgniterThroatTemp,
];

pub const ECU_SOLENOID_VALVES: [ECUSolenoidValve; MAX_ECU_VALVES] = [
    ECUSolenoidValve::IgniterFuelMain,
    ECUSolenoidValve::IgniterGOxMain,
    ECUSolenoidValve::FuelPress,
    ECUSolenoidValve::FuelVent,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgniterState {
    Idle = 0,
    StartupGOxLead = 1,
    StartupIgnition = 2,
    Firing = 3,
    Shutdown = 4,
    Abort = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuelTankState {
    Idle = 0,
    Pressurized = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUTelemetryFrame {
    pub igniter_state: IgniterState,
    pub fuel_tank_state: FuelTankState,
    pub sensors: [f32; MAX_ECU_SENSORS],
    pub solenoid_valves: [bool; MAX_ECU_VALVES],
    pub sparking: bool,
}

impl ECUTelemetryFrame {
    pub const fn default() -> Self {
        Self {
            igniter_state: IgniterState::Idle,
            fuel_tank_state: FuelTankState::Idle,
            sensors: [0_f32; MAX_ECU_SENSORS],
            solenoid_valves: [false; MAX_ECU_VALVES],
            sparking: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ECUDAQFrame {
    pub sensor_values: [u16; MAX_ECU_SENSORS],
}

impl ECUDAQFrame {
    pub const fn default() -> Self {
        Self {
            sensor_values: [0_u16; MAX_ECU_SENSORS],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ECUConfiguration {
    pub sensor_configs: [SensorConfig; MAX_ECU_SENSORS],
}

impl ECUConfiguration {
    pub const fn default() -> Self {
        Self {
            sensor_configs: [SensorConfig::default(); MAX_ECU_SENSORS],
        }
    }
}
