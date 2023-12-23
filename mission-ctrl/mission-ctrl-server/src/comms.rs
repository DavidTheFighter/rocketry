use std::sync::Arc;

use big_brother::{interface::std_interface::StdInterface, BigBrother};
use shared::comms_hal::{NetworkAddress, Packet};

use crate::{observer::{ObserverHandler, ObserverEvent, ObserverResponse}, process_is_running, timestamp};

const NETWORK_MAP_SIZE: usize = 128;
const HEARTBEAT_TIME_S: f64 = 0.25;

struct CommsThread {
    observer_handler: Arc<ObserverHandler>,
}

impl CommsThread {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
        }
    }

    pub fn run(&mut self) {
        let mut std_interface = StdInterface::new([169, 254, 255, 255])
            .expect("Failed to create std interface for comms thread");
        let mut bb: BigBrother<'_, NETWORK_MAP_SIZE, Packet, NetworkAddress> = BigBrother::new(
            NetworkAddress::MissionControl,
            rand::random(),
            NetworkAddress::Broadcast,
            [Some(&mut std_interface), None],
        );

        let mut last_heartbeat_time = timestamp() - HEARTBEAT_TIME_S;

        while process_is_running() {
            if let Some((event_id, address, packet)) = self.get_send_packet_event() {
                if let Err(err) = bb.send_packet(&packet, address) {
                    eprintln!("comms_thread: Failed to send packet: {:?} ({:?})", err, packet);
                }

                self.observer_handler.notify(ObserverEvent::EventResponse(
                    event_id,
                    Ok(ObserverResponse::Empty),
                ));
            }

            loop {
                match bb.recv_packet() {
                    Ok(recv) => {
                        if let Some((packet, remote)) = recv {

                            if let Some(ip) = bb.get_network_mapping(remote) {
                                self.observer_handler.notify(ObserverEvent::PacketReceived {
                                    packet,
                                    ip,
                                    address: remote,
                                });
                            } else {
                                eprintln!("comms_thread: Failed to get mapping for a packet that was just received!")
                            }
                        } else {
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("comms_thread: Failed to receive packet: {:?}", err);

                        break;
                    }
                }
            }

            if timestamp() - last_heartbeat_time > HEARTBEAT_TIME_S {
                if let Err(err) = bb.send_packet(&Packet::Heartbeat, NetworkAddress::Broadcast) {
                    eprintln!("comms_thread: Failed to send heartbeat: {:?}", err);
                }

                last_heartbeat_time = timestamp();
            }
        }
    }

    fn get_send_packet_event(&self) -> Option<(u64, NetworkAddress, Packet)> {
        if let Some((event_id, event)) = self.observer_handler.get_event() {
            if let ObserverEvent::SendPacket { address, packet } = event {
                return Some((event_id, address, packet));
            }
        }

        None
    }
}

pub fn comms_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();
    CommsThread::new(observer_handler).run();
}
