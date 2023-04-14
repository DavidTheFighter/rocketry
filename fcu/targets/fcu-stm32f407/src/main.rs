#![no_main]
#![no_std]

mod comms;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use stm32f4xx_hal::{pac, prelude::*};

pub(crate) fn now() -> u64 {
    app::monotonics::now().duration_since_epoch().ticks()
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use core::{
        mem::MaybeUninit,
        sync::atomic::{compiler_fence, AtomicU32, Ordering},
    };
    use cortex_m::peripheral::DWT;
    use hal::comms_hal::{NetworkAddress, Packet};
    use rtic::export::Queue;
    use smoltcp::iface;
    use stm32_eth::{EthPins, EthernetDMA, RxRingEntry, TxRingEntry};
    use stm32f4xx_hal::{
        gpio::{Output, PA10},
        prelude::*,
    };
    use systick_monotonic::Systick;

    use crate::comms::{send_packet, eth_interrupt, init_comms, NetworkingStorage, RX_RING_ENTRY_DEFAULT, TX_RING_ENTRY_DEFAULT};

    const CRYSTAL_FREQ: u32 = 25_000_000;
    const MCU_FREQ: u32 = 37_500_000;
    const PCLK1_FREQ: u32 = 37_500_000;
    const PCLK2_FREQ: u32 = 37_500_000;

    const CPU_USAGE_RATE_MS: u64 = 250;
    const PACKET_QUEUE_SIZE: usize = 16;

    #[local]
    struct Local {
        blue_led: PA10<Output>,
        dwt: DWT,
    }

    #[shared]
    struct Shared {
        interface: iface::Interface<'static, &'static mut EthernetDMA<'static, 'static>>,
        udp_socket_handle: iface::SocketHandle,
        packet_queue: Queue<Packet, PACKET_QUEUE_SIZE>,
        cpu_utilization: AtomicU32,
    }

    #[task(local = [blue_led], priority = 1)]
    fn heartbeat_blink_led(ctx: heartbeat_blink_led::Context) {
        heartbeat_blink_led::spawn_after(1000.millis().into()).unwrap();
        ctx.local.blue_led.toggle();
    }

    extern "Rust" {
        #[task(
            local = [data: [u8; 512] = [0u8; 512]],
            shared = [interface, udp_socket_handle],
            capacity = 8,
            priority = 12,
        )]
        fn send_packet(ctx: send_packet::Context, packet: Packet, address: NetworkAddress);

        #[task(
            binds = ETH,
            local = [data: [u8; 512] = [0u8; 512]],
            shared = [interface, udp_socket_handle, packet_queue],
            priority = 12,
        )]
        fn eth_interrupt(ctx: eth_interrupt::Context);
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<1000>;

    #[init(local = [
        rx_ring: [RxRingEntry; 4] = [RX_RING_ENTRY_DEFAULT; 4],
        tx_ring: [TxRingEntry; 4] = [TX_RING_ENTRY_DEFAULT; 4],
        net_storage: NetworkingStorage = NetworkingStorage::new(),
        dma: MaybeUninit<EthernetDMA<'static, 'static>> = MaybeUninit::uninit(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut core = ctx.core;
        let p = ctx.device;

        let rcc = p.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(CRYSTAL_FREQ.Hz())
            .require_pll48clk()
            .sysclk(MCU_FREQ.Hz())
            .hclk(MCU_FREQ.Hz())
            .pclk1(PCLK1_FREQ.Hz())
            .pclk2(PCLK2_FREQ.Hz())
            .freeze();

        let mono = Systick::new(core.SYST, clocks.hclk().raw());

        core.DWT.enable_cycle_counter();

        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();
        let gpioc = p.GPIOC.split();

        let blue_led = gpioa.pa10.into_push_pull_output();

        let eth_pins = EthPins {
            ref_clk: gpioa.pa1,
            crs: gpioa.pa7,
            tx_en: gpiob.pb11,
            tx_d0: gpiob.pb12,
            tx_d1: gpiob.pb13,
            rx_d0: gpioc.pc4,
            rx_d1: gpioc.pc5,
        };
        let (eth_dma, _eth_mac) = stm32_eth::new(
            p.ETHERNET_MAC,
            p.ETHERNET_MMC,
            p.ETHERNET_DMA,
            ctx.local.rx_ring,
            ctx.local.tx_ring,
            clocks,
            eth_pins,
        )
        .unwrap();

        let eth_dma = ctx.local.dma.write(eth_dma);
        eth_dma.enable_interrupt();

        let (interface, udp_socket_handle) = init_comms(ctx.local.net_storage, eth_dma);

        heartbeat_blink_led::spawn().unwrap();

        send_packet::spawn(Packet::DeviceBooted, NetworkAddress::MissionControl).unwrap();

        (
            Shared {
                interface,
                udp_socket_handle,
                packet_queue: Queue::new(),
                cpu_utilization: AtomicU32::new(0),
            },
            Local {
                blue_led,
                dwt: core.DWT,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [dwt], shared = [&cpu_utilization])]
    fn idle(ctx: idle::Context) -> ! {
        let mut last_report_time = crate::now();
        let mut accum_cycles = 0;

        loop {
            rtic::export::interrupt::free(|_cs| {
                let before = ctx.local.dwt.cyccnt.read();
                compiler_fence(Ordering::SeqCst);
                rtic::export::wfi();
                compiler_fence(Ordering::SeqCst);
                let after = ctx.local.dwt.cyccnt.read();

                let elapsed = after.wrapping_sub(before);
                accum_cycles += elapsed;

                let current_time = crate::now();
                if current_time - last_report_time >= CPU_USAGE_RATE_MS {
                    let total_cycles = ((current_time - last_report_time) as u32) * (MCU_FREQ / 1000);
                    let cpu_util = (100 * (total_cycles - accum_cycles)) / total_cycles;

                    ctx.shared.cpu_utilization.store(cpu_util, Ordering::Relaxed);
                    last_report_time = current_time;
                    accum_cycles = 0;
                }
            });
        }
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:?}", ef);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let dp = unsafe { pac::Peripherals::steal() };
    let gpioe = dp.GPIOE.split();
    let mut led = gpioe.pe5.into_push_pull_output();

    loop {
        led.set_low();
        cortex_m::asm::delay(3_000_000);
        led.set_high();
        cortex_m::asm::delay(3_000_000);
    }
}
