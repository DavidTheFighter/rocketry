use smoltcp::{iface, phy, socket::udp, storage, wire};

use crate::big_brother::{BigBrotherEndpoint, BigBrotherError, UDP_PORT};

use super::BigBrotherInterface;

const SOCKET_STORAGE_SIZE: usize = 512;
const SOCKET_METADATA_SIZE: usize = 8;

pub struct SmoltcpInterfaceStorage<'a> {
    sockets: [iface::SocketStorage<'a>; 1],
    udp_socket_buffer: UdpSocketBuffer,
}

pub struct SmoltcpInterface<'a, D>
where
    D: phy::Device + Sized,
{
    interface: iface::Interface,
    device: D,
    sockets_set: iface::SocketSet<'a>,
    udp_socket_handle: iface::SocketHandle,
}

impl<'a, D> SmoltcpInterface<'a, D>
where
    D: phy::Device + Sized,
{
    pub fn new(
        mac_address: [u8; 6],
        mut device: D,
        ip_addr: [u8; 4],
        cidr_len: u8,
        storage: &'a mut SmoltcpInterfaceStorage<'a>,
        timestamp: u32,
    ) -> Self {
        let timestamp = smoltcp::time::Instant::from_millis(timestamp);

        let (rx_buffer, tx_buffer) = storage.udp_socket_buffer.into_udp_socket_buffers();

        let config = iface::Config::new(wire::EthernetAddress(mac_address).into());
        let mut interface = iface::Interface::new(config, &mut device, timestamp);

        let ip_cidr = wire::IpCidr::Ipv4(wire::Ipv4Cidr::new(wire::Ipv4Address(ip_addr), cidr_len));
        interface.update_ip_addrs(|addr| {
            addr.push(ip_cidr).ok();
        });

        let mut sockets_set = iface::SocketSet::new(&mut storage.sockets[..]);

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
    D: phy::Device + Sized,
{
    fn poll(&mut self, timestamp: u32) {
        let timestamp = smoltcp::time::Instant::from_millis(timestamp);

        self.interface
            .poll(timestamp, &mut self.device, &mut self.sockets_set);
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

    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
        None
    }
}

impl<'a> SmoltcpInterfaceStorage<'a> {
    pub const fn new() -> Self {
        Self {
            sockets: [iface::SocketStorage::EMPTY],
            udp_socket_buffer: UdpSocketBuffer::new(),
        }
    }
}

struct UdpSocketBuffer {
    rx_storage: [u8; SOCKET_STORAGE_SIZE],
    rx_metadata_storage: [storage::PacketMetadata<udp::UdpMetadata>; SOCKET_METADATA_SIZE],
    tx_storage: [u8; SOCKET_STORAGE_SIZE],
    tx_metadata_storage: [storage::PacketMetadata<udp::UdpMetadata>; SOCKET_METADATA_SIZE],
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
