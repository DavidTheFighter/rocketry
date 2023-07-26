use std::net::{UdpSocket, IpAddr};
use std::process::Command;

use hal::comms_hal::{Packet, NetworkAddress};
use local_ip_address::local_ip;
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn broadcast_ip() {
    let socket = UdpSocket::bind("0.0.0.0:25570").expect("Failed to bind socket");
    socket.set_broadcast(true).expect("Failed to set broadcast");

    let my_local_ip = local_ip().expect("Failed to get local IP address");

    if let IpAddr::V4(ip4) = my_local_ip {
        let packet = Packet::ComponentIpAddress {
            addr: NetworkAddress::GroundCamera(0),
            ip: ip4.octets(),
        };

        let mut buffer = [0u8; 256];
        let bytes_written = packet.serialize(&mut buffer).expect("Failed to serialize packet");

        socket
            .send_to(&buffer[0..bytes_written], "255.255.255.255:25565")
            .expect("Failed to send packet");

        println!("Broadcast IP: {}", ip4);
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn streamish(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(broadcast_ip, m)?)?;
    Ok(())
}