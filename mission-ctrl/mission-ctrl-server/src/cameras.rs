pub mod connection;

use std::{sync::Arc, time::Duration};

use hal::comms_hal::{Packet, NetworkAddress};

use crate::{observer::{ObserverHandler, ObserverEvent}, process_is_running};

use self::connection::CameraConnection;

pub const CAMERA_CONNECTION_TIMEOUT: f64 = 5.0;

pub struct CameraStreaming {
    observer_handler: Arc<ObserverHandler>,
    active_connections: Vec<CameraConnection>,
}

impl CameraStreaming {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            active_connections: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                self.handle_packet(packet);
            }

            // self.active_connections.iter_mut().filter(|connection| {
            //     if timestamp() - connection.last_ping > CAMERA_CONNECTION_TIMEOUT {
            //         return false;
            //     }

            //     return true;
            // });
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet {
            Packet::ComponentIpAddress { addr, ip: _ } => {
                self.handle_ping(addr);
            },
            _ => {}
        }
    }

    fn handle_ping(&mut self, address: NetworkAddress) {
        let mut found = false;
        for connection in &mut self.active_connections {
            if connection.address == address {
                found = true;
                connection.ping();
                break;
            }
        }

        if !found {
            let new_connection = self.create_connection(address);
            self.active_connections.push(new_connection);
        }
    }

    fn create_connection(&mut self, address: NetworkAddress) -> CameraConnection {
        let mut connection = CameraConnection::new(address);

        connection
    }

    fn get_packet(&self) -> Option<Packet> {
        let timeout = Duration::from_millis(10);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::PacketReceived { address: _, packet } = event {
                return Some(packet);
            }
        }

        None
    }
}

pub fn camera_streaming_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();

    CameraStreaming::new(observer_handler).run();
}