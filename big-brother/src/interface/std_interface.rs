use std::net::UdpSocket;

use crate::big_brother::{BigBrotherEndpoint, BigBrotherError, UDP_PORT};

use super::BigBrotherInterface;

pub const MAX_CHAIN_LENGTH: usize = 5;

pub struct StdInterface {
    udp_socket: UdpSocket,
    chain_port: Option<u16>,
    broadcast_ip: [u8; 4],
}

impl StdInterface {
    pub fn new(broadcast_ip: [u8; 4]) -> Result<Self, BigBrotherError> {
        let mut chain_port = None;

        for attempt in 0..MAX_CHAIN_LENGTH {
            let port = UDP_PORT + attempt as u16;
            let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", port));
            if let Ok(udp_socket) = udp_socket {
                print!("Bound to port {}\n", port);
                udp_socket
                    .set_nonblocking(true)
                    .map_err(|_| BigBrotherError::SocketConfigFailure)?;
                udp_socket
                    .set_broadcast(true)
                    .map_err(|_| BigBrotherError::SocketConfigFailure)?;

                if attempt > 0 {
                    chain_port = Some(port);
                }

                return Ok(Self {
                    udp_socket,
                    chain_port,
                    broadcast_ip,
                });
            }
        }

        Err(BigBrotherError::SocketBindFailure)
    }
}

impl BigBrotherInterface for StdInterface {
    fn poll(&mut self, _timestamp: u32) {}

    fn send_udp(
        &mut self,
        destination: BigBrotherEndpoint,
        data: &mut [u8],
    ) -> Result<(), BigBrotherError> {
        let destination = format!(
            "{}.{}.{}.{}:{}",
            destination.ip[0],
            destination.ip[1],
            destination.ip[2],
            destination.ip[3],
            destination.port,
        );

        self.udp_socket
            .send_to(data, destination)
            .map_err(|e| BigBrotherError::from(e))
            .map(|_| ())
    }

    fn recv_udp(
        &mut self,
        data: &mut [u8],
    ) -> Result<Option<(usize, BigBrotherEndpoint)>, BigBrotherError> {
        let recv = self.udp_socket.recv_from(data);
        if let Err(e) = &recv {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
        }

        let (size, remote) = recv.map_err(|e| BigBrotherError::from(e))?;

        let (ip, port) =
            parse_remote(&remote.to_string()).map_err(|_| BigBrotherError::SocketConfigFailure)?;

        let remote = BigBrotherEndpoint { ip, port };

        Ok(Some((size, remote)))
    }

    fn broadcast_ip(&self) -> [u8; 4] {
        self.broadcast_ip
    }

    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}

fn parse_remote(remote: &str) -> Result<([u8; 4], u16), BigBrotherError> {
    let remote = remote.split(":").collect::<Vec<&str>>();
    let ip = remote[0].split(".").collect::<Vec<&str>>();
    let port = remote[1]
        .parse::<u16>()
        .map_err(|_| BigBrotherError::SocketConfigFailure)?;

    if ip.len() != 4 {
        return Err(BigBrotherError::SocketConfigFailure);
    }

    let ip = [
        ip[0]
            .parse::<u8>()
            .map_err(|_| BigBrotherError::SocketConfigFailure)?,
        ip[1]
            .parse::<u8>()
            .map_err(|_| BigBrotherError::SocketConfigFailure)?,
        ip[2]
            .parse::<u8>()
            .map_err(|_| BigBrotherError::SocketConfigFailure)?,
        ip[3]
            .parse::<u8>()
            .map_err(|_| BigBrotherError::SocketConfigFailure)?,
    ];

    Ok((ip, port))
}

impl From<std::io::Error> for BigBrotherError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::AddrNotAvailable => BigBrotherError::SendUnnaddressable,
            std::io::ErrorKind::AddrInUse => BigBrotherError::SocketBindFailure,
            std::io::ErrorKind::OutOfMemory => BigBrotherError::SmoltcpSendBufferFull,
            _ => BigBrotherError::SendFailure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_remote() {
        let test_str = "192.168.0.15:8080";
        let (ip, port) = parse_remote(test_str).unwrap();
        assert_eq!(ip, [192, 168, 0, 15]);
        assert_eq!(port, 8080);
    }
}
