use std::{net::UdpSocket, sync::{Arc, RwLock}, time::Duration};
use shared::comms_manager::CommsManager;
use shared::comms_hal::{Packet, NetworkAddress, PACKET_BUFFER_SIZE};

use crate::{observer::{ObserverEvent, ObserverHandler, ObserverResponse}, process_is_running, timestamp};

use super::{SEND_PORT, RECV_PORT, NETWORK_MAP_SIZE};

struct SendingThread {
    observer_handler: Arc<ObserverHandler>,
    comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>,
    socket: UdpSocket,
}

impl SendingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>, comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", SEND_PORT))
            .expect("send_thread: Failed to bind socket");
        socket.set_broadcast(true).expect("send_thread: Failed to set broadcast on socket");
        comms_manager.as_ref().write().unwrap().set_broadcast_address([169, 254, 255, 255]);

        Self {
            observer_handler,
            comms_manager,
            socket,
        }
    }

    pub fn run(&mut self) {
        let mut last_heartbeat_time = timestamp();

        self.send_heartbeat();

        while process_is_running() {
            if let Some((event_id, destination, packet)) = self.receive_packet_event() {
                self.send_packet(packet, destination, event_id);
            }

            if timestamp() - last_heartbeat_time > 0.5 {
                self.send_heartbeat();
                last_heartbeat_time = timestamp();
            }
        }
    }

    fn send_packet(&self, packet: Packet, destination: NetworkAddress, event_id: u64) {
        let mut buffer = [0_u8; PACKET_BUFFER_SIZE];

        match self.comms().process_packet(packet, destination, &mut buffer) {
            Ok((size, ip)) => {
                let address = Self::ip_str_from_octets(ip, RECV_PORT);

                if let Err(err) = self.socket.send_to(&buffer[0..size], address) {
                    self.send_packet_resonse(
                        event_id,
                        Err(format!("send_thread: Failed to send packet: {err}")),
                    );
                } else {
                    self.send_packet_resonse(event_id, Ok(ObserverResponse::Empty));
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

    fn send_heartbeat(&self) {
        let mut buffer = [0_u8; PACKET_BUFFER_SIZE];
        let packet = Packet::Heartbeat;

        match self.comms().process_packet(packet, NetworkAddress::Broadcast, &mut buffer) {
            Ok((size, ip)) => {
                let address = Self::ip_str_from_octets(ip, RECV_PORT);

                if let Err(err) = self.socket.send_to(&buffer[0..size], address) {
                    eprintln!("send_thread: Failed to send heartbeat packet: {:?}", err);
                }
            },
            Err(e) => {
                eprintln!("send_thread: Failed to send heartbeat packet: {:?}", e);
            }
        }
    }

    fn send_packet_resonse(&self, event_id: u64, result: Result<ObserverResponse, String>) {
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

    fn comms(&self) -> std::sync::RwLockReadGuard<'_, CommsManager<NETWORK_MAP_SIZE>> {
        self.comms_manager.as_ref().read().unwrap()
    }

    fn ip_str_from_octets(ipv4: [u8; 4], port: u16) -> String {
        format!("{}.{}.{}.{}:{}", ipv4[0], ipv4[1], ipv4[2], ipv4[3], port)
    }
}

pub fn send_thread(observer_handler: Arc<ObserverHandler>, comms_manager: Arc<RwLock<CommsManager<NETWORK_MAP_SIZE>>>) {
    observer_handler.register_observer_thread();
    SendingThread::new(observer_handler, comms_manager).run();
}
