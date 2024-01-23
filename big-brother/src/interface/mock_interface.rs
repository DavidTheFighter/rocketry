use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::dedupe;
use crate::{
    big_brother::{
        BigBrotherEndpoint, BigBrotherError, BigBrotherPacket, UDP_PORT, WORKING_BUFFER_SIZE,
    },
    serdes,
};

use super::{mock_topology::MockPhysicalInterface, BigBrotherInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPayload {
    pub host: BigBrotherEndpoint,   // To
    pub remote: BigBrotherEndpoint, // From
    pub data: Vec<u8>,
}

pub struct MockInterface {
    pub sent_packets: Vec<MockPayload>,
    pub received_packets: Vec<MockPayload>,
    pub host_ip: [u8; 4],
    pub host_port: u16,
    pending_recv_packets: Option<VecDeque<MockPayload>>,
    physical_interface: Option<Arc<Mutex<MockPhysicalInterface>>>,
}

impl MockInterface {
    pub fn new() -> Self {
        let dummy_ip = [192, 168, 0, 1];
        let dummy_port = UDP_PORT;

        MockInterface {
            sent_packets: Vec::new(),
            received_packets: Vec::new(),
            host_ip: dummy_ip,
            host_port: dummy_port,
            pending_recv_packets: Some(VecDeque::new()),
            physical_interface: None,
        }
    }

    pub fn new_networked(physical_interface: Arc<Mutex<MockPhysicalInterface>>) -> Self {
        let host = physical_interface
            .lock()
            .expect("Failed to lock physical interface for virtual interface init")
            .register_virtual_interface();

        Self {
            sent_packets: Vec::new(),
            received_packets: Vec::new(),
            host_ip: host.ip,
            host_port: host.port,
            pending_recv_packets: None,
            physical_interface: Some(physical_interface),
        }
    }

    pub fn add_recv_packet<A, P>(
        &mut self,
        to_addr: A,
        from_addr: A,
        remote_ip: [u8; 4],
        remote_port: u16,
        counter: dedupe::CounterType,
        packet: &P,
    ) -> Result<(), BigBrotherError>
    where
        P: Serialize,
        A: Serialize,
    {
        let mut buffer = [0u8; WORKING_BUFFER_SIZE];
        let packet = BigBrotherPacket::UserPacket(packet);
        let size = serdes::serialize_packet(&packet, from_addr, to_addr, counter, &mut buffer)?;
        let payload = MockPayload {
            host: BigBrotherEndpoint {
                ip: self.host_ip,
                port: self.host_port,
            },
            remote: BigBrotherEndpoint {
                ip: remote_ip,
                port: remote_port,
            },
            data: buffer[..size].to_vec(),
        };

        self.add_recv_payload(payload);

        Ok(())
    }

    pub fn add_recv_payload(&mut self, payload: MockPayload) {
        if self.physical_interface.is_some() {
            panic!("When using a mocked network topology, you should not add packets to the virtual interface directly. Instead, use another interface to send packets to this interface.");
        } else if let Some(pending_packets) = &mut self.pending_recv_packets {
            pending_packets.push_back(payload);
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
        let payload = MockPayload {
            host: destination,
            remote: BigBrotherEndpoint {
                ip: self.host_ip,
                port: self.host_port,
            },
            data: data.to_vec(),
        };

        self.sent_packets.push(payload.clone());

        if let Some(physical_interface) = &mut self.physical_interface {
            physical_interface
                .lock()
                .expect("Failed to unlock physical interface to send UDP")
                .send_udp(payload);
        }

        Ok(())
    }

    fn recv_udp(
        &mut self,
        data: &mut [u8],
    ) -> Result<Option<(usize, BigBrotherEndpoint)>, BigBrotherError> {
        if let Some(phy) = &mut self.physical_interface {
            let payload = phy
                .lock()
                .expect("Failed to unlock physical interface for virtual recv_udp")
                .recv_udp(self.host_port);

            if let Some(payload) = payload {
                data[..payload.data.len()].copy_from_slice(payload.data.as_slice());
                return Ok(Some((payload.data.len(), payload.remote)));
            }
        } else if let Some(pending_packets) = &mut self.pending_recv_packets {
            if let Some(payload) = pending_packets.pop_front() {
                data[..payload.data.len()].copy_from_slice(payload.data.as_slice());
                return Ok(Some((payload.data.len(), payload.remote)));
            }
        }

        Ok(None)
    }

    fn broadcast_ip(&self) -> [u8; 4] {
        if let Some(phy) = &self.physical_interface {
            phy.lock()
                .expect("Failed to lock physical interface for virtual interface")
                .broadcast_ip()
        } else {
            [255, 255, 255, 255]
        }
    }

    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}
