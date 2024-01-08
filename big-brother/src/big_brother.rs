use serde::{Deserialize, Serialize};

use crate::{
    interface::BigBrotherInterface,
    network_map::NetworkMap,
    serdes::{deserialize_metadata, deserialize_packet, serialize_packet, SerdesError}, dedupe::{is_duplicate, self},
};

pub const UDP_PORT: u16 = 25560;
pub const MAX_INTERFACE_COUNT: usize = 2;
pub const WORKING_BUFFER_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub enum BigBrotherError {
    SerializationError(SerdesError),
    SmoltcpSendBufferFull,
    SmoltcpRecvExhausted,
    AccidentalIpv6,
    UnknownNetworkAddress,
    NetworkMapFull,
    SendUnnaddressable,
    SocketBindFailure,
    SocketConfigFailure,
    SendFailure,
}

#[derive(Debug, Clone)]
pub struct BigBrotherEndpoint {
    pub ip: [u8; 4],
    pub port: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum BigBrotherPacket<T> {
    MetaPacket(BigBrotherMetapacket),
    UserPacket(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum BigBrotherMetapacket {
    Heartbeat {
        session_id: u32,
    },
}

pub trait Broadcastable {
    fn is_broadcast(&self) -> bool;
}

pub struct BigBrother<'a, const NETWORK_MAP_SIZE: usize, P, A> {
    pub(crate) network_map: NetworkMap<A, NETWORK_MAP_SIZE>,
    pub(crate) host_addr: A,
    pub(crate) working_buffer: [u8; WORKING_BUFFER_SIZE],
    pub interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    broadcast_address: A,
    broadcast_counter: dedupe::CounterType,
    session_id: u32,
    use_dedupe: bool,
    missed_packets: u32,
    last_heartbeat_timestamp: u32,
    _packet_type: core::marker::PhantomData<P>,
}

impl<'a, 'b, const NETWORK_MAP_SIZE: usize, P, A> BigBrother<'a, NETWORK_MAP_SIZE, P, A>
where
    P: Serialize + for<'de> Deserialize<'de>,
    A: Copy + PartialEq + Eq + Broadcastable + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
{
    pub fn new(
        host_addr: A,
        session_id: u32,
        broadcast_address: A,
        interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    ) -> Self {
        let mut bb = Self {
            network_map: NetworkMap::new(host_addr),
            host_addr,
            broadcast_address,
            broadcast_counter: 0,
            session_id,
            working_buffer: [0_u8; WORKING_BUFFER_SIZE],
            interfaces,
            use_dedupe: true,
            missed_packets: 0,
            last_heartbeat_timestamp: 0,
            _packet_type: core::marker::PhantomData,
        };

        bb.send_bb_packet(
            BigBrotherPacket::MetaPacket(BigBrotherMetapacket::Heartbeat
            {
                session_id: session_id,
            }),
            broadcast_address,
        );

        bb
    }

    pub fn send_packet(&mut self, packet: &P, destination: A) -> Result<(), BigBrotherError> {
        self.send_bb_packet(BigBrotherPacket::UserPacket(packet), destination)
    }

    pub fn recv_packet(&mut self) -> Result<Option<(P, A)>, BigBrotherError> {
        loop {
            if let Some((size, source_interface_index, remote)) = self.recv_next_udp()? {
                let metadata = deserialize_metadata(&mut self.working_buffer)?;

                let mapping = self.network_map.map_network_address(
                    metadata.from_addr,
                    remote.ip,
                    remote.port,
                    source_interface_index,
                    false,
                )?;

                let dedupe = if self.use_dedupe {
                    is_duplicate(&metadata, mapping)
                } else {
                    Ok(0)
                };

                if !metadata.to_addr.is_broadcast() || dedupe.is_ok() {
                    self.try_forward_udp(source_interface_index, &remote, metadata.to_addr, size)?;

                    // Only update the mapping if it's a valid packet and (at least for now)
                    // don't map our own network address
                    if metadata.from_addr != self.host_addr {
                        let _ = self.network_map.map_network_address(
                            metadata.from_addr,
                            remote.ip,
                            remote.port,
                            source_interface_index,
                            true,
                        )?;
                    }
                }

                if let Ok(missed_packets) = dedupe {
                    self.missed_packets += missed_packets as u32;
                }

                if metadata.to_addr == self.host_addr || metadata.to_addr.is_broadcast() {
                    let packet: BigBrotherPacket<P> = deserialize_packet(&mut self.working_buffer)?;

                    match packet {
                        BigBrotherPacket::MetaPacket(metapacket) => {
                            match metapacket {
                                BigBrotherMetapacket::Heartbeat { session_id } => {
                                    self.network_map.update_session_id(metadata.from_addr, session_id)?;
                                }
                            }
                        }
                        BigBrotherPacket::UserPacket(packet) => {
                            if dedupe.is_ok() {
                                return Ok(Some((packet, metadata.from_addr)));
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(None)
    }

    pub fn poll_1ms(&mut self, timestamp: u32) {
        if timestamp.wrapping_sub(self.last_heartbeat_timestamp) > 100 {
            self.last_heartbeat_timestamp = timestamp;

            self.send_bb_packet(
                BigBrotherPacket::MetaPacket(BigBrotherMetapacket::Heartbeat {
                    session_id: self.session_id,
                }),
                self.broadcast_address,
            );
        }

        for interface in &mut self.interfaces {
            if let Some(interface) = interface {
                interface.poll(timestamp);
            }
        }
    }

    pub fn get_missed_packets(&self) -> u32 {
        self.missed_packets
    }

    pub fn get_network_mapping(&mut self, address: A) -> Option<[u8; 4]> {
        if let Ok(mapping) = self.network_map.get_address_mapping(address) {
            Some(mapping.ip)
        } else {
            None
        }
    }

    fn send_bb_packet(&mut self, packet: BigBrotherPacket<&P>, destination: A) -> Result<(), BigBrotherError> {
        if destination.is_broadcast() {
            let size = serialize_packet(
                &packet,
                self.host_addr,
                destination,
                self.broadcast_counter,
                &mut self.working_buffer,
            )?;
            self.broadcast_counter = self.broadcast_counter.wrapping_add(1);

            for interface in &mut self.interfaces {
                if let Some(interface) = interface {
                    let destination_endpoint = BigBrotherEndpoint {
                        ip: interface.broadcast_ip(),
                        port: UDP_PORT,
                    };

                    // // print!("Broad({}->{}): ", self.broadcast_counter - 1, self.broadcast_counter);
                    interface.send_udp(
                        destination_endpoint,
                        &mut self.working_buffer[..size],
                    )?;
                }
            }

            Ok(())
        } else {
            let mapping = self.network_map.get_address_mapping(destination)?;
            let size = serialize_packet(
                &packet,
                self.host_addr,
                destination,
                mapping.to_counter,
                &mut self.working_buffer,
            )?;
            mapping.to_counter = mapping.to_counter.wrapping_add(1);

            let destination_endpoint = BigBrotherEndpoint {
                ip: mapping.ip,
                port: mapping.port,
            };

            if let Some(interface) = self.interfaces[mapping.interface_index as usize].as_mut() {
                interface.send_udp(destination_endpoint, &mut self.working_buffer[..size])
            } else {
                Err(BigBrotherError::SendUnnaddressable)
            }
        }
    }

    fn recv_next_udp(
        &mut self,
    ) -> Result<Option<(usize, u8, BigBrotherEndpoint)>, BigBrotherError> {
        for (interface_index, interface) in self.interfaces.iter_mut().enumerate() {
            if let Some(interface) = interface {
                if let Some((size, remote)) = interface.recv_udp(&mut self.working_buffer)? {
                    return Ok(Some((size, interface_index as u8, remote)));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::serdes::PacketMetadata;

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestPacket {
        Heartbeat,
        SomeData {
            a: u32,
            b: u32,
            c: bool,
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestNetworkAddress {
        Broadcast,
        A,
        B,
        C,
    }

    fn create_big_brother(host_addr: TestNetworkAddress, interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT]) -> BigBrother<64, TestPacket, TestNetworkAddress> {
        // Create a session ID from rng
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let session_id: u32 = nanos;

        BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            host_addr,
            session_id,
            TestNetworkAddress::Broadcast,
            interfaces,
        )
    }

    #[test]
    fn test_reserialize() {
        let mut interface0 = MockInterface::new();

        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            [Some(&mut interface0), None],
        );

        let comparison_packet = TestPacket::SomeData {
            a: 0xA0A1A2A3,
            b: 0xFF00FF00,
            c: true,
        };

        bb.send_packet(&comparison_packet, TestNetworkAddress::Broadcast)
            .expect("Failed to send packet");
        bb.poll_1ms(0);

        let packet_data = bb.interfaces[0].as_mut().unwrap().as_mut_any().unwrap().downcast_mut::<MockInterface>().unwrap().sent_packets[1].1.clone();
        bb.interfaces[0].as_mut().unwrap().as_mut_any().unwrap().downcast_mut::<MockInterface>().unwrap().recv_packets.push((
            BigBrotherEndpoint {
                ip: [1, 2, 3, 4],
                port: UDP_PORT,
            },
            packet_data,
        ));

        let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();

        assert_eq!(packet.0, comparison_packet);
    }

    #[test]
    fn test_send() {
        let mut interface0 = MockInterface::new();

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        let _ = bb
            .network_map
            .map_network_address(TestNetworkAddress::A, [1, 2, 3, 4], UDP_PORT, 0, true)
            .unwrap();

        let test_packet = TestPacket::SomeData {
            a: 0xA0A1A2A3,
            b: 0xFF00FF00,
            c: true,
        };

        for _ in 0..16 {
            bb.send_packet(&test_packet, TestNetworkAddress::A)
                .expect("Failed to send packet");
            bb.poll_1ms(0);
        }

        for i in 0..16 {
            let metadata: PacketMetadata<TestNetworkAddress> =
                deserialize_metadata(&mut interface0.sent_packets[1 + i].1[..])
                    .expect("Failed to deserialize metadata");
            let packet: BigBrotherPacket<TestPacket> = deserialize_packet(&mut interface0.sent_packets[1 + i].1[..])
                .expect("Failed to deserialize packet");

            assert_eq!(metadata.from_addr, TestNetworkAddress::B);
            assert_eq!(metadata.to_addr, TestNetworkAddress::A);
            assert_eq!(metadata.counter, i as dedupe::CounterType);

            match packet {
                BigBrotherPacket::MetaPacket(_) => {
                    panic!("Received metapacket");
                }
                BigBrotherPacket::UserPacket(packet) => {
                    assert_eq!(packet, test_packet);
                }
            }
        }
    }

    // #[test]
    // fn test_dedupe_new_session()

    pub struct MockInterface {
        sent_packets: Vec<(BigBrotherEndpoint, Vec<u8>)>,
        recv_packets: Vec<(BigBrotherEndpoint, Vec<u8>)>,
    }

    impl MockInterface {
        pub fn new() -> Self {
            Self {
                sent_packets: Vec::new(),
                recv_packets: Vec::new(),
            }
        }
    }

    impl BigBrotherInterface for MockInterface {
        fn poll(&mut self, _timestamp: u32) {}

        fn send_udp(
            &mut self,
            destination: BigBrotherEndpoint,
            data: &mut [u8],
        ) -> Result<(), BigBrotherError> {
            self.sent_packets.push((destination, data.to_vec()));
            Ok(())
        }

        fn recv_udp(
            &mut self,
            data: &mut [u8],
        ) -> Result<Option<(usize, BigBrotherEndpoint)>, BigBrotherError> {
            if self.recv_packets.len() > 0 {
                let (remote, packet) = self.recv_packets.remove(0);
                data[..packet.len()].copy_from_slice(&packet);
                Ok(Some((packet.len(), remote.clone())))
            } else {
                Ok(None)
            }
        }

        fn broadcast_ip(&self) -> [u8; 4] {
            [255, 255, 255, 255]
        }
        
        fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
            Some(self)
        }
    }

    impl Broadcastable for TestNetworkAddress {
        fn is_broadcast(&self) -> bool {
            match self {
                TestNetworkAddress::Broadcast => true,
                _ => false,
            }
        }
    }
}
