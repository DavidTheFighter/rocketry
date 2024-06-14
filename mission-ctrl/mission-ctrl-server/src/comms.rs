use std::{net::UdpSocket, sync::Arc};

use big_brother::{big_brother::WORKING_BUFFER_SIZE, interface::{bridge_interface::BridgeInterface, std_interface::StdInterface}, serdes::PacketMetadata, BigBrother};
use shared::{comms_hal::{NetworkAddress, Packet}, REALTIME_SIMULATION_CTRL_PORT, REALTIME_SIMULATION_SIM_PORT};

use crate::{
    observer::{ObserverEvent, ObserverHandler, ObserverResponse},
    process_is_running, timestamp,
};

const NETWORK_MAP_SIZE: usize = 128;

struct CommsThread {
    observer_handler: Arc<ObserverHandler>,
    start_timestamp: f64,
}

impl CommsThread {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            start_timestamp: timestamp(),
        }
    }

    pub fn run(&mut self) {
        let mut std_interface = StdInterface::new([169, 254, 255, 255])
            .expect("Failed to create std interface for comms thread");

        let mut simulation_interface = BridgeInterface::new(REALTIME_SIMULATION_CTRL_PORT, REALTIME_SIMULATION_SIM_PORT)
            .expect("Failed to create simulation interface for comms thread");
        let mut bb: BigBrother<'_, NETWORK_MAP_SIZE, Packet, NetworkAddress> = BigBrother::new(
            NetworkAddress::MissionControl,
            rand::random(),
            NetworkAddress::Broadcast,
            [Some(&mut std_interface), Some(&mut simulation_interface)],
        );

        let mut last_poll_time = timestamp();

        while process_is_running() {
            if let Some((event_id, address, packet)) = self.get_send_packet_event() {
                if let Err(err) = bb.send_packet(&packet, address) {
                    eprintln!(
                        "comms_thread: Failed to send packet: {:?} ({:?})",
                        err, packet
                    );
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

            if timestamp() - last_poll_time > 0.001 {
                bb.poll_1ms(((timestamp() - self.start_timestamp) * 1e3) as u32);

                self.update_bitrates(bb.get_recv_bitrate() as u32);

                last_poll_time = timestamp();
            }

            std::thread::sleep(std::time::Duration::from_micros(250));
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

    fn update_bitrates(&mut self, bitrate: u32) {
        self.observer_handler.notify(ObserverEvent::UpdateBitrate {
            source_address: NetworkAddress::FlightController,
            bitrate,
        });
    }
}

pub fn comms_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();
    CommsThread::new(observer_handler).run();
}
