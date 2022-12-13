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
    Startup = 1,
    Firing = 2,
    Shutdown = 3,
    Abort = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuelTankState {
    Idle = 0,
    Pressurized = 1,
}

pub struct ECUDAQFrame {
    pub sensors: [u16; 6],
}
