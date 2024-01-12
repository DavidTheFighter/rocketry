use postcard::{
    from_bytes_cobs,
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use serde::{Deserialize, Serialize};

use crate::{big_brother::BigBrotherError, dedupe};

// Serialization format:
// u8: metadata size
// u8: packet size
// [u8; metadata size]: metadata
// [u8; packet size]: packet

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketMetadata<T> {
    pub to_addr: T,
    pub from_addr: T,
    pub counter: dedupe::CounterType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SerdesError {
    Unknown,
    PacketTooLong,
    PostcardImplementation,
    SerdeError,
    UnexpectedEnd,
    BadVar,
    BadEncoding,
}

pub fn serialize_packet<P, A>(
    packet: &P,
    host_addr: A,
    destination: A,
    counter: dedupe::CounterType,
    buffer: &mut [u8],
) -> Result<usize, BigBrotherError>
where
    P: Serialize,
    A: Serialize,
{
    let metadata = PacketMetadata {
        to_addr: destination,
        from_addr: host_addr,
        counter,
    };

    let mut buf_ptr = 2;

    let metadata_size = serialize_postcard(&metadata, &mut buffer[buf_ptr..])
        .map_err(|e| BigBrotherError::SerializationError(e))?;
    buf_ptr += metadata_size;

    let packet_size = serialize_postcard(packet, &mut buffer[buf_ptr..])
        .map_err(|e| BigBrotherError::SerializationError(e))?;
    buf_ptr += packet_size;

    buffer[0] = u8::try_from(metadata_size)
        .map_err(|_| BigBrotherError::SerializationError(SerdesError::PacketTooLong))?;
    buffer[1] = u8::try_from(packet_size)
        .map_err(|_| BigBrotherError::SerializationError(SerdesError::PacketTooLong))?;

    Ok(buf_ptr)
}

pub fn deserialize_metadata<'a, A>(
    buffer: &'a [u8],
) -> Result<PacketMetadata<A>, BigBrotherError>
where
    A: Deserialize<'a>,
{
    let metadata_size = buffer[0] as usize;

    let mut buf_ptr = 2;

    let metadata = deserialize_postcard(&buffer[buf_ptr..(buf_ptr + metadata_size)])
        .map_err(|e| BigBrotherError::SerializationError(e))?;
    buf_ptr += metadata_size;

    Ok(metadata)
}

pub fn deserialize_packet<'a, T>(buffer: &'a [u8]) -> Result<T, BigBrotherError>
where
    T: Deserialize<'a>,
{
    let metadata_size = buffer[0] as usize;
    let packet_size = buffer[1] as usize;

    let mut buf_ptr = 2 + metadata_size;

    let packet = deserialize_postcard(&buffer[buf_ptr..(buf_ptr + packet_size)])
        .map_err(|e| BigBrotherError::SerializationError(e))?;
    buf_ptr += packet_size;

    Ok(packet)
}

pub fn serialize_postcard<T>(value: &T, buffer: &mut [u8]) -> Result<usize, SerdesError>
where
    T: Serialize,
{
    match postcard::to_slice(value, buffer) {
        Ok(buffer) => Ok(buffer.len()),
        Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
    }
}

pub fn deserialize_postcard<'a, T>(buffer: &'a [u8]) -> Result<T, SerdesError>
where
    T: Deserialize<'a>,
{
    match postcard::from_bytes(buffer) {
        Ok(value) => Ok(value),
        Err(err) => Err(postcard_serialization_err_to_hal_err(err)),
    }
}

fn postcard_serialization_err_to_hal_err(err: postcard::Error) -> SerdesError {
    match err {
        postcard::Error::WontImplement
        | postcard::Error::NotYetImplemented
        | postcard::Error::SerializeSeqLengthUnknown => SerdesError::PostcardImplementation,
        postcard::Error::SerializeBufferFull => SerdesError::PacketTooLong,
        postcard::Error::SerdeSerCustom | postcard::Error::SerdeDeCustom => SerdesError::SerdeError,
        postcard::Error::DeserializeUnexpectedEnd => SerdesError::UnexpectedEnd,
        postcard::Error::DeserializeBadVarint
        | postcard::Error::DeserializeBadBool
        | postcard::Error::DeserializeBadChar
        | postcard::Error::DeserializeBadUtf8
        | postcard::Error::DeserializeBadOption
        | postcard::Error::DeserializeBadEnum => SerdesError::BadVar,
        postcard::Error::DeserializeBadEncoding => SerdesError::BadEncoding,
        _ => SerdesError::Unknown,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::{
        big_brother::WORKING_BUFFER_SIZE,
        network_map::tests::{TestNetworkAddress, NETWORK_ADDRESS_TEST_DEFAULTS},
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TestPacketStruct {
        pub a: u8,
        pub b: bool,
        pub c: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TestPacketNestedStruct {
        pub a: u8,
        pub b: bool,
        pub c: TestPacketStruct,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestPacket {
        Unit,
        NewType(u32),
        Tuple((u8, u32)),
        InlineStruct { a: u8, b: bool, c: u32 },
        Struct(TestPacketStruct),
        NestedStruct(TestPacketNestedStruct),
    }

    pub const TEST_PACKET_DEFAULTS: [TestPacket; 6] = [
        TestPacket::Unit,
        TestPacket::NewType(0xAAAAAAAA),
        TestPacket::Tuple((0x12, 0xFFFFFFFF)),
        TestPacket::InlineStruct {
            a: 0xAB,
            b: true,
            c: 0xFF345A7B,
        },
        TestPacket::Struct(TestPacketStruct {
            a: 0xA0,
            b: false,
            c: 0xBA34567A,
        }),
        TestPacket::NestedStruct(TestPacketNestedStruct {
            a: 0x12,
            b: true,
            c: TestPacketStruct {
                a: 0x34,
                b: false,
                c: 0x12345678,
            },
        }),
    ];

    pub fn test_packet_serdes<T>(packet: &T)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + Eq,
    {
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        let dummy_host_addr = TestNetworkAddress::EngineController(250);
        let dummy_to_addr = TestNetworkAddress::EngineController(251);
        let dummy_counter = 0xA4B5;

        let size = serialize_packet(
            packet,
            dummy_host_addr,
            dummy_to_addr,
            dummy_counter,
            &mut buffer,
        )
        .unwrap();
        let metadata: PacketMetadata<TestNetworkAddress> =
            deserialize_metadata(&mut buffer).unwrap();
        let recv_packet: T = deserialize_packet(&mut buffer).unwrap();

        assert!(size < WORKING_BUFFER_SIZE);
        assert!(recv_packet == *packet);
        assert_eq!(metadata.from_addr, dummy_host_addr);
        assert_eq!(metadata.to_addr, dummy_to_addr);
    }

    #[test]
    fn test_packet_reserialization() {
        let host_addr = TestNetworkAddress::EngineController(250);
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];
        let mut counter = 0;

        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            for packet in &TEST_PACKET_DEFAULTS {
                // println!("Testing packet: {:?} to address: {:?}", packet, address);

                let size =
                    serialize_packet(packet, host_addr, *address, counter, &mut buffer).unwrap();
                let metadata: PacketMetadata<TestNetworkAddress> =
                    deserialize_metadata(&mut buffer).unwrap();
                let recv_packet: TestPacket = deserialize_packet(&mut buffer).unwrap();

                assert!(size < WORKING_BUFFER_SIZE);
                assert_eq!(recv_packet, *packet);
                assert_eq!(metadata.from_addr, host_addr);
                assert_eq!(metadata.to_addr, *address);
                assert_eq!(metadata.counter, counter);

                counter += 1;
            }
        }
    }
}
