use std::{net::UdpSocket, sync::Arc, time::Duration};
use hal::comms_hal::{Packet, NetworkAddress};

use crate::{observer::{ObserverEvent, ObserverHandler}, process_is_running};

use super::{addresses::AddressManager, SEND_PORT};

const BUFFER_SIZE: usize = 1024;

struct SendingThread {
    observer_handler: Arc<ObserverHandler>,
    address_manager: Arc<AddressManager>,
}

impl SendingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>, address_manager: Arc<AddressManager>) -> Self {
        Self {
            observer_handler,
            address_manager,
        }
    }

    pub fn run(&self) {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", SEND_PORT))
            .expect("send_thread: Failed to bind socket");
        let mut buffer = [0_u8; BUFFER_SIZE];

        while process_is_running() {
            if let Some((event_id, addr, packet)) = self.receive_packet_event() {
                match packet.serialize(&mut buffer) {
                    Ok(size) => {
                        let ip_address = self.address_manager.network_address_to_ip(addr);

                        if let Some(ip_address) = ip_address {
                            let address = format!("{}:{}", ip_address, SEND_PORT);

                            if let Err(err) = socket.send_to(&buffer[0..size], address) {
                                self.send_packet_resonse(
                                    event_id,
                                    Err(format!("send_thread: Failed to send packet: {err}")),
                                );
                            } else {
                                self.send_packet_resonse(event_id, Ok(()));
                            }
                        } else {
                            self.send_packet_resonse(
                                event_id,
                                Err(format!("send_thread: Failed to map network addr to ip")),
                            );
                        }
                    }
                    Err(err) => {
                        self.send_packet_resonse(
                            event_id,
                            Err(format!("send_thread: Failed to serialize packet: {:?}", err)),
                        );
                    }
                }
            }
        }
    }

    fn send_packet_resonse(&self, event_id: u64, result: Result<(), String>) {
        if let Err(err) = &result {
            eprintln!("{err}");
        }

        self.observer_handler.notify(ObserverEvent::EventResponse(
            event_id,
            result,
        ));
    }

    fn receive_packet_event(&self) -> Option<(u64, NetworkAddress, Packet)> {
        let timeout = Duration::from_millis(10);

        if let Some((event_id, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::SendPacket { address, packet } = event {
                return Some((event_id, address, packet));
            }
        }

        None
    }
}

pub fn send_thread(observer_handler: Arc<ObserverHandler>, address_manager: Arc<AddressManager>) {
    observer_handler.register_observer_thread();
    SendingThread::new(observer_handler, address_manager).run();
}
