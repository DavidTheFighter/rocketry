#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use smoltcp::{wire::{self, IpEndpoint, EthernetAddress}, iface::{self, SocketStorage}, storage::PacketMetadata, socket::{UdpSocketBuffer, UdpSocket}};
use stm32_eth::{EthPins, RxRingEntry, TxRingEntry};
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

pub const DEVICE_MAC_ADDR: [u8; 6] = [0x00, 0x80, 0xE1, 0x00, 0x00, 0x01];
pub const DEVICE_IP_ADDR: wire::Ipv4Address = wire::Ipv4Address::new(169, 254, 0, 7);
pub const DEVICE_CIDR_LENGTH: u8 = 16;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(24.MHz()).freeze();

    // Create a timer based on SysTick
    let mut timer = p.TIM1.counter_ms(&clocks);
    timer.start(10.secs()).unwrap();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let eth_pins = EthPins {
        ref_clk: gpioa.pa1,
        crs: gpioa.pa7,
        tx_en: gpiob.pb11,
        tx_d0: gpiob.pb12,
        tx_d1: gpiob.pb13,
        rx_d0: gpioc.pc4,
        rx_d1: gpioc.pc5,
    };

    let mut rx_ring: [RxRingEntry; 2] = [RxRingEntry::INIT; 2];
    let mut tx_ring: [TxRingEntry; 1] = [TxRingEntry::INIT; 1];

    let (mut eth_dma, _eth_mac) = stm32_eth::new(
        p.ETHERNET_MAC,
        p.ETHERNET_MMC,
        p.ETHERNET_DMA,
        &mut rx_ring,
        &mut tx_ring,
        clocks,
        eth_pins,
    )
    .unwrap();

    eth_dma.enable_interrupt();

    // Storage

    let mut ip_addrs = [wire::IpCidr::Ipv4(wire::Ipv4Cidr::new(DEVICE_IP_ADDR, DEVICE_CIDR_LENGTH))];
    let mut sockets = [SocketStorage::EMPTY];
    let mut neighbor_cache: [Option<(wire::IpAddress, iface::Neighbor)>; 8] = [None; 8];
    let mut routes_cache: [Option<(wire::IpCidr, iface::Route)>; 8] = [None; 8];

    let mut rx_storage = [0_u8; 512];
    let mut rx_metadata_storage = [PacketMetadata::EMPTY; 8];
    let mut tx_storage = [0_u8; 512];
    let mut tx_metadata_storage = [PacketMetadata::EMPTY; 8];

    let neighbor_cache = smoltcp::iface::NeighborCache::new(&mut neighbor_cache[..]);
    let routes = smoltcp::iface::Routes::new(&mut routes_cache[..]);

    let (rx_buffer, tx_buffer) = (
        UdpSocketBuffer::new(&mut rx_metadata_storage[..], &mut rx_storage[..]),
        UdpSocketBuffer::new(&mut tx_metadata_storage[..], &mut tx_storage[..]),
    );
    let udp_socket = UdpSocket::new(rx_buffer, tx_buffer);

    let mut interface = iface::InterfaceBuilder::new(&mut eth_dma, &mut sockets[..])
        .hardware_addr(EthernetAddress(DEVICE_MAC_ADDR).into())
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    let udp_socket_handle = interface.add_socket(udp_socket);
    let udp_socket = interface.get_socket::<UdpSocket>(udp_socket_handle);

    let smoltcp_now = || -> smoltcp::time::Instant {
        smoltcp::time::Instant::from_millis(timer.now().duration_since_epoch().to_millis())
    };

    udp_socket.bind(4080).unwrap();
    interface.poll(smoltcp_now()).unwrap();

    let start_time = timer.now();

    loop {
        if let Some(time) = timer.now().checked_duration_since(start_time) {
            if time.to_millis() > 1000 {
                // TODO Boot into main program
            }
        }

        
    }
}
