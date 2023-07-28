use std::net::{UdpSocket, IpAddr, Ipv4Addr};

use hal::comms_hal::{PACKET_BUFFER_SIZE, Packet, UDP_RECV_PORT};

use crate::{broadcast, stream::Stream};


pub struct Streamish {
    socket: UdpSocket,
    stream: Option<Stream>,
}

impl Streamish {
    pub fn new() -> Self {
        let addr = format!("0.0.0.0:{}", UDP_RECV_PORT);

        Self {
            socket: UdpSocket::bind(addr).expect("Failed to bind socket"),
            stream: None,
        }
    }

    pub fn run(&mut self) {
        self.socket.set_broadcast(true).expect("Failed to set broadcast");
        self.socket.set_nonblocking(true).expect("Failed to set non-blocking");

        let mut last_broadcast_time = get_timestamp();

        loop {
            let mut buffer = [0u8; PACKET_BUFFER_SIZE];

            while let Ok((bytes_read, addr)) = self.socket.recv_from(&mut buffer) {
                let packet = Packet::deserialize(&mut buffer[0..bytes_read])
                    .expect("Failed to deserialize packet");

                if let IpAddr::V4(ip4) = addr.ip() {
                    self.handle_packet(packet, ip4);
                }
            }

            if get_timestamp() - last_broadcast_time > 0.5 {
                broadcast::broadcast_ip(&self.socket);
                last_broadcast_time = get_timestamp();
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet, addr: Ipv4Addr) {
        match packet {
            Packet::StartCameraStream { port } => {
                if let Some(stream) = &mut self.stream {
                    stream.stop();
                    self.stream = None;
                    eprintln!("Streamish: Stopping existing stream");
                }

                let stream = Stream::new(port, addr);
                self.stream = Some(stream);
            },
            Packet::StopCameraStream => {
                if let Some(stream) = &mut self.stream {
                    stream.stop();
                    self.stream = None;
                } else {
                    eprintln!("Streamish: Received stop stream packet, but no stream is running");
                }
            },
            Packet::StopApplication => {
                if let Some(stream) = &mut self.stream {
                    stream.stop();
                    self.stream = None;
                }

                std::process::exit(0);
            },
            Packet::ComponentIpAddress { addr: _, ip: _ } => {},
            _ => eprintln!("Streamish: Received unhandled packet: {:?}", packet),
        }
    }
}

fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}