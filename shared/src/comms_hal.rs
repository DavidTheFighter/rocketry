use big_brother::big_brother::Broadcastable;
use serde::{Deserialize, Serialize};

use crate::{
    alerts,
    ecu_hal::{EcuCommand, EcuResponse, EcuTelemetry, EcuTelemetryFrame},
    fcu_hal::{FcuDebugInfo, FcuSensorData, FcuTelemetryFrame, VehicleCommand},
    streamish_hal::StreamishCommand,
    SensorConfig,
};

use strum_macros::EnumCount as EnumCountMacro;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    FlightController,
    MissionControl,
    MissionControlSimBridge,
    EthbootProgrammer,
    Camera(u8),
    Unknown,
}

impl Broadcastable for NetworkAddress {
    fn is_broadcast(&self) -> bool {
        matches!(self, NetworkAddress::Broadcast)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationError {
    Unknown,
    PacketTooLong,
    PostcardImplementation,
    SerdeError,
    UnexpectedEnd,
    BadVar,
    BadEncoding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumCountMacro)]
pub enum Packet {
    // -- Direct commands -- //
    EnableDataLogging(bool),
    ResetMcu {
        magic_number: u64, // crate::RESET_MAGIC_NUMBER
    },

    // -- Commands -- //,
    VehicleCommand(VehicleCommand),
    EcuCommand(EcuCommand),
    StreamishCommand(StreamishCommand),

    // -- Data -- //
    EcuTelemetry(EcuTelemetry),
    EcuResponse(EcuResponse),
    FcuTelemetry(FcuTelemetryFrame),
    EnableDebugInfo(bool),
    FcuDebugInfo(FcuDebugInfo),
    FcuDebugSensorMeasurement(FcuSensorData),
    AlertBitmask(alerts::AlertBitmaskType),

    // -- Misc -- //
    DeviceBooted,
    Heartbeat,
    DoNothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketWithAddress {
    pub address: NetworkAddress,
    pub packet: Packet,
}

pub mod tests_data {
    use super::*;
    use crate::{
        ecu_hal::{
            self, EcuBinaryOutput, EcuConfig, EngineConfig, EngineState, IgniterConfig,
            IgniterState, TankConfig,
        },
        fcu_hal, SensorCalibration, RESET_MAGIC_NUMBER,
    };
    use mint::Vector3;
    use strum::EnumCount;

    const SENSOR_CALIBRATION: SensorCalibration = SensorCalibration {
        x0: 0.1,
        x1: 0.2,
        x2: 0.3,
        x3: 0.4,
    };

    const SENSOR_CONFIG: SensorConfig = SensorConfig {
        premin: 0.1,
        premax: 0.9,
        postmin: 10.0,
        postmax: 99.9,
        calibration: Some(SENSOR_CALIBRATION),
    };

    pub const ADDRESS_TEST_DEFAULTS: [NetworkAddress; 8] = [
        NetworkAddress::FlightController,
        NetworkAddress::EngineController(0),
        NetworkAddress::EngineController(42),
        NetworkAddress::EngineController(201),
        NetworkAddress::Camera(1),
        NetworkAddress::Camera(70),
        NetworkAddress::Camera(255),
        NetworkAddress::Broadcast,
    ];

    pub const PACKET_TEST_DEFAULTS: [Packet; Packet::COUNT] = [
        Packet::DeviceBooted,
        Packet::EnableDataLogging(true),
        Packet::ResetMcu {
            magic_number: RESET_MAGIC_NUMBER,
        },
        Packet::VehicleCommand(VehicleCommand::IgniteSolidMotor {
            magic_number: fcu_hal::IGNITION_MAGIC_NUMBER,
        }),
        Packet::EcuCommand(EcuCommand::SetSparking(true)),
        Packet::StreamishCommand(StreamishCommand::StartCameraStream { port: 25565 }),
        Packet::FcuTelemetry(FcuTelemetryFrame::default()),
        Packet::EcuTelemetry(EcuTelemetry::Telemetry(EcuTelemetryFrame {
            timestamp: 0xABAD_1234_FEDC_DEAD,
            engine_state: EngineState::Idle,
            igniter_state: IgniterState::Shutdown,
            engine_chamber_pressure_pa: 1234.567,
            engine_fuel_injector_pressure_pa: 19522.4,
            engine_oxidizer_injector_pressure_pa: 96420.425,
            igniter_chamber_pressure_pa: 1234.567,
            fuel_pump_state: ecu_hal::PumpState::Idle,
            oxidizer_pump_state: ecu_hal::PumpState::Idle,
            fuel_pump_outlet_pressure_pa: 19522.4,
            oxidizer_pump_outlet_pressure_pa: 96420.425,
        })),
        Packet::EcuResponse(EcuResponse::Config(EcuConfig {
            engine_config: Some(EngineConfig {
                use_pumps: true,
                fuel_injector_pressure_setpoint_pa: 1234.567,
                fuel_injector_startup_pressure_tolerance_pa: 0.14962,
                fuel_injector_running_pressure_tolerance_pa: 0.14962,
                oxidizer_injector_pressure_setpoint_pa: 0.14962,
                oxidizer_injector_startup_pressure_tolerance_pa: 0.14962,
                oxidizer_injector_running_pressure_tolerance_pa: 0.14962,
                engine_target_combustion_pressure_pa: 0.14962,
                engine_combustion_pressure_tolerance_pa: 0.14962,
                pump_startup_timeout_s: 0.14962,
                igniter_startup_timeout_s: 0.14962,
                engine_startup_timeout_s: 0.14962,
                engine_firing_duration_s: Some(79.21968),
                engine_shutdown_duration_s: 0.14962,
            }),
            igniter_config: Some(IgniterConfig {
                startup_timeout_s: 749.248,
                startup_pressure_threshold_pa: 749.248,
                startup_stable_time_s: 749.248,
                test_firing_duration_s: 749.248,
                shutdown_duration_s: 749.248,
                max_throat_temp_k: 749.248,
            }),
            fuel_tank_config: Some(TankConfig {
                press_valve: None,
                fill_valve: Some(EcuBinaryOutput::FuelFillValve),
                vent_valve: Some(EcuBinaryOutput::FuelVentValve),
                press_min_threshold_pa: 0.14962,
                press_max_threshold_pa: 1542.012,
            }),
            oxidizer_tank_config: None,
            telemetry_rate_s: 0.945218,
        })),
        Packet::AlertBitmask(0xAAAA_AAAA),
        Packet::EnableDebugInfo(true),
        Packet::FcuDebugInfo(FcuDebugInfo::default()),
        Packet::FcuDebugSensorMeasurement(FcuSensorData::Accelerometer {
            acceleration: Vector3 {
                x: 0.1,
                y: 0.2,
                z: 0.3,
            },
            raw_data: Vector3 {
                x: 14,
                y: -72,
                z: 19852,
            },
        }),
        Packet::Heartbeat,
        Packet::DoNothing,
    ];
}

#[cfg(test)]
pub mod tests {
    use super::tests_data::*;
    use super::*;
    use std::io::Write;

    use big_brother::{
        big_brother::WORKING_BUFFER_SIZE,
        serdes::{deserialize_postcard, serialize_postcard},
    };

    #[test]
    fn test_packet_sizes() {
        let mut buffer = Vec::with_capacity(1024 * 1024);
        for _ in 0..buffer.capacity() {
            buffer.push(0u8);
        }

        let mut file = std::fs::File::create("../packet_sizes.txt").unwrap();

        let mut max_metadata_size = 0;

        for address in &ADDRESS_TEST_DEFAULTS {
            let bytes_written = serialize_postcard(address, &mut buffer).unwrap();
            assert!(bytes_written <= WORKING_BUFFER_SIZE);

            max_metadata_size = max_metadata_size.max(bytes_written);
        }

        let line = format!("Max metadata size: {},\n\n", max_metadata_size);
        file.write_all(line.as_bytes()).unwrap();

        for packet in &PACKET_TEST_DEFAULTS {
            println!("Serializing: {:?}", packet);
            let bytes_written = serialize_postcard(packet, &mut buffer).unwrap();
            assert!(bytes_written <= WORKING_BUFFER_SIZE);

            let packet_name = format!("{:?}", packet);
            let packet_name = packet_name.split('(').next().unwrap();
            let packet_name = packet_name.split(' ').next().unwrap();

            let line = format!("{}: {},\n", packet_name, bytes_written);
            file.write_all(line.as_bytes()).unwrap();
        }
    }

    #[test]
    fn packet_reserialization() {
        let mut buffer = [0u8; WORKING_BUFFER_SIZE];

        for packet in &PACKET_TEST_DEFAULTS {
            let bytes_written = serialize_postcard(packet, &mut buffer).unwrap();
            let reserialized_packet: Packet =
                deserialize_postcard(&mut buffer[..bytes_written]).unwrap();
            assert_eq!(*packet, reserialized_packet);
        }
    }

    #[test]
    fn address_reserialization() {
        let mut buffer = [0u8; WORKING_BUFFER_SIZE];

        for address in &ADDRESS_TEST_DEFAULTS {
            let bytes_written = serialize_postcard(address, &mut buffer).unwrap();
            let reserialized_address: NetworkAddress =
                deserialize_postcard(&mut buffer[..bytes_written]).unwrap();
            assert_eq!(*address, reserialized_address);
        }
    }
}
