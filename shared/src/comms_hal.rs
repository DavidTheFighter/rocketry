use postcard::{
    from_bytes_cobs,
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use serde::{Deserialize, Serialize};

use crate::{
    ecu_hal::{
        EcuDAQFrame, EcuSensor, EcuSolenoidValve, EcuTelemetryFrame, FuelTankState, IgniterConfig,
    },
    fcu_hal::{FcuConfig, FcuDebugInfo, FcuDevStatsFrame, FcuTelemetryFrame},
    SensorConfig,
};

use strum_macros::EnumCount as EnumCountMacro;

pub const DAQ_PACKET_FRAMES: usize = 10;
pub const PACKET_BUFFER_SIZE: usize = 256;

pub const UDP_RECV_PORT: u16 = 25565;
pub const UDP_SEND_PORT: u16 = 25566;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    FlightController,
    MissionControl,
    GroundCamera(u8),
    Unknown,
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
    RequestFcuDebugInfo,
    FcuDebugInfo(FcuDebugInfo),
    FcuDevStatsFrame(FcuDevStatsFrame),
    EcuDAQ([EcuDAQFrame; DAQ_PACKET_FRAMES]),
    AlertBitmask(u32),
    // FcuDataLogPage(DataLogBuffer),

    // -- Misc -- //
    Heartbeat,
    StopApplication,
    DoNothing,
}

impl Packet {
    pub fn allow_drop(&self) -> bool {
        matches!(
            self,
            Packet::EcuTelemetry(_)
                | Packet::FcuTelemetry(_)
                | Packet::EcuDAQ(_)
                | Packet::FcuDevStatsFrame(_)
        )
    }

    /// Serializes this packet and writes it to the given buffer.
    ///
    /// # Errors
    /// If the buffer is too short or the packet cannot be serialized, an error is returned.
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        match Cobs::try_new(Slice::new(buffer)) {
            Ok(flavor) => {
                let serialized =
                    serialize_with_flavor::<Packet, Cobs<Slice>, &mut [u8]>(self, flavor);

                match serialized {
                    Ok(output_buffer) => Ok(output_buffer.len()),
                    Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
                }
            }
            Err(_err) => Err(SerializationError::Unknown),
        }
    }

    /// Deserializes a packet from the given buffer and returns that packet.
    ///
    /// # Errors
    /// If the data within the buffer is incorrect for whatever reason then an error is returned.
    pub fn deserialize(buffer: &mut [u8]) -> Result<Packet, SerializationError> {
        match from_bytes_cobs(buffer) {
            Ok(packet) => Ok(packet),
            Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
        }
    }
}

impl NetworkAddress {
    /// Serializes this network address and writes it to the given buffer.
    ///
    /// # Errors
    /// If the buffer is too short or the address cannot be serialized, an error is returned.
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        match Cobs::try_new(Slice::new(buffer)) {
            Ok(flavor) => {
                let serialized =
                    serialize_with_flavor::<NetworkAddress, Cobs<Slice>, &mut [u8]>(self, flavor);

                match serialized {
                    Ok(output_buffer) => Ok(output_buffer.len()),
                    Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
                }
            }
            Err(_err) => Err(SerializationError::Unknown),
        }
    }

    /// Deserializes a network address from the given buffer and returns that address.
    ///
    /// # Errors
    /// If the data within the buffer is incorrect for whatever reason then an error is returned.
    pub fn deserialize(buffer: &mut [u8]) -> Result<NetworkAddress, SerializationError> {
        match from_bytes_cobs(buffer) {
            Ok(address) => Ok(address),
            Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
        }
    }
}

fn postcard_serialization_err_to_hal_err(err: postcard::Error) -> SerializationError {
    match err {
        postcard::Error::WontImplement
        | postcard::Error::NotYetImplemented
        | postcard::Error::SerializeSeqLengthUnknown => SerializationError::PostcardImplementation,
        postcard::Error::SerializeBufferFull => SerializationError::PacketTooLong,
        postcard::Error::SerdeSerCustom | postcard::Error::SerdeDeCustom => {
            SerializationError::SerdeError
        }
        postcard::Error::DeserializeUnexpectedEnd => SerializationError::UnexpectedEnd,
        postcard::Error::DeserializeBadVarint
        | postcard::Error::DeserializeBadBool
        | postcard::Error::DeserializeBadChar
        | postcard::Error::DeserializeBadUtf8
        | postcard::Error::DeserializeBadOption
        | postcard::Error::DeserializeBadEnum => SerializationError::BadVar,
        postcard::Error::DeserializeBadEncoding => SerializationError::BadEncoding,
        _ => SerializationError::Unknown,
    }
}

pub mod tests_data {
    use super::*;
    use crate::{SensorCalibration, fcu_hal::{ARMING_MAGIC_NUMBER, IGNITION_MAGIC_NUMBER}, RESET_MAGIC_NUMBER};
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
        Packet::AlertBitmask(0x1234),
        Packet::RequestFcuDebugInfo,
        Packet::FcuDebugInfo(FcuDebugInfo::default()),
        Packet::FcuDevStatsFrame(FcuDevStatsFrame::default()),
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

    #[test]
    fn test_packet_sizes() {
        let mut buffer = Vec::with_capacity(1024 * 1024);
        for _ in 0..buffer.capacity() {
            buffer.push(0u8);
        }

        let mut file = std::fs::File::create("../packet_sizes.txt").unwrap();

        for packet in &PACKET_TEST_DEFAULTS {
            println!("Serializing: {:?}", packet);
            let bytes_written = packet.serialize(&mut buffer[0..]).unwrap();
            assert!(bytes_written <= PACKET_BUFFER_SIZE);

            let packet_name = format!("{:?}", packet);
            let packet_name = packet_name.split('(').next().unwrap();
            let packet_name = packet_name.split(' ').next().unwrap();

            let line = format!("{}: {},\n", packet_name, bytes_written);
            file.write_all(line.as_bytes()).unwrap();
        }
    }

    #[test]
    fn packet_reserialization() {
        let mut buffer = [0u8; PACKET_BUFFER_SIZE];

        for packet in &PACKET_TEST_DEFAULTS {
            let bytes_written = packet.serialize(&mut buffer).unwrap();
            let reserialized_packet = Packet::deserialize(&mut buffer[0..bytes_written]).unwrap();

            assert_eq!(*packet, reserialized_packet);
        }
    }
}
