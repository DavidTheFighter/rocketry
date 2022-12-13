use smoltcp::{wire::{self, IpEndpoint, EthernetAddress}, iface::{self, SocketStorage}, storage::PacketMetadata, socket::{UdpSocketBuffer, UdpSocket}};
use stm32_eth::{RxRingEntry, TxRingEntry, EthernetDMA};
use crate::now_fn;

pub const DEVICE_MAC_ADDR: [u8; 6] = [0x00, 0x80, 0xE1, 0x00, 0x00, 0x00];
pub const DEVICE_IP_ADDR: wire::Ipv4Address = wire::Ipv4Address::new(169, 254, 0, 6);
pub const DEVICE_CIDR_LENGTH: u8 = 16;
pub const DEVICE_PORT: u16 = 25565;

pub const RX_RING_ENTRY_DEFAULT: RxRingEntry = RxRingEntry::new();
pub const TX_RING_ENTRY_DEFAULT: TxRingEntry = TxRingEntry::new();

pub fn init_comms(
    net_storage: &'static mut NetworkingStorage,
    eth_dma: &'static mut EthernetDMA<'static, 'static>,
) -> (iface::Interface<'static, &'static mut EthernetDMA<'static, 'static>>, iface::SocketHandle) {
    let neighbor_cache = smoltcp::iface::NeighborCache::new(&mut net_storage.neighbor_cache[..]);
    let routes = smoltcp::iface::Routes::new(&mut net_storage.routes_cache[..]);

    let (rx_buffer, tx_buffer) = net_storage.udp_socket_storage.into_udp_socket_buffers();
    let udp_socket = UdpSocket::new(rx_buffer, tx_buffer);

    let mut interface = iface::InterfaceBuilder::new(eth_dma, &mut net_storage.sockets[..])
        .hardware_addr(EthernetAddress(DEVICE_MAC_ADDR).into())
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut net_storage.ip_addrs[..])
        .routes(routes)
        .finalize();

    let udp_socket_handle = interface.add_socket(udp_socket);
    let udp_socket = interface.get_socket::<UdpSocket>(udp_socket_handle);

    udp_socket.bind(DEVICE_PORT).unwrap();
    interface.poll(now_fn()).unwrap();

    (interface, udp_socket_handle)
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