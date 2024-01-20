use big_brother::big_brother::Broadcastable;
use serde::{Deserialize, Serialize};

use crate::{
    ecu_hal::{
        EcuDAQFrame, EcuSensor, EcuSolenoidValve, EcuTelemetryFrame, FuelTankState, IgniterConfig,
    },
    fcu_hal::{FcuConfig, FcuDebugInfo, FcuDevStatsFrame, FcuTelemetryFrame, FcuSensorData},
    SensorConfig,
};

use strum_macros::EnumCount as EnumCountMacro;

pub const DAQ_PACKET_FRAMES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    FlightController,
    MissionControl,
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
    SetSolenoidValve {
        valve: EcuSolenoidValve,
        state: bool,
    },
    SetSparking(bool),
    DeviceBooted,
    ConfigureSensor {
        sensor: EcuSensor,
        config: SensorConfig,
    },
    ConfigureIgniter(IgniterConfig),
    ConfigureFcu(FcuConfig),
    EraseDataLogFlash,
    EnableDataLogging(bool),
    RetrieveDataLogPage(u32),
    ResetMcu {
        magic_number: u64, // crate::RESET_MAGIC_NUMBER
    },

    // -- Dev Only -- //
    StartDevStatsFrame,

    // -- Commands -- //,
    TransitionFuelTankState(FuelTankState),
    FireIgniter,
    StartCameraStream {
        port: u16,
    },
    StopCameraStream,
    StartCalibration {
        zero: bool,
    },
    ArmVehicle {
        magic_number: u64, // fcu_hal::ARMING_MAGIC_NUMBER
    },
    IgniteSolidMotor {
        magic_number: u64, // fcu_hal::IGNITION_MAGIC_NUMBER
    },
    EnterBootloader,

    // -- Data -- //
    FcuTelemetry(FcuTelemetryFrame),
    EcuTelemetry(EcuTelemetryFrame),
    EnableDebugInfo(bool),
    FcuDebugInfo(FcuDebugInfo),
    FcuDevStatsFrame(FcuDevStatsFrame),
    FcuDebugSensorMeasurement(FcuSensorData),
    EcuDAQ([EcuDAQFrame; DAQ_PACKET_FRAMES]),
    AlertBitmask(u32),
    // FcuDataLogPage(DataLogBuffer),

    // -- Misc -- //
    Heartbeat,
    StopApplication,
    DoNothing,
}

pub mod tests_data {
    use super::*;
    use crate::{SensorCalibration, fcu_hal::{ARMING_MAGIC_NUMBER, IGNITION_MAGIC_NUMBER}, RESET_MAGIC_NUMBER};
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
        Packet::SetSolenoidValve {
            valve: EcuSolenoidValve::IgniterFuelMain,
            state: true,
        },
        Packet::SetSparking(true),
        Packet::DeviceBooted,
        Packet::ConfigureSensor {
            sensor: EcuSensor::IgniterGOxInjectorPressure,
            config: SENSOR_CONFIG,
        },
        Packet::ConfigureIgniter(IgniterConfig::default()),
        Packet::ConfigureFcu(FcuConfig::default()),
        Packet::EraseDataLogFlash,
        Packet::EnableDataLogging(true),
        Packet::RetrieveDataLogPage(42),
        Packet::ResetMcu {
            magic_number: RESET_MAGIC_NUMBER,
        },
        Packet::StartDevStatsFrame,
        Packet::TransitionFuelTankState(FuelTankState::Pressurized),
        Packet::FireIgniter,
        Packet::StartCameraStream { port: 42 },
        Packet::StopCameraStream,
        Packet::StartCalibration { zero: true },
        Packet::ArmVehicle { magic_number: ARMING_MAGIC_NUMBER },
        Packet::IgniteSolidMotor { magic_number: IGNITION_MAGIC_NUMBER },
        Packet::EnterBootloader,
        Packet::FcuTelemetry(FcuTelemetryFrame::default()),
        Packet::EcuTelemetry(EcuTelemetryFrame::default()),
        Packet::AlertBitmask(0xAAAA_AAAA),
        Packet::EnableDebugInfo(true),
        Packet::FcuDebugInfo(FcuDebugInfo::default()),
        Packet::FcuDevStatsFrame(FcuDevStatsFrame::default()),
        Packet::FcuDebugSensorMeasurement(FcuSensorData::Accelerometer {
            acceleration: Vector3 { x: 0.1, y: 0.2, z: 0.3 },
            raw_data: Vector3 { x: 14, y: -72, z: 19852 },
        }),
        Packet::EcuDAQ([EcuDAQFrame::default(); DAQ_PACKET_FRAMES]),
        Packet::Heartbeat,
        Packet::StopApplication,
        Packet::DoNothing,
    ];
}

#[cfg(test)]
pub mod tests {
    use super::tests_data::*;
    use super::*;
    use std::io::Write;

    use big_brother::{serdes::{serialize_postcard, deserialize_postcard}, big_brother::WORKING_BUFFER_SIZE};

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
            let reserialized_packet: Packet = deserialize_postcard(&mut buffer[..bytes_written]).unwrap();
            assert_eq!(*packet, reserialized_packet);
        }
    }

    #[test]
    fn address_reserialization() {
        let mut buffer = [0u8; WORKING_BUFFER_SIZE];

        for address in &ADDRESS_TEST_DEFAULTS {
            let bytes_written = serialize_postcard(address, &mut buffer).unwrap();
            let reserialized_address: NetworkAddress = deserialize_postcard(&mut buffer[..bytes_written]).unwrap();
            assert_eq!(*address, reserialized_address);
        }
    }
}
