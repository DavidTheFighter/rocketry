use serde::Serialize;

use crate::{big_brother::{BigBrotherEndpoint, BigBrotherError, WORKING_BUFFER_SIZE, UDP_PORT, BigBrotherPacket}, serdes};

use super::BigBrotherInterface;


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

    pub fn add_recv_packet<A, P>(
        &mut self,
        to_addr: A,
        from_addr: A,
        host_ip: [u8; 4],
        counter: u16,
        packet: &P,
    ) -> Result<(), BigBrotherError>
    where
        P: Serialize,
        A: Serialize,
    {
        let mut buffer = [0u8; WORKING_BUFFER_SIZE];
        let packet = BigBrotherPacket::UserPacket(packet);
        let size = serdes::serialize_packet(&packet, from_addr, to_addr, counter, &mut buffer)?;
        self.recv_packets.push((
            BigBrotherEndpoint {
                ip: host_ip,
                port: UDP_PORT,
            },
            buffer[..size].to_vec(),
        ));

        Ok(())
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
        if self.recv_packets.len() == 0 {
            return Ok(None);
        }

        let (endpoint, packet) = self.recv_packets.remove(0);
        let size = packet.len();
        data[..size].copy_from_slice(&packet);
        Ok(Some((size, endpoint)))
    }

    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}
