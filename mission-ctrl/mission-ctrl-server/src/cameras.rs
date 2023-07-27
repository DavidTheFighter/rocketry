use std::{sync::Arc, time::Duration};

use hal::comms_hal::Packet;

use crate::{observer::{ObserverHandler, ObserverEvent}, process_is_running};

pub struct CameraStreaming {
    observer_handler: Arc<ObserverHandler>,
}

impl CameraStreaming {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                self.handle_packet(packet);
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet {
            Packet::ComponentIpAddress { addr, ip } => {
                println!("Camera {:?} is at {:?}", addr, ip);
            },
            _ => {}
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