use serde::{Deserialize, Serialize};

use crate::{
    interface::BigBrotherInterface,
    network_map::NetworkMap,
    serdes::{deserialize_metadata, deserialize_packet, serialize_packet, SerdesError},
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
pub(crate) enum BigBrotherPacket<A> {
    MetaPacket(BigBrotherMetapacket),
    UserPacket(A),
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
    pub(crate) broadcast_ip: [u8; 4],
    broadcast_address: A,
    session_id: u32,
    use_dedupe: bool,
    missed_packets: u32,
    last_heartbeat_timestamp: u32,
    _packet_type: core::marker::PhantomData<P>,
}

impl<'a, 'b, const NETWORK_MAP_SIZE: usize, P, A> BigBrother<'a, NETWORK_MAP_SIZE, P, A>
where
    P: Serialize + for<'de> Deserialize<'de>,
    A: Copy + PartialEq + Eq + Broadcastable + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(
        host_addr: A,
        session_id: u32,
        broadcast_ip: [u8; 4],
        broadcast_address: A,
        interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    ) -> Self {
        let mut bb = Self {
            network_map: NetworkMap::new(),
            host_addr,
            broadcast_ip,
            broadcast_address,
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
            if let Some((_size, source_interface_index, remote)) = self.recv_next_udp()? {
                let metadata = deserialize_metadata(&mut self.working_buffer)?;

                let mapping = self.network_map.map_network_address(
                    metadata.from_addr,
                    remote.ip,
                    remote.port,
                    source_interface_index,
                )?;

                let mut is_duplicate = false;

                if self.use_dedupe && !metadata.to_addr.is_broadcast() {
                    let wrapped = metadata.counter < u16::MAX / 2
                        && mapping.from_counter > u16::MAX / 2
                        && metadata.counter > mapping.from_counter;

                    if metadata.counter < mapping.from_counter && !wrapped {
                        is_duplicate = true;
                    } else {
                        self.missed_packets +=
                            metadata.counter.wrapping_sub(mapping.from_counter) as u32;

                        mapping.from_counter = metadata.counter.wrapping_add(1);
                    }
                }

                self.forward_udp(source_interface_index, metadata.to_addr)?;

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
                            if !is_duplicate {
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
        if timestamp - self.last_heartbeat_timestamp > 100 {
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
                0,
                &mut self.working_buffer,
            )?;

            let destination_endpoint = BigBrotherEndpoint {
                ip: self.broadcast_ip,
                port: UDP_PORT,
            };

            for interface in &mut self.interfaces {
                if let Some(interface) = interface {
                    interface.send_udp(
                        destination_endpoint.clone(),
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

    fn forward_udp(
        &mut self,
        source_interface_index: u8,
        destination: A,
    ) -> Result<(), BigBrotherError> {
        if destination == self.host_addr {
            return Ok(());
        }

        if destination.is_broadcast() {
            for (interface_index, interface) in self.interfaces.iter_mut().enumerate() {
                if let Some(interface) = interface {
                    if interface_index == source_interface_index as usize {
                        continue;
                    }

                    let destination_endpoint = BigBrotherEndpoint {
                        ip: [255, 255, 255, 255],
                        port: UDP_PORT,
                    };

                    interface
                        .send_udp(destination_endpoint.clone(), &mut self.working_buffer[..])?;
                }
            }
        } else if let Ok(network_mapping) = self.network_map.get_address_mapping(destination) {
            let destination_endpoint = BigBrotherEndpoint {
                ip: network_mapping.ip,
                port: network_mapping.port,
            };

            if let Some(interface) =
                self.interfaces[network_mapping.interface_index as usize].as_mut()
            {
                interface.send_udp(destination_endpoint, &mut self.working_buffer[..])?;
            } else {
                return Err(BigBrotherError::SendUnnaddressable);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::serdes::PacketMetadata;
    use rand::{self, Rng};

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
        let session_id: u32 = rand::thread_rng().gen();

        BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            host_addr,
            session_id,
            [255, 255, 255, 255],
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
    fn test_broadcast() {
        let mut interface0 = MockInterface::new();
        let mut interface1 = MockInterface::new();
        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), Some(&mut interface1)];
        let mut bb = create_big_brother(
            TestNetworkAddress::A,
            interfaces,
        );

        bb.send_packet(&TestPacket::Heartbeat, TestNetworkAddress::Broadcast)
            .expect("Failed to send packet");
        bb.poll_1ms(0);

        assert_eq!(interface0.sent_packets.len(), 2);
        assert_eq!(interface0.sent_packets[1].0.ip, [255, 255, 255, 255]);
        assert_eq!(interface0.sent_packets[1].0.port, UDP_PORT);

        assert_eq!(interface1.sent_packets.len(), 2);
        assert_eq!(interface1.sent_packets[1].0.ip, [255, 255, 255, 255]);
        assert_eq!(interface1.sent_packets[1].0.port, UDP_PORT);
    }

    #[test]
    fn test_recv_packet() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        for i in 0..16 {
            let size = serialize_packet(
                &BigBrotherPacket::UserPacket(TestPacket::Heartbeat),
                TestNetworkAddress::A,
                TestNetworkAddress::B,
                i,
                &mut buffer,
            )
            .expect("Failed to serialize packet");

            interface0.recv_packets.push((
                BigBrotherEndpoint {
                    ip: [1, 2, 3, 4],
                    port: UDP_PORT,
                },
                buffer[..size].to_vec(),
            ));
        }

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        for _ in 0..16 {
            let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();

            assert_eq!(packet.0, TestPacket::Heartbeat);
            assert_eq!(packet.1, TestNetworkAddress::A);
        }
    }

    #[test]
    fn test_no_recv_incorrect_destination() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];
        let size = serialize_packet(
            &TestPacket::Heartbeat,
            TestNetworkAddress::A,
            TestNetworkAddress::B,
            0,
            &mut buffer,
        )
        .expect("Failed to serialize packet");

        interface0.recv_packets.push((
            BigBrotherEndpoint {
                ip: [1, 2, 3, 4],
                port: UDP_PORT,
            },
            buffer[..size].to_vec(),
        ));

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::C,
            interfaces,
        );

        assert!(bb.recv_packet().unwrap().is_none());
    }

    #[test]
    fn test_recv_broadcast() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        // Ensure the counter doesn't matter because the dst is broadcast
        for _ in 0..16 {
            let size = serialize_packet(
                &BigBrotherPacket::UserPacket(TestPacket::Heartbeat),
                TestNetworkAddress::A,
                TestNetworkAddress::Broadcast,
                0,
                &mut buffer,
            )
            .expect("Failed to serialize packet");

            interface0.recv_packets.push((
                BigBrotherEndpoint {
                    ip: [1, 2, 3, 4],
                    port: UDP_PORT,
                },
                buffer[..size].to_vec(),
            ));
        }

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        for _ in 0..16 {
            let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();

            assert_eq!(packet.0, TestPacket::Heartbeat);
            assert_eq!(packet.1, TestNetworkAddress::A);
        }
    }

    #[test]
    fn test_recv_on_second_interface() {
        let mut interface0 = MockInterface::new();
        let mut interface1 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];
        let size = serialize_packet(
            &BigBrotherPacket::UserPacket(TestPacket::Heartbeat),
            TestNetworkAddress::A,
            TestNetworkAddress::Broadcast,
            0,
            &mut buffer,
        )
        .expect("Failed to serialize packet");

        interface1.recv_packets.push((
            BigBrotherEndpoint {
                ip: [1, 2, 3, 4],
                port: UDP_PORT,
            },
            buffer[..size].to_vec(),
        ));

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), Some(&mut interface1)];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();

        assert_eq!(packet.0, TestPacket::Heartbeat);
        assert_eq!(packet.1, TestNetworkAddress::A);
    }

    #[test]
    fn test_recv_dedupe() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        for _ in 0..16 {
            let size = serialize_packet(
                &BigBrotherPacket::UserPacket(TestPacket::Heartbeat),
                TestNetworkAddress::A,
                TestNetworkAddress::B,
                4,
                &mut buffer,
            )
            .expect("Failed to serialize packet");

            interface0.recv_packets.push((
                BigBrotherEndpoint {
                    ip: [1, 2, 3, 4],
                    port: UDP_PORT,
                },
                buffer[..size].to_vec(),
            ));
        }

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );
        bb.use_dedupe = true;

        for i in 0..16 {
            if i == 0 {
                let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();
                assert_eq!(packet.0, TestPacket::Heartbeat);
                assert_eq!(packet.1, TestNetworkAddress::A);
            } else {
                assert!(bb.recv_packet().unwrap().is_none());
            }
        }
    }

    #[test]
    fn test_recv_counter_wrap() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        let counter_start = u16::MAX - 16;

        for i in 0..16 {
            let size = serialize_packet(
                &BigBrotherPacket::UserPacket(TestPacket::Heartbeat),
                TestNetworkAddress::A,
                TestNetworkAddress::B,
                counter_start.wrapping_add(i * 2),
                &mut buffer,
            )
            .expect("Failed to serialize packet");

            interface0.recv_packets.push((
                BigBrotherEndpoint {
                    ip: [1, 2, 3, 4],
                    port: UDP_PORT,
                },
                buffer[..size].to_vec(),
            ));
        }

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        for _ in 0..16 {
            let packet = bb.recv_packet().expect("Failed to receive packet").unwrap();
            assert_eq!(packet.0, TestPacket::Heartbeat);
            assert_eq!(packet.1, TestNetworkAddress::A);
        }
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
            .map_network_address(TestNetworkAddress::A, [1, 2, 3, 4], UDP_PORT, 0)
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
            assert_eq!(metadata.counter, i as u16);

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

    #[test]
    fn test_send_counter_wrap() {
        let mut interface0 = MockInterface::new();

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = create_big_brother(
            TestNetworkAddress::B,
            interfaces,
        );

        let mapping = bb
            .network_map
            .map_network_address(TestNetworkAddress::A, [1, 2, 3, 4], UDP_PORT, 0)
            .unwrap();
        mapping.to_counter = u16::MAX - 8;

        for _ in 0..16 {
            bb.send_packet(&TestPacket::Heartbeat, TestNetworkAddress::A)
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
            assert_eq!(metadata.counter, (u16::MAX - 8).wrapping_add(i as u16));
            match packet {
                BigBrotherPacket::MetaPacket(_) => {
                    panic!("Received metapacket");
                }
                BigBrotherPacket::UserPacket(packet) => {
                    assert_eq!(packet, TestPacket::Heartbeat);
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
