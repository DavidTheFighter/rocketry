use serde::{Deserialize, Serialize};

use crate::{ecu_hal::{ECUSolenoidValve, ECUSensor, IgniterState, FuelTankState, MAX_ECU_SENSORS, MAX_ECU_VALVES}, SensorConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    MissionControl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    // -- Direct commands -- //
    SetSolenoidValve {
        valve: ECUSolenoidValve,
        state: bool,
    },
    SetSparking(bool),
    ConfigureSensor {
        sensor: ECUSensor,
        config: SensorConfig,
    },

    // -- Commands -- //,
    PressurizeFuelTank,
    DepressurizeFuelTank,
    
    // -- Data -- //
    ECUTelemetry {
        igniter_state: IgniterState,
        fuel_tank_state: FuelTankState,
        sensors: [f32; MAX_ECU_SENSORS],
        solenoid_valves: [bool; MAX_ECU_VALVES],
    },
}
