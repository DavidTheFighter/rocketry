use postcard::{
    from_bytes_cobs,
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use serde::{Deserialize, Serialize};

use crate::{
    ecu_hal::{
        EcuDAQFrame, EcuTelemetryFrame, EcuSensor, EcuSolenoidValve, FuelTankState, IgniterConfig,
    },
    SensorConfig,
};

pub const DAQ_PACKET_FRAMES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    FlightController,
    MissionControl,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // -- Commands -- //,
    TransitionFuelTankState(FuelTankState),
    FireIgniter,

    // -- Data -- //
    EcuTelemetry(EcuTelemetryFrame),
    EcuDAQ([EcuDAQFrame; DAQ_PACKET_FRAMES]),
}

impl Packet {
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
                    Err(err) => match err {
                        postcard::Error::WontImplement
                        | postcard::Error::NotYetImplemented
                        | postcard::Error::SerializeSeqLengthUnknown => {
                            Err(SerializationError::PostcardImplementation)
                        }
                        postcard::Error::SerializeBufferFull => {
                            Err(SerializationError::PacketTooLong)
                        }
                        postcard::Error::SerdeSerCustom | postcard::Error::SerdeDeCustom => {
                            Err(SerializationError::SerdeError)
                        }
                        _ => Err(SerializationError::Unknown),
                    },
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
            Err(err) => match err {
                postcard::Error::WontImplement | postcard::Error::NotYetImplemented => {
                    Err(SerializationError::PostcardImplementation)
                }
                postcard::Error::SerdeSerCustom | postcard::Error::SerdeDeCustom => {
                    Err(SerializationError::SerdeError)
                }
                postcard::Error::DeserializeUnexpectedEnd => Err(SerializationError::UnexpectedEnd),
                postcard::Error::DeserializeBadVarint
                | postcard::Error::DeserializeBadBool
                | postcard::Error::DeserializeBadChar
                | postcard::Error::DeserializeBadUtf8
                | postcard::Error::DeserializeBadOption
                | postcard::Error::DeserializeBadEnum => Err(SerializationError::BadVar),
                postcard::Error::DeserializeBadEncoding => Err(SerializationError::BadEncoding),
                _ => Err(SerializationError::Unknown),
            },
        }
    }
}
