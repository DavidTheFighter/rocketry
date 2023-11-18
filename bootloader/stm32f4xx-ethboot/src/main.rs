#![no_std]
#![no_main]

use core::{ptr::read_volatile, sync::atomic::{AtomicU32, Ordering}};
use cortex_m::peripheral::SYST;
use defmt_brtt as _;

use ethboot_shared::{SocketBuffer, Bootloader, BootloaderAction};
use hal::{interrupt, flash::{LockedFlash, FlashExt}};
use panic_halt as _;

use cortex_m_rt::{entry, exception};
use smoltcp::{
    wire::{self, EthernetAddress},
    iface, time::Instant,
};
use stm32_eth::{EthPins, dma::{RxRingEntry, TxRingEntry}, PartsIn};
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

pub const DEVICE_MAC_ADDR: EthernetAddress = EthernetAddress([0x00, 0x80, 0xE1, 0x00, 0x00, 0x01]);
pub const DEVICE_IP_ADDR: wire::Ipv4Address = wire::Ipv4Address::new(169, 254, 0, 7);
pub const DEVICE_CIDR_LENGTH: u8 = 16;

static TIME: AtomicU32 = AtomicU32::new(0);

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut cp = pac::CorePeripherals::take().unwrap();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr
        .sysclk(96.MHz())
        .hclk(96.MHz())
        .freeze();

    unsafe {
        pac::Peripherals::steal()
            .RCC
            .apb1enr
            .modify(|_, w| w.pwren().set_bit());
    }

    // Enable backup domain
    p.PWR.cr.modify(|_, w| w.dbp().set_bit());
    p.PWR.csr.modify(|_, w| w.bre().set_bit());

    defmt::info!("Starting bootloader");

    defmt::info!("Should boot immediately: {}", should_boot_immediately());

    if should_boot_immediately() {
        set_should_boot_immediately(false);
        boot_to_main();
    }

    // Setup SysTick
    cp.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    cp.SYST.set_reload(96_000_000 / 1000 - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    defmt::info!("Flash setup");

    let mut flash = LockedFlash::new(p.FLASH);
    let mut unlocked_flash = flash.unlocked();

    let mut blue_led = gpioc.pc14.into_push_pull_output();

    let mut rx_ring: [RxRingEntry; 2] = [RxRingEntry::INIT; 2];
    let mut tx_ring: [TxRingEntry; 2] = [TxRingEntry::INIT; 2];

    let eth_pins = EthPins {
        ref_clk: gpioa.pa1,
        crs: gpioa.pa7,
        tx_en: gpiob.pb11,
        tx_d0: gpiob.pb12,
        tx_d1: gpiob.pb13,
        rx_d0: gpioc.pc4,
        rx_d1: gpioc.pc5,
    };

    let ethernet_parts = PartsIn {
        dma: p.ETHERNET_DMA,
        mac: p.ETHERNET_MAC,
        mmc: p.ETHERNET_MMC,
        ptp: p.ETHERNET_PTP,
    };

    defmt::info!("Pre ethernet setup");

    let stm32_eth::Parts {
        mut dma,
        mac: _,
        ptp: _,
    } = stm32_eth::new(ethernet_parts, &mut rx_ring, &mut tx_ring, clocks, eth_pins)
        .expect("Failed to intialize STM32 eth");

    defmt::info!("Post ethernet setup");

    dma.enable_interrupt();

    defmt::info!("Post INT enable");

    let mut sockets = [iface::SocketStorage::EMPTY; 1];
    let mut socket_buffers: ethboot_shared::SocketBuffer = SocketBuffer::new();

    let config = iface::Config::new(DEVICE_MAC_ADDR.into());

    let smoltcp_now = || -> smoltcp::time::Instant {
        smoltcp::time::Instant::from_millis(TIME.load(Ordering::Relaxed))
    };

    let mut device = &mut dma;

    let mut bootloader = Bootloader::new(
        config,
        &mut device,
        &mut sockets,
        &mut socket_buffers,
        smoltcp_now(),
    );

    defmt::info!("Post bootloader setup");

    let mut start_time = smoltcp_now().total_millis();

    let mut working_buffer = [0_u8; 512];

    defmt::info!("Entering main loop");

    loop {
        if let Some(time) = smoltcp_now().total_millis().checked_sub(start_time) {
            if time > 1000 {
                // TODO Boot into main program
                defmt::info!("Timeout");
                start_time = smoltcp_now().total_millis();
            }
        }

        if let Ok(action) = bootloader.poll(smoltcp_now(), &mut working_buffer) {
            match action {
                BootloaderAction::None => {},
                BootloaderAction::Ping => {
                    // start_time = timer.now();
                },
                BootloaderAction::EraseFlash { sector } => {
                    unlocked_flash.erase(sector as u8).expect("Failed to erase flash");
                },
                BootloaderAction::ProgramFlash { offset, data } => {
                    unlocked_flash
                        .program(offset as usize, data.iter())
                        .expect("Failed to program flash");
                },
                BootloaderAction::VerifyFlash { start_offset, end_offset, checksum } => {
                    reset_to_boot();
                },
            }
        }

        let led_time = TIME.load(Ordering::Relaxed) % 1000;

        if led_time > 250 {
            blue_led.set_low();
        } else if led_time > 200 {
            blue_led.set_high();
        } else if led_time > 50 {
            blue_led.set_low();
        } else {
            blue_led.set_high();
        }

        // bootloader.poll_interface(smoltcp_now());//.expect("Failed to poll interface");

        // if let Some((source, command)) = bootloader.receive(&mut packet_data).unwrap() {
        //     start_time = timer.now();

        //     match command {
        //         BootloaderCommand::PingBootloader => {
        //             bootloader.send(
        //                 source,
        //                 BootloaderCommand::Response {
        //                     command: BootloaderCommandIndex::PingBootloader as u8,
        //                     success: true,
        //                 },
        //                 &mut packet_data,
        //             ).expect("Failed to send ping response");
        //         },
        //         BootloaderCommand::Response { command: _, success: _ } => {},
        //         BootloaderCommand::EraseFlash { sector } => {
        //             unlocked_flash.erase(sector as u8).expect("Failed to erase flash");
        //         },
        //         BootloaderCommand::ProgramFlash { flash_offset, buffer_offset, buffer_length } => {
        //             let data_iter = packet_data
        //                 .iter()
        //                 .skip(buffer_offset as usize)
        //                 .take(buffer_length as usize);

        //             unlocked_flash.program(flash_offset as usize, data_iter).expect("Failed to program flash");
        //         },
        //         BootloaderCommand::VerifyFlash { checksum: _ } => {

        //         },
        //     }
        // }

        // recv etc.

        // interface.poll(smoltcp_now()).unwrap();

        // let udp_socket = interface.get_socket::<UdpSocket>(udp_socket_handle);
        // if let Ok((size, remote_addr)) = udp_socket.recv_slice(&mut packet_data) {
        //     start_time = timer.now();

        //     if let Some(cmd) = BootloaderCommand::deserialize(&mut packet_data) {
        //         match cmd {
        //             BootloaderCommand::PingBootloader => {

        //             },
        //             _ => {}
        //         }
        //     }
        // }

        // Check if there's network traffic on the bootloader port
        //  - Reset timer (to allow trapping in bootloader)
        //  - Check if the first 4 bytes are some magic number
        //    - Magic number 1: Command packet, just reset timer (like tester present)
        //    - Magic number 2: This is a flash data packet, actually flash something

        // Also flash an led quickly to inform its in bootloader
    }
}

const BKPSRAM_BASE: u32 = 0x4002_4000;

fn should_boot_immediately() -> bool {
    unsafe {
        let ptr = BKPSRAM_BASE as *const u32;
        let val = read_volatile(ptr);

        val == 0xdeadbeef
    }
}

fn set_should_boot_immediately(boot: bool) {
    unsafe {
        let ptr = BKPSRAM_BASE as *mut u32;
        let val = if boot { 0xdeadbeef } else { 0x00000000 };

        ptr.write_volatile(val);
    }
}

fn boot_to_main() {
    // TODO Jump to main
}

fn reset_to_boot() {
    set_should_boot_immediately(true);
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
fn SysTick() {
    TIME.fetch_add(1, Ordering::Relaxed);
}

#[interrupt]
fn ETH() {
    stm32_eth::eth_interrupt_handler();
}
