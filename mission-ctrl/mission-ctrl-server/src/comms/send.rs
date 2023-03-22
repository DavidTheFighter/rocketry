use std::{net::UdpSocket, sync::Arc, collections::HashMap, time::Duration};
use hal::comms_hal::{Packet, NetworkAddress};

use crate::{observer::{ObserverEvent, ObserverHandler}, process_is_running};

const BUFFER_SIZE: usize = 1024;

struct SendingThread {
    observer_handler: Arc<ObserverHandler>,
    address_map: HashMap<NetworkAddress, String>,
}

impl SendingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        let mut instance = Self {
            observer_handler,
            address_map: HashMap::new(),
        };

        instance.address_map.insert(
            NetworkAddress::EngineController(0),
            String::from("169.254.0.6:25565"),
        );

        instance.address_map.insert(
            NetworkAddress::FlightController,
            String::from("169.254.0.7:25565"),
        );

        instance
    }

    pub fn run(&self) {
        let socket = UdpSocket::bind("0.0.0.0:25566")
            .expect("send_thread: Failed to bind socket");
        let mut buffer = [0_u8; BUFFER_SIZE];

        while process_is_running() {
            if let Some((event_id, addr, packet)) = self.receive_packet_event() {
                match packet.serialize(&mut buffer) {
                    Ok(size) => {
                        if let Some(address) = self.address_map.get(&addr) {
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
                                Err(format!("send_thread: No address for {:?}", addr)),
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

pub fn send_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();
    SendingThread::new(observer_handler).run();
}
