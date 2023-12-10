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

pub trait Broadcastable {
    fn is_broadcast(&self) -> bool;
}

pub struct BigBrother<'a, const NETWORK_MAP_SIZE: usize, P, A> {
    pub(crate) network_map: NetworkMap<A, NETWORK_MAP_SIZE>,
    pub(crate) host_addr: A,
    pub(crate) working_buffer: [u8; WORKING_BUFFER_SIZE],
    pub interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    missed_packets: u32,
    _packet_type: core::marker::PhantomData<P>,
}

impl<'a, 'b, const NETWORK_MAP_SIZE: usize, P, A> BigBrother<'a, NETWORK_MAP_SIZE, P, A>
where
    P: Serialize + for<'de> Deserialize<'de>,
    A: Copy + PartialEq + Eq + Broadcastable + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(
        host_addr: A,
        interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    ) -> Self {
        Self {
            network_map: NetworkMap::new(),
            host_addr,
            working_buffer: [0_u8; WORKING_BUFFER_SIZE],
            interfaces,
            missed_packets: 0,
            _packet_type: core::marker::PhantomData,
        }
    }

    pub fn send_packet(&mut self, packet: &P, destination: A) -> Result<(), BigBrotherError> {
        if destination.is_broadcast() {
            let size = serialize_packet(
                packet,
                self.host_addr,
                destination,
                0,
                &mut self.working_buffer,
            )?;

            let destination_endpoint = BigBrotherEndpoint {
                ip: [255, 255, 255, 255],
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
                packet,
                self.host_addr,
                destination,
                mapping.to_counter,
                &mut self.working_buffer,
            )?;
            mapping.to_counter = mapping.to_counter.wrapping_add(1);

            let destination_endpoint = BigBrotherEndpoint {
                ip: mapping.ip,
                port: UDP_PORT,
            };

            if let Some(interface) = self.interfaces[mapping.interface_index as usize].as_mut() {
                interface.send_udp(destination_endpoint, &mut self.working_buffer[..size])
            } else {
                Err(BigBrotherError::SendUnnaddressable)
            }
        }
    }

    pub fn recv_packet(&mut self) -> Result<Option<(P, A)>, BigBrotherError> {
        loop {
            if let Some((_size, source_interface_index, remote)) = self.recv_next_udp()? {
                let metadata = deserialize_metadata(&mut self.working_buffer)?;

                let mapping = self.network_map.map_network_address(
                    metadata.from_addr,
                    remote.ip,
                    source_interface_index,
                )?;

                if !metadata.to_addr.is_broadcast() {
                    if metadata.to_addr != self.host_addr {
                        continue;
                    }

                    let wrapped = metadata.counter < u16::MAX / 2
                        && mapping.from_counter > u16::MAX / 2
                        && metadata.counter > mapping.from_counter;

                    if metadata.counter < mapping.from_counter && !wrapped {
                        continue;
                    }

                    if metadata.counter > mapping.from_counter {
                        self.missed_packets +=
                            metadata.counter.wrapping_sub(mapping.from_counter) as u32;
                    }

                    mapping.from_counter = metadata.counter.wrapping_add(1);
                }

                self.forward_udp(source_interface_index, metadata.to_addr)?;

                if metadata.to_addr == self.host_addr || metadata.to_addr.is_broadcast() {
                    let packet = deserialize_packet(&mut self.working_buffer)?;
                    return Ok(Some((packet, metadata.from_addr)));
                }
            } else {
                break;
            }
        }

        Ok(None)
    }

    pub fn poll(&mut self, timestamp: u32) {
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
                port: UDP_PORT,
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

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestPacket {
        Heartbeat,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestNetworkAddress {
        Broadcast,
        A,
        B,
        C,
    }

    #[test]
    fn test_broadcast() {
        let mut interface0 = MockInterface::new();
        let mut interface1 = MockInterface::new();
        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), Some(&mut interface1)];
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            TestNetworkAddress::A,
            interfaces,
        );

        bb.send_packet(&TestPacket::Heartbeat, TestNetworkAddress::Broadcast)
            .expect("Failed to send packet");
        bb.poll(0);

        assert_eq!(interface0.sent_packets.len(), 1);
        assert_eq!(interface0.sent_packets[0].0.ip, [255, 255, 255, 255]);
        assert_eq!(interface0.sent_packets[0].0.port, UDP_PORT);

        assert_eq!(interface1.sent_packets.len(), 1);
        assert_eq!(interface1.sent_packets[0].0.ip, [255, 255, 255, 255]);
        assert_eq!(interface1.sent_packets[0].0.port, UDP_PORT);
    }

    #[test]
    fn test_recv_packet() {
        let mut interface0 = MockInterface::new();
        let mut buffer = [0_u8; WORKING_BUFFER_SIZE];

        for i in 0..16 {
            let size = serialize_packet(
                &TestPacket::Heartbeat,
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
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
                &TestPacket::Heartbeat,
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
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
            &TestPacket::Heartbeat,
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
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
                &TestPacket::Heartbeat,
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            TestNetworkAddress::B,
            interfaces,
        );

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
                &TestPacket::Heartbeat,
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
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
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            TestNetworkAddress::B,
            interfaces,
        );

        let _ = bb
            .network_map
            .map_network_address(TestNetworkAddress::A, [1, 2, 3, 4], 0)
            .unwrap();

        for _ in 0..16 {
            bb.send_packet(&TestPacket::Heartbeat, TestNetworkAddress::A)
                .expect("Failed to send packet");
            bb.poll(0);
        }

        for i in 0..16 {
            let metadata: PacketMetadata<TestNetworkAddress> =
                deserialize_metadata(&mut interface0.sent_packets[i].1[..])
                    .expect("Failed to deserialize metadata");
            let packet: TestPacket = deserialize_packet(&mut interface0.sent_packets[i].1[..])
                .expect("Failed to deserialize packet");

            assert_eq!(metadata.from_addr, TestNetworkAddress::B);
            assert_eq!(metadata.to_addr, TestNetworkAddress::A);
            assert_eq!(metadata.counter, i as u16);
            assert_eq!(packet, TestPacket::Heartbeat);
        }
    }

    #[test]
    fn test_send_counter_wrap() {
        let mut interface0 = MockInterface::new();

        let interfaces: [Option<&mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] =
            [Some(&mut interface0), None];
        let mut bb = BigBrother::<64, TestPacket, TestNetworkAddress>::new(
            TestNetworkAddress::B,
            interfaces,
        );

        let mapping = bb
            .network_map
            .map_network_address(TestNetworkAddress::A, [1, 2, 3, 4], 0)
            .unwrap();
        mapping.to_counter = u16::MAX - 8;

        for _ in 0..16 {
            bb.send_packet(&TestPacket::Heartbeat, TestNetworkAddress::A)
                .expect("Failed to send packet");
            bb.poll(0);
        }

        for i in 0..16 {
            let metadata: PacketMetadata<TestNetworkAddress> =
                deserialize_metadata(&mut interface0.sent_packets[i].1[..])
                    .expect("Failed to deserialize metadata");
            let packet: TestPacket = deserialize_packet(&mut interface0.sent_packets[i].1[..])
                .expect("Failed to deserialize packet");

            assert_eq!(metadata.from_addr, TestNetworkAddress::B);
            assert_eq!(metadata.to_addr, TestNetworkAddress::A);
            assert_eq!(metadata.counter, (u16::MAX - 8).wrapping_add(i as u16));
            assert_eq!(packet, TestPacket::Heartbeat);
        }
    }

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
