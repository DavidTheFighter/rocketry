use std::net::Ipv4Addr;

use big_brother::{interface::BigBrotherInterface, BigBrother};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    streamish_hal,
};

use crate::stream::Stream;

pub struct Streamish<'a> {
    stream: Option<Stream>,
    comms: BigBrother<'a, 64, Packet, NetworkAddress>,
}

impl<'a> Streamish<'a> {
    pub fn new(interface: &'a mut dyn BigBrotherInterface) -> Self {
        Self {
            stream: None,
            comms: BigBrother::new(
                NetworkAddress::Camera(0),
                rand::random(),
                NetworkAddress::Broadcast,
                [Some(interface), None],
            ),
        }
    }

    pub fn run(&mut self) {
        let mut last_broadcast_time = get_timestamp();

        loop {
            while let Some((packet, remote)) = self.comms.recv_packet().ok().flatten() {
                if let Some(mapping) = self.comms.get_network_mapping(remote) {
                    self.handle_packet(packet, Ipv4Addr::from(mapping));
                }
            }

            if get_timestamp() - last_broadcast_time > 0.5 {
                if let Err(e) = self
                    .comms
                    .send_packet(&Packet::Heartbeat, NetworkAddress::Broadcast)
                {
                    eprintln!("Streamish: Failed to send heartbeat packet: {:?}", e);
                }
                last_broadcast_time = get_timestamp();
            }
        }
    }

    fn handle_packet(&mut self, packet: Packet, src_addr: Ipv4Addr) {
        match packet {
            Packet::StreamishCommand(command) => self.handle_command(command, src_addr),
            Packet::Heartbeat => {}
            _ => eprintln!("Streamish: Received unhandled packet: {:?}", packet),
        }
    }

    fn handle_command(&mut self, command: streamish_hal::StreamishCommand, src_addr: Ipv4Addr) {
        match command {
            streamish_hal::StreamishCommand::StartCameraStream { port } => {
                if let Some(stream) = &mut self.stream {
                    if stream.port == port && stream.stream_addr == src_addr {
                        eprintln!(
                            "Streamish: Tried starting a new stream with same settings, ignoring"
                        );
                        return;
                    }

                    stream.stop();
                    self.stream = None;
                    eprintln!("Streamish: Stopping existing stream");
                }

                let stream = Stream::new(port, src_addr);
                self.stream = Some(stream);
            }
            streamish_hal::StreamishCommand::StopCameraStream => {
                if let Some(stream) = &mut self.stream {
                    stream.stop();
                    self.stream = None;
                } else {
                    eprintln!("Streamish: Received stop stream packet, but no stream is running");
                }
            }
            streamish_hal::StreamishCommand::StopApplication => {
                if let Some(stream) = &mut self.stream {
                    stream.stop();
                    self.stream = None;
                }

                std::process::exit(0);
            }
        }
    }

    // fn ipv4_from_ip(ip: IpAddr) -> Ipv4Addr {
    //     match ip {
    //         IpAddr::V4(ipv4) => ipv4,
    //         IpAddr::V6(ipv6) => ipv6.to_ipv4().expect("recv_thread: Failed to convert IPv6 address to IPv4"),
    //     }
    // }

    // fn ip_str_from_octets(ipv4: [u8; 4], port: u16) -> String {
    //     format!("{}.{}.{}.{}:{}", ipv4[0], ipv4[1], ipv4[2], ipv4[3], port)
    // }
}

fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}
