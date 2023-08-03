use std::{net::{UdpSocket, Ipv4Addr}, time::Duration, sync::Arc, io::ErrorKind};
use hal::comms_hal::{Packet, NetworkAddress};

use crate::{observer::{ObserverEvent, ObserverHandler}, process_is_running};

use super::{addresses::AddressManager, RECV_PORT};

const BUFFER_SIZE: usize = 1024;

struct RecievingThread {
    observer_handler: Arc<ObserverHandler>,
    address_manager: Arc<AddressManager>,
}

impl RecievingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>, address_manager: Arc<AddressManager>) -> Self {
        Self {
            observer_handler,
            address_manager,
        }
    }

    pub fn run(&mut self) {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", RECV_PORT))
            .expect("recv_thread: Failed to bind socket");
        let mut buffer = [0_u8; BUFFER_SIZE];

        socket
            .set_read_timeout(Some(Duration::from_millis(10)))
            .expect("recv_thread: Failed to set socket timeout");

        println!("recv_thread: Listening on port {}", RECV_PORT);

        while process_is_running() {
            match socket.recv_from(&mut buffer) {
                Ok((size, saddress)) => {
                    let packet = Packet::deserialize(&mut buffer[0..size]);
                    let address = self.address_manager.ip_to_network_address(saddress.ip());

                    match packet {
                        Ok(packet) => {
                            self.handle_packet(packet, address);
                        }
                        Err(err) => {
                            println!("recv_thread: Packet deserialization error: {:?} ({} bytes: {:?})",
                                err,
                                size,
                                &buffer[0..size],
                            );
                        }
                    }
                },
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => {}
                    _ => eprintln!("recv_thread: socket.recv_from(..) error: {:?}", err),
                },
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet, address: Option<NetworkAddress>) {
        let mut address = address.unwrap_or(NetworkAddress::Unknown);

        if let Packet::ComponentIpAddress { addr, ip } = packet {
            self.address_manager.map_ip_address(addr, Ipv4Addr::from(ip));
            address = addr;
        }

        self.observer_handler.notify(ObserverEvent::PacketReceived {
            packet,
            address,
        });
    }
}

pub fn recv_thread(observer_handler: Arc<ObserverHandler>, address_manager: Arc<AddressManager>) {
    observer_handler.register_observer_thread();
    RecievingThread::new(observer_handler, address_manager).run();
}
