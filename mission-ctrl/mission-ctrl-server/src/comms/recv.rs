use std::{net::{UdpSocket, Ipv4Addr, IpAddr}, time::Duration, sync::{Arc, RwLock}, io::ErrorKind};
use comms_manager::CommsManager;

use crate::{observer::{ObserverEvent, ObserverHandler}, process_is_running};

use super::{RECV_PORT, NETWORK_MAP_SIZE};

const BUFFER_SIZE: usize = 1024;

struct RecievingThread {
    observer_handler: Arc<ObserverHandler>,
    comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>,
}

impl RecievingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>, comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>) -> Self {
        Self {
            observer_handler,
            comms_manager,
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
                    let source_address = Self::ipv4_from_ip(saddress.ip()).octets();
                    let packet = self.comms_mut().extract_packet(&mut buffer[0..size], source_address);

                    match packet {
                        Ok((packet, address)) => {
                            self.observer_handler.notify(ObserverEvent::PacketReceived {
                                packet,
                                ip: source_address,
                                address,
                            });
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

    // fn comms(&self) -> std::sync::RwLockReadGuard<'_, CommsManager<NETWORK_MAP_SIZE>> {
    //     self.comms_manager.as_ref().read().unwrap()
    // }

    fn comms_mut(&mut self) -> std::sync::RwLockWriteGuard<'_, CommsManager<NETWORK_MAP_SIZE>> {
        self.comms_manager.as_ref().write().unwrap()
    }

    fn ipv4_from_ip(ip: IpAddr) -> Ipv4Addr {
        match ip {
            IpAddr::V4(ipv4) => ipv4,
            IpAddr::V6(ipv6) => ipv6.to_ipv4().expect("recv_thread: Failed to convert IPv6 address to IPv4"),
        }
    }
}

pub fn recv_thread(observer_handler: Arc<ObserverHandler>, comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>) {
    observer_handler.register_observer_thread();
    RecievingThread::new(observer_handler, comms_manager).run();
}
