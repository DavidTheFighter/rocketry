use std::{net::UdpSocket, time::Duration, sync::Arc, io::ErrorKind};
use hal::comms_hal::Packet;

use crate::{observer::{ObserverEvent, ObserverHandler}, process_is_running};

const BUFFER_SIZE: usize = 1024;

struct RecievingThread {
    observer_handler: Arc<ObserverHandler>,
}

impl RecievingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
        }
    }

    pub fn run(&mut self) {
        let socket = UdpSocket::bind("0.0.0.0:25565")
            .expect("recv_thread: Failed to bind socket");
        let mut buffer = [0_u8; BUFFER_SIZE];

        socket
            .set_read_timeout(Some(Duration::from_millis(10)))
            .expect("recv_thread: Failed to set socket timeout");

        while process_is_running() {
            match socket.recv_from(&mut buffer) {
                Ok((size, _address)) => {
                    let packet = Packet::deserialize(&mut buffer[0..size]);

                    match packet {
                        Ok(packet) => {
                            self.observer_handler.notify(ObserverEvent::PacketReceived(packet));
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
}

pub fn recv_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();
    RecievingThread::new(observer_handler).run();
}
