use serde::{Deserialize, Serialize};

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
    Startup = 2,
    Firing = 3,
    Shutdown = 4,
    Abort = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuelTankState {
    Idle = 0,
    Depressurized = 1,
    Pressurized = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUTelemetryFrame {
    pub timestamp: u64,
    pub igniter_state: IgniterState,
    pub fuel_tank_state: FuelTankState,
    pub sensors: [f32; MAX_ECU_SENSORS],
    pub solenoid_valves: [bool; MAX_ECU_VALVES],
    pub sparking: bool,
    pub cpu_utilization: u32,
}

impl ECUTelemetryFrame {
    pub const fn default() -> Self {
        Self {
            timestamp: 0,
            igniter_state: IgniterState::Idle,
            fuel_tank_state: FuelTankState::Idle,
            sensors: [0_f32; MAX_ECU_SENSORS],
            solenoid_valves: [false; MAX_ECU_VALVES],
            sparking: false,
            cpu_utilization: 0,
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
