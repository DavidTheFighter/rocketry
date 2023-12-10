use crate::app;
use shared::comms_hal::{NetworkAddress, Packet, UDP_RECV_PORT};
use rtic::mutex_prelude::{TupleExt03, TupleExt04};

pub fn eth_interrupt(ctx: app::eth_interrupt::Context) {
    ctx.shared.fcu.lock(|fcu| {
        fcu.poll();
    });
}

#[derive(Copy, Clone)]
pub struct IpSocketStorage {
    pub rx_storage: [u8; 512],
    pub rx_metadata_storage: [PacketMetadata<IpEndpoint>; 8],
    pub tx_storage: [u8; 512],
    pub tx_metadata_storage: [PacketMetadata<IpEndpoint>; 8],
}

impl IpSocketStorage {
    pub const fn new() -> Self {
        Self {
            rx_storage: [0_u8; 512],
            rx_metadata_storage: [PacketMetadata::EMPTY; 8],
            tx_storage: [0_u8; 512],
            tx_metadata_storage: [PacketMetadata::EMPTY; 8],
        }
    }

    pub fn into_udp_socket_buffers(&mut self) -> (UdpSocketBuffer, UdpSocketBuffer) {
        (
            UdpSocketBuffer::new(&mut self.rx_metadata_storage[..], &mut self.rx_storage[..]),
            UdpSocketBuffer::new(&mut self.tx_metadata_storage[..], &mut self.tx_storage[..]),
        )
    }
}

pub struct NetworkingStorage {
    pub ip_addrs: [wire::IpCidr; 1],
    pub sockets: [iface::SocketStorage<'static>; 1],
    pub udp_socket_storage: IpSocketStorage,
    pub neighbor_cache: [Option<(wire::IpAddress, iface::Neighbor)>; 8],
    pub routes_cache: [Option<(wire::IpCidr, iface::Route)>; 8],
}

impl NetworkingStorage {
    pub const fn new() -> Self {
        let ip_addr = wire::IpCidr::Ipv4(wire::Ipv4Cidr::new(DEVICE_IP_ADDR, DEVICE_CIDR_LENGTH));

        Self {
            ip_addrs: [ip_addr],
            sockets: [SocketStorage::EMPTY],
            udp_socket_storage: IpSocketStorage::new(),
            neighbor_cache: [None; 8],
            routes_cache: [None; 8],
        }
    }
}
