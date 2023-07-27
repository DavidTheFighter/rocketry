use std::net::{UdpSocket, IpAddr};

use hal::comms_hal::{Packet, NetworkAddress};
use local_ip_address::local_ip;

pub fn broadcast_ip(socket: &UdpSocket) {
    let my_local_ip = local_ip().expect("Failed to get local IP address");

    if let IpAddr::V4(ip4) = my_local_ip {
        let packet = Packet::ComponentIpAddress {
            addr: NetworkAddress::GroundCamera(0),
            ip: ip4.octets(),
        };

        let mut buffer = [0u8; 256];
        let bytes_written = packet.serialize(&mut buffer).expect("Failed to serialize packet");

        socket
            .send_to(&buffer[0..bytes_written], "255.255.255.255")
            .expect("Failed to send packet");
    }
}