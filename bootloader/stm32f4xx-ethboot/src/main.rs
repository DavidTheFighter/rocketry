#![no_std]
#![no_main]

use core::{
    ptr::read_volatile,
    sync::atomic::{AtomicU32, Ordering},
};
use defmt_brtt as _;

use ethboot_shared::{Bootloader, BootloaderAction, SocketBuffer};
use hal::{
    flash::{FlashExt, LockedFlash},
    interrupt,
};
use panic_halt as _;

use cortex_m_rt::{entry, exception};
use smoltcp::{
    iface,
    wire::{self, EthernetAddress},
};
use stm32_eth::{
    dma::{RxRingEntry, TxRingEntry},
    EthPins, PartsIn,
};
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

    unsafe {
        pac::Peripherals::steal()
            .RCC
            .apb1enr
            .modify(|_, w| w.pwren().set_bit());
    }

    // Enable backup domain
    p.PWR.cr.modify(|_, w| w.dbp().set_bit());
    p.PWR.csr.modify(|_, w| w.bre().set_bit());

    unsafe {
        pac::Peripherals::steal()
            .RCC
            .ahb1enr
            .modify(|_, w| w.bkpsramen().set_bit());
    }

    if should_boot_immediately() {
        defmt::info!("Booting immediately");
        set_should_boot_immediately(false);
        boot_to_main();
    }

    let clocks = rcc.cfgr.sysclk(96.MHz()).hclk(96.MHz()).freeze();

    defmt::info!("Starting bootloader");

    // Setup SysTick
    cp.SYST
        .set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    cp.SYST.set_reload(96_000_000 / 1000 - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let mut flash = LockedFlash::new(p.FLASH);

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

    let stm32_eth::Parts {
        mut dma,
        mac: _,
        ptp: _,
    } = stm32_eth::new(ethernet_parts, &mut rx_ring, &mut tx_ring, clocks, eth_pins)
        .expect("Failed to intialize STM32 eth");

    dma.enable_interrupt();

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

    let mut start_time = smoltcp_now().total_millis();
    let mut allow_timeout = true;

    let mut working_buffer = [0_u8; 512];

    loop {
        if let Some(time) = smoltcp_now().total_millis().checked_sub(start_time) {
            if allow_timeout && time > 3000 {
                reset_to_boot();
            }
        }

        if let Ok(action) = bootloader.poll(smoltcp_now(), &mut working_buffer) {
            match action {
                BootloaderAction::None => {}
                BootloaderAction::Ping => {
                    start_time = smoltcp_now().total_millis();
                    defmt::info!("Ping");
                }
                BootloaderAction::EraseFlash { sector } => {
                    start_time = smoltcp_now().total_millis();
                    allow_timeout = false;
                    defmt::info!("Erase flash sector {}", sector);
                    flash
                        .unlocked()
                        .erase(sector as u8)
                        .expect("Failed to erase flash");
                    defmt::info!("Done erasing!");
                    bootloader.complete_action(&mut working_buffer);
                }
                BootloaderAction::ProgramFlash { offset, data } => {
                    start_time = smoltcp_now().total_millis();
                    allow_timeout = false;
                    defmt::info!(
                        "Program flash at offset {} with {} bytes",
                        offset,
                        data.len()
                    );
                    flash
                        .unlocked()
                        .program(offset as usize, data.iter())
                        .expect("Failed to program flash");

                    let current_flash = flash.read();

                    for (i, byte) in data.iter().enumerate() {
                        let flash_byte = current_flash[offset as usize + i];
                        if flash_byte != *byte {
                            panic!(
                                "Flash mismatch at offset {}: {} != {}",
                                offset + i as u32,
                                flash_byte,
                                byte
                            );
                        }
                    }

                    bootloader.complete_action(&mut working_buffer);
                }
                BootloaderAction::VerifyFlash {
                    start_offset,
                    end_offset,
                    checksum,
                } => {
                    start_time = smoltcp_now().total_millis();
                    allow_timeout = false;
                    defmt::info!(
                        "Verify flash from {} to {} with checksum {}",
                        start_offset,
                        end_offset,
                        checksum
                    );

                    let current_flash = flash.read();
                    let mut current_checksum = 0;

                    for i in start_offset..end_offset {
                        current_checksum += current_flash[i as usize] as u128;
                    }

                    defmt::info!("Checksum: {} vs {}", current_checksum, checksum);

                    if current_checksum == checksum {
                        bootloader.complete_action(&mut working_buffer);
                        reset_to_boot();
                    }
                }
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
    }
}

const BKPSRAM_BASE: u32 = 0x4002_4000;

fn should_boot_immediately() -> bool {
    unsafe {
        let ptr = BKPSRAM_BASE as *const u32;
        let val = read_volatile(ptr);

        defmt::info!("Should boot immediately: {}", val);

        val == 0xdeadbeef
    }
}

fn set_should_boot_immediately(boot: bool) {
    unsafe {
        let ptr = BKPSRAM_BASE as *mut u32;
        let val = if boot { 0xdeadbeef } else { 0x00000000 };

        ptr.write_volatile(val);
    }

    defmt::info!("Writing boot immediately: {}", boot);
    defmt::info!("Read boot immediately: {}", should_boot_immediately());
}

fn boot_to_main() {
    let vector_table = 0x0801_0000 as *const u32;
    unsafe {
        let p = cortex_m::Peripherals::steal();

        p.SCB.vtor.write(vector_table as u32);

        cortex_m::asm::bootload(vector_table);
    }
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
