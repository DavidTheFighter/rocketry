pub mod connection;

use std::{sync::Arc, time::Duration, net::Ipv4Addr};

use hal::comms_hal::{Packet, NetworkAddress};

use crate::{observer::{ObserverHandler, ObserverEvent}, process_is_running, timestamp};

use self::connection::CameraConnection;

pub const CAMERA_CONNECTION_TIMEOUT: f64 = 5.0;
pub const CAMERA_CONNECTION_PORT_START: u16 = 5000;
pub const TRANSCODE_PORT_START: u16 = 5500;

pub struct CameraStreaming {
    observer_handler: Arc<ObserverHandler>,
    active_connections: Vec<CameraConnection>,
    connection_port_counter: u16,
    transcode_port_counter: u16,
}

impl CameraStreaming {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            active_connections: Vec::new(),
            connection_port_counter: CAMERA_CONNECTION_PORT_START,
            transcode_port_counter: TRANSCODE_PORT_START,
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                self.handle_packet(packet);
            }

            self.active_connections.retain_mut(|connection| {
                if timestamp() - connection.last_ping > CAMERA_CONNECTION_TIMEOUT {
                    print!("Dropping camera connection: {:?}...", connection.address);
                    connection.drop_connection();
                    println!(" done");
                    return false;
                }

                return true;
            });
        }

        for connection in &mut self.active_connections {
            print!("Dropping camera connection: {:?}...", connection.address);
            connection.drop_connection();
            println!(" done");
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet {
            Packet::ComponentIpAddress { addr, ip } => {
                self.handle_ping(addr, Ipv4Addr::from(ip));
            },
            _ => {}
        }
    }

    fn handle_ping(&mut self, address: NetworkAddress, connection_ip: Ipv4Addr) {
        let mut found = false;
        for connection in &mut self.active_connections {
            if connection.address == address {
                found = true;
                connection.ping();
                break;
            }
        }

        if !found {
            let connection_port = self.connection_port_counter;
            let transcode_port = self.transcode_port_counter;
            match CameraConnection::new(
                address,
                connection_ip,
                connection_port,
                transcode_port,
                self.observer_handler.clone(),
            ) {
                Some(connection) => {
                    self.active_connections.push(connection);
                },
                None => {
                    println!("Failed to start transcoding process for camera: {:?}", address);
                    return;
                }
            }

            println!("New camera connection: {:?} @ {:?}:{}, transcoding on {}",
                address,
                connection_ip,
                connection_port,
                transcode_port,
            );

            self.connection_port_counter += 1;
            self.transcode_port_counter += 1;
        }
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