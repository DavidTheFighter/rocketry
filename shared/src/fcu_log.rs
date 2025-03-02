use core::fmt;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum DataPoint {
    Accelerometer { x: i16, y: i16, z: i16 },
    Gyroscope { x: i16, y: i16, z: i16 },
    Magnetometer { x: i16, y: i16, z: i16, h: i16 },
}

#[derive(Debug, Clone)]
pub struct DataLogBuffer {
    pub buffer: [u8; 256],
}

impl DataPoint {
    pub fn serialize<'a>(&self, buffer: &'a mut [u8; 16]) -> &'a [u8] {
        match self {
            DataPoint::Accelerometer { x, y, z } => {
                buffer[0] = 0x01;
                buffer[1] = (x >> 8) as u8;
                buffer[2] = (x & 0xFF) as u8;
                buffer[3] = (y >> 8) as u8;
                buffer[4] = (y & 0xFF) as u8;
                buffer[5] = (z >> 8) as u8;
                buffer[6] = (z & 0xFF) as u8;
                &buffer[..7]
            }
            DataPoint::Gyroscope { x, y, z } => {
                buffer[0] = 0x02;
                buffer[1] = (x >> 8) as u8;
                buffer[2] = (x & 0xFF) as u8;
                buffer[3] = (y >> 8) as u8;
                buffer[4] = (y & 0xFF) as u8;
                buffer[5] = (z >> 8) as u8;
                buffer[6] = (z & 0xFF) as u8;
                &buffer[..7]
            }
            DataPoint::Magnetometer { x, y, z, h } => {
                buffer[0] = 0x03;
                buffer[1] = (x >> 8) as u8;
                buffer[2] = (x & 0xFF) as u8;
                buffer[3] = (y >> 8) as u8;
                buffer[4] = (y & 0xFF) as u8;
                buffer[5] = (z >> 8) as u8;
                buffer[6] = (z & 0xFF) as u8;
                buffer[7] = (h >> 8) as u8;
                buffer[8] = (h & 0xFF) as u8;
                &buffer[..9]
            }
        }
    }

    pub fn deserialize(buffer: &[u8]) -> (Option<DataPoint>, Option<&[u8]>) {
        if buffer.is_empty() {
            return (None, None);
        }

        let data_type = buffer[0];
        let data = &buffer[1..];

        match data_type {
            0x01 => {
                if data.len() < 6 {
                    return (None, None);
                }

                let x = (i16::from(data[0]) << 8) | i16::from(data[1]);
                let y = (i16::from(data[2]) << 8) | i16::from(data[3]);
                let z = (i16::from(data[4]) << 8) | i16::from(data[5]);

                (Some(DataPoint::Accelerometer { x, y, z }), Some(&data[6..]))
            }
            0x02 => {
                if data.len() < 6 {
                    return (None, None);
                }

                let x = (i16::from(data[0]) << 8) | i16::from(data[1]);
                let y = (i16::from(data[2]) << 8) | i16::from(data[3]);
                let z = (i16::from(data[4]) << 8) | i16::from(data[5]);

                (Some(DataPoint::Gyroscope { x, y, z }), Some(&data[6..]))
            }
            0x03 => {
                if data.len() < 8 {
                    return (None, None);
                }

                let x = (i16::from(data[0]) << 8) | i16::from(data[1]);
                let y = (i16::from(data[2]) << 8) | i16::from(data[3]);
                let z = (i16::from(data[4]) << 8) | i16::from(data[5]);
                let h = (i16::from(data[6]) << 8) | i16::from(data[7]);

                (
                    Some(DataPoint::Magnetometer { x, y, z, h }),
                    Some(&data[8..]),
                )
            }
            _ => (None, None),
        }
    }
}

impl Serialize for DataLogBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.buffer)
    }
}

impl<'de> Deserialize<'de> for DataLogBuffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(DataLogBufferVisitor)
    }
}

struct DataLogBufferVisitor;

impl<'de> Visitor<'de> for DataLogBufferVisitor {
    type Value = DataLogBuffer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let mut buffer = [0u8; 256];
        buffer.copy_from_slice(v);
        Ok(DataLogBuffer { buffer })
    }
}
