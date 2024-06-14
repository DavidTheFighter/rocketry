use std::net::UdpSocket;

use crate::big_brother::{BigBrotherEndpoint, BigBrotherError, UDP_PORT};

use super::BigBrotherInterface;

pub const MAX_CHAIN_LENGTH: usize = 5;

pub struct BridgeInterface {
    udp_socket: UdpSocket,
    target_port: u16,
}

impl BridgeInterface {
    pub fn new(bind_port: u16, target_port: u16) -> Result<Self, BigBrotherError> {
        let udp_socket = UdpSocket::bind(format!("127.0.0.1:{}", bind_port));
        if let Ok(udp_socket) = udp_socket {
            print!("BBound to port {}\n", bind_port);
            udp_socket
                .set_nonblocking(true)
                .map_err(|_| BigBrotherError::SocketConfigFailure)?;
            udp_socket
                .set_broadcast(true)
                .map_err(|_| BigBrotherError::SocketConfigFailure)?;

            return Ok(Self {
                udp_socket,
                target_port,
            });
        }

        Err(BigBrotherError::SocketBindFailure)
    }
}

impl BigBrotherInterface for BridgeInterface {
    fn poll(&mut self, _timestamp: u32) {}

    fn send_udp(
        &mut self,
        destination: BigBrotherEndpoint,
        data: &mut [u8],
    ) -> Result<(), BigBrotherError> {
        let destination = format!(
            "127.0.0.1:{}",
            self.target_port,
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

        let (_, port) =
            parse_remote(&remote.to_string()).map_err(|_| BigBrotherError::SocketConfigFailure)?;

        let remote = BigBrotherEndpoint {
            ip: [127, 0, 0, 1],
            port,
        };

        Ok(Some((size, remote)))
    }

    fn broadcast_ip(&self) -> [u8; 4] {
        [127, 0, 0, 1]
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
