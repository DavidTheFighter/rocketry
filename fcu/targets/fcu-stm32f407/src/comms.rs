use crate::app;
use hal::comms_hal::{NetworkAddress, Packet, UDP_RECV_PORT};
use rtic::mutex_prelude::{TupleExt03, TupleExt04};
use smoltcp::{
    iface::{self, SocketStorage},
    socket::{UdpSocket, UdpSocketBuffer},
    storage::PacketMetadata,
    wire::{self, EthernetAddress, IpEndpoint},
};
use stm32_eth::{EthernetDMA, RxRingEntry, TxRingEntry};

pub const DEVICE_MAC_ADDR: [u8; 6] = [0x00, 0x80, 0xE1, 0x00, 0x00, 0x01];
pub const DEVICE_IP_ADDR: wire::Ipv4Address = wire::Ipv4Address::new(169, 254, 0, 7);
pub const DEVICE_CIDR_LENGTH: u8 = 16;

pub const RX_RING_ENTRY_DEFAULT: RxRingEntry = RxRingEntry::new();
pub const TX_RING_ENTRY_DEFAULT: TxRingEntry = TxRingEntry::new();

pub fn eth_interrupt(ctx: app::eth_interrupt::Context) {
    let iface = ctx.shared.interface;
    let udp = ctx.shared.udp_socket_handle;
    let packet_queue = ctx.shared.packet_queue;
    let comms_manager = ctx.shared.comms_manager;

    (iface, udp, packet_queue, comms_manager).lock(|iface, udp_handle, packet_queue, comms_manager| {
        iface.device_mut().interrupt_handler();
        iface.poll(smoltcp_now()).ok();

        let buffer = ctx.local.data;
        let udp_socket = iface.get_socket::<UdpSocket>(*udp_handle);

        if !udp_socket.can_recv() {
            return;
        }

        while let Ok((recv_bytes, sender)) = udp_socket.recv_slice(buffer) {
            let source_address = match sender.addr {
                wire::IpAddress::Unspecified => [255, 255, 255, 255],
                wire::IpAddress::Ipv4(ip) => ip.0,
                _ => [255, 255, 255, 255],
            };

            match comms_manager.extract_packet(buffer, source_address) {
                Ok((packet, address)) => {
                    packet_queue.enqueue((address, packet)).unwrap();
                },
                Err(e) => {
                    defmt::error!("Failed to extract packet: {:?}", e.error_str());
                }
            }
        }

        iface.poll(smoltcp_now()).ok();
    });
}

pub fn send_packet(ctx: app::send_packet::Context, packet: Packet, address: NetworkAddress) {
    let iface = ctx.shared.interface;
    let udp = ctx.shared.udp_socket_handle;
    let comms_manager = ctx.shared.comms_manager;

    (iface, udp, comms_manager).lock(|iface, udp_handle, comms_manager| {
        let udp_socket = iface.get_socket::<UdpSocket>(*udp_handle);
        let buffer = ctx.local.data;

        if !udp_socket.can_send() {
            return;
        }

        let result = comms_manager.process_packet(packet, address, buffer);

        match result {
            Ok((size, ip)) => {
                let ip = wire::Ipv4Address::from_bytes(&ip);
                let endpoint = IpEndpoint::new(ip.into(), UDP_RECV_PORT);

                let send_result = udp_socket
                    .send_slice(&buffer[0..size], endpoint);

                if let Err(err) = send_result {
                    // defmt::error!("Failed to send packet: {:?}", err);
                    match err {
                        smoltcp::Error::Exhausted => {
                            defmt::error!("Failed to send packet: Exhausted");
                            // app::send_packet::spawn_after(5.millis().into(), packet, address).unwrap();
                            return;
                        },
                        smoltcp::Error::Illegal => defmt::error!("Failed to send packet: Illegal"),
                        smoltcp::Error::Unaddressable => defmt::error!("Failed to send packet: Unaddressable"),
                        smoltcp::Error::Finished => defmt::error!("Failed to send packet: Finished"),
                        smoltcp::Error::Truncated => defmt::error!("Failed to send packet: Truncated"),
                        smoltcp::Error::Checksum => defmt::error!("Failed to send packet: Checksum"),
                        smoltcp::Error::Unrecognized => defmt::error!("Failed to send packet: Unrecognized"),
                        smoltcp::Error::Fragmented => defmt::error!("Failed to send packet: Fragmented"),
                        smoltcp::Error::Malformed => defmt::error!("Failed to send packet: Malformed"),
                        smoltcp::Error::Dropped => defmt::error!("Failed to send packet: Dropped"),
                        smoltcp::Error::NotSupported => defmt::error!("Failed to send packet: NotSupported"),
                        _ => defmt::error!("Failed to send packet: Unknown"),
                    }
                }
            },
            Err(e) => {
                defmt::error!("Failed to process packet: {:?}", e.error_str());
            }
        }

        if let Err(_err) = iface.poll(smoltcp_now()) {
            defmt::error!("Failed to poll interface");
        }
    });
}

pub fn init_comms(
    net_storage: &'static mut NetworkingStorage,
    eth_dma: &'static mut EthernetDMA<'static, 'static>,
) -> (
    iface::Interface<'static, &'static mut EthernetDMA<'static, 'static>>,
    iface::SocketHandle,
) {
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

    udp_socket.bind(UDP_RECV_PORT).unwrap();
    interface.poll(smoltcp_now()).unwrap();

    (interface, udp_socket_handle)
}

fn smoltcp_now() -> smoltcp::time::Instant {
    let time = app::monotonics::now().duration_since_epoch().ticks();
    smoltcp::time::Instant::from_millis(time as i64)
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
