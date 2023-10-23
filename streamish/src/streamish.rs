use std::{net::{UdpSocket, IpAddr, Ipv4Addr}, time::Duration};

use comms_manager::CommsManager;
use hal::comms_hal::{PACKET_BUFFER_SIZE, Packet, UDP_RECV_PORT, NetworkAddress};

use crate::stream::Stream;

const NETWORK_MAP_SIZE: usize = 64;

pub struct Streamish {
    socket: UdpSocket,
    stream: Option<Stream>,
    comms_manager: CommsManager<NETWORK_MAP_SIZE>,
}

impl Streamish {
    pub fn new() -> Self {
        let addr = format!("0.0.0.0:{}", UDP_RECV_PORT);

        Self {
            socket: UdpSocket::bind(addr).expect("Failed to bind socket"),
            stream: None,
            comms_manager: CommsManager::<NETWORK_MAP_SIZE>::new(hal::comms_hal::NetworkAddress::GroundCamera(0)),
        }
    }

    pub fn run(&mut self) {
        let timeout = Duration::from_millis(10);

        self.socket.set_broadcast(true).expect("Failed to set broadcast");
        self.socket.set_read_timeout(Some(timeout)).expect("Failed to set read timeout");

        let mut last_broadcast_time = get_timestamp();
        let mut buffer = [0u8; PACKET_BUFFER_SIZE];

        loop {
            while let Ok((bytes_read, addr)) = self.socket.recv_from(&mut buffer) {
                let source_address = Self::ipv4_from_ip(addr.ip()).octets();

                match self.comms_manager.extract_packet(&mut buffer[..bytes_read], source_address) {
                    Ok((packet, source_address)) => {
                        if let Some(ip) = self.comms_manager.network_address_to_ip(source_address) {
                            self.handle_packet(packet, Ipv4Addr::from(ip));
                        }
                    },
                    Err(e) => eprintln!("Streamish: Failed to extract packet: {:?}", e),
                }
            }

            if get_timestamp() - last_broadcast_time > 0.5 {
                self.send_heartbeat();
                last_broadcast_time = get_timestamp();
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet, src_addr: Ipv4Addr) {
        match packet {
            Packet::StartCameraStream { port } => {
                if let Some(stream) = &mut self.stream {
                    if stream.port == port && stream.stream_addr == src_addr {
                        eprintln!("Streamish: Tried starting a new stream with same settings, ignoring");
                        return;
                    }

                    stream.stop();
                    self.stream = None;
                    eprintln!("Streamish: Stopping existing stream");
                }

                let stream = Stream::new(port, src_addr);
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
            Packet::Heartbeat => {},
            _ => eprintln!("Streamish: Received unhandled packet: {:?}", packet),
        }
    }

    fn send_heartbeat(&self) {
        let mut buffer = [0u8; PACKET_BUFFER_SIZE];
        let packet = Packet::Heartbeat;
        let destination = NetworkAddress::Broadcast;
        match self.comms_manager.process_packet(packet, destination, &mut buffer) {
            Ok((bytes_written, ip)) => {
                let addr = Self::ip_str_from_octets(ip, UDP_RECV_PORT);
                self.socket
                    .send_to(&buffer[0..bytes_written], addr)
                    .expect("Failed to send packet");
            },
            Err(e) => eprintln!("Streamish: Failed to process packet: {:?}", e),
        }
    }

    fn ipv4_from_ip(ip: IpAddr) -> Ipv4Addr {
        match ip {
            IpAddr::V4(ipv4) => ipv4,
            IpAddr::V6(ipv6) => ipv6.to_ipv4().expect("recv_thread: Failed to convert IPv6 address to IPv4"),
        }
    }

    fn ip_str_from_octets(ipv4: [u8; 4], port: u16) -> String {
        format!("{}.{}.{}.{}:{}", ipv4[0], ipv4[1], ipv4[2], ipv4[3], port)
    }
}

fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}