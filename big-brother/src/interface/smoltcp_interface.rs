use smoltcp::{iface, phy, socket::udp, storage, wire};

use crate::big_brother::{BigBrotherEndpoint, BigBrotherError, UDP_PORT};

use super::BigBrotherInterface;

const SOCKET_STORAGE_SIZE: usize = 512;
const SOCKET_METADATA_SIZE: usize = 8;

pub struct SmoltcpInterface<'a, D>
where
    D: phy::Device + ?Sized,
{
    interface: iface::Interface,
    device: &'a mut D,
    sockets_set: iface::SocketSet<'a>,
    udp_socket_handle: iface::SocketHandle,
}

impl<'a, D> SmoltcpInterface<'a, D>
where
    D: phy::Device + ?Sized,
{
    pub fn new(
        config: iface::Config,
        device: &'a mut D,
        ip_addr: wire::IpCidr,
        sockets: &'a mut [iface::SocketStorage<'a>; 1],
        udp_socket_buffer: &'a mut UdpSocketBuffer,
        timestamp: u32,
    ) -> Self {
        let timestamp = smoltcp::time::Instant::from_millis(timestamp);

        let (rx_buffer, tx_buffer) = udp_socket_buffer.into_udp_socket_buffers();

        let mut interface = iface::Interface::new(config, device, timestamp);

        interface.update_ip_addrs(|addr| {
            addr.push(ip_addr).ok();
        });

        let mut sockets_set = iface::SocketSet::new(&mut sockets[..]);

        let mut udp_socket = udp::Socket::new(rx_buffer, tx_buffer);
        udp_socket
            .bind(UDP_PORT)
            .expect("failed to bind UDP socket");

        let udp_socket_handle = sockets_set.add(udp_socket);

        Self {
            interface,
            device,
            sockets_set,
            udp_socket_handle,
        }
    }
}

impl<'a, D> BigBrotherInterface for SmoltcpInterface<'a, D>
where
    D: phy::Device + ?Sized,
{
    fn poll(&mut self, timestamp: u32) {
        let timestamp = smoltcp::time::Instant::from_millis(timestamp);

        self.interface
            .poll(timestamp, self.device, &mut self.sockets_set);
    }

    fn send_udp(
        &mut self,
        destination: BigBrotherEndpoint,
        data: &mut [u8],
    ) -> Result<(), BigBrotherError> {
        let destination = wire::IpEndpoint::new(
            wire::IpAddress::Ipv4(wire::Ipv4Address(destination.ip)),
            destination.port,
        );

        self.sockets_set
            .get_mut::<udp::Socket>(self.udp_socket_handle)
            .send_slice(data, destination)
            .map_err(|e| BigBrotherError::from(e))
    }

    fn recv_udp(
        &mut self,
        data: &mut [u8],
    ) -> Result<Option<(usize, BigBrotherEndpoint)>, BigBrotherError> {
        let socket = self
            .sockets_set
            .get_mut::<udp::Socket>(self.udp_socket_handle);

        if !socket.can_recv() {
            return Ok(None);
        }

        let (size, metadata) = socket
            .recv_slice(data)
            .map_err(|e| BigBrotherError::from(e))?;

        let remote = BigBrotherEndpoint {
            ip: match metadata.endpoint.addr {
                wire::IpAddress::Ipv4(ip) => ip.0,
            },
            port: metadata.endpoint.port,
        };

        Ok(Some((size, remote)))
    }

    fn as_mut_any(&'static mut self) -> &mut dyn core::any::Any {
        self
    }
}

pub struct UdpSocketBuffer {
    pub rx_storage: [u8; SOCKET_STORAGE_SIZE],
    pub rx_metadata_storage: [storage::PacketMetadata<udp::UdpMetadata>; SOCKET_METADATA_SIZE],
    pub tx_storage: [u8; SOCKET_STORAGE_SIZE],
    pub tx_metadata_storage: [storage::PacketMetadata<udp::UdpMetadata>; SOCKET_METADATA_SIZE],
}

impl UdpSocketBuffer {
    pub const fn new() -> Self {
        Self {
            rx_storage: [0_u8; SOCKET_STORAGE_SIZE],
            rx_metadata_storage: [storage::PacketMetadata::EMPTY; SOCKET_METADATA_SIZE],
            tx_storage: [0_u8; SOCKET_STORAGE_SIZE],
            tx_metadata_storage: [storage::PacketMetadata::EMPTY; SOCKET_METADATA_SIZE],
        }
    }

    pub fn into_udp_socket_buffers(&mut self) -> (udp::PacketBuffer, udp::PacketBuffer) {
        (
            udp::PacketBuffer::new(&mut self.rx_metadata_storage[..], &mut self.rx_storage[..]),
            udp::PacketBuffer::new(&mut self.tx_metadata_storage[..], &mut self.tx_storage[..]),
        )
    }
}

impl From<udp::SendError> for BigBrotherError {
    fn from(send_error: udp::SendError) -> Self {
        match send_error {
            udp::SendError::Unaddressable => BigBrotherError::SendUnnaddressable,
            udp::SendError::BufferFull => BigBrotherError::SmoltcpSendBufferFull,
        }
    }
}

impl From<udp::RecvError> for BigBrotherError {
    fn from(recv_error: udp::RecvError) -> Self {
        match recv_error {
            udp::RecvError::Exhausted => BigBrotherError::SmoltcpRecvExhausted,
        }
    }
}
