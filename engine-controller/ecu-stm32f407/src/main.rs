#![no_main]
#![no_std]

pub mod ecu_driver;
mod peripherals;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use stm32f4xx_hal::{pac, prelude::*};

pub(crate) fn now() -> u64 {
    app::monotonics::now().duration_since_epoch().ticks()
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use crate::peripherals::{adc_dma, ADCStorage};
    use crate::ecu_driver::{Stm32F407EcuDriver, EcuControlPins, ecu_update};
    use core::{
        mem::MaybeUninit,
        sync::atomic::{compiler_fence, AtomicU32, Ordering},
    };
    use big_brother::interface::smoltcp_interface::{SmoltcpInterface, SmoltcpInterfaceStorage};
    use cortex_m::peripheral::DWT;
    use ecu_rs::ecu::EcuBigBrother;
    use ecu_rs::Ecu;
    use rand_core::RngCore;
    use shared::comms_hal::{NetworkAddress, Packet};
    use rtic::export::Queue;
    use stm32_eth::EthPins;
    use stm32f4xx_hal::{
        adc::{
            config::{AdcConfig, Clock, Dma, Resolution, SampleTime, Scan, Sequence},
            Adc, Temperature,
        },
        dma::{config::DmaConfig, StreamsTuple, Transfer},
        gpio::{Output, PE5},
        prelude::*,
    };
    use systick_monotonic::Systick;

    const CRYSTAL_FREQ: u32 = 25_000_000;
    const MCU_FREQ: u32 = 37_500_000;
    const PCLK1_FREQ: u32 = 37_500_000;
    const PCLK2_FREQ: u32 = 37_500_000;

    const CPU_USAGE_RATE_MS: u64 = 250;
    const PACKET_QUEUE_SIZE: usize = 16;

    #[local]
    struct Local {
        blue_led: PE5<Output>,
        adc: ADCStorage,
        dwt: DWT,
    }

    #[shared]
    struct Shared {
        cpu_utilization: AtomicU32,
        ecu: ecu_rs::Ecu<'static>,
    }

    #[task(local = [blue_led], priority = 1)]
    fn heartbeat_blink_led(ctx: heartbeat_blink_led::Context) {
        heartbeat_blink_led::spawn_after(1000.millis().into()).unwrap();
        ctx.local.blue_led.toggle();
    }

    #[task(
        binds = ETH,
        shared = [ecu],
        priority = 12,
    )]
    fn eth_interrupt(mut ctx: eth_interrupt::Context) {
        stm32_eth::eth_interrupt_handler();

        ctx.shared.ecu.lock(|ecu| {
            ecu.poll_interfaces();
        });
    }

    extern "Rust" {
        #[task(
            shared = [ecu, &cpu_utilization],
            local = [],
            capacity = 8,
            priority = 7,
        )]
        fn ecu_update(mut ctx: ecu_update::Context);

        #[task(binds = DMA2_STREAM0,
            local = [adc],
            shared = [ecu],
            priority = 10
        )]
        fn adc_dma(mut ctx: adc_dma::Context);
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<1000>;

    #[init(local = [
        eth_device: MaybeUninit<stm32_eth::dma::EthernetDMA<'static,'static> > = MaybeUninit::uninit(),
        rx_ring: [stm32_eth::dma::RxRingEntry; 4] = [stm32_eth::dma::RxRingEntry::INIT; 4],
        tx_ring: [stm32_eth::dma::TxRingEntry; 4] = [stm32_eth::dma::TxRingEntry::INIT; 4],
        smoltcp_interface_storage: SmoltcpInterfaceStorage<'static> = SmoltcpInterfaceStorage::new(),
        smoltcp_interface: MaybeUninit<SmoltcpInterface<'static, &'static mut stm32_eth::dma::EthernetDMA<'static, 'static>>> = MaybeUninit::uninit(),
        big_brother: MaybeUninit<EcuBigBrother<'static>> = MaybeUninit::uninit(),
        ecu_driver: MaybeUninit<Stm32F407EcuDriver> = MaybeUninit::uninit(),
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
        let gpioe = p.GPIOE.split();
        let gpiof = p.GPIOF.split();

        let blue_led = gpioe.pe5.into_push_pull_output();
        let spark_ctrl = gpioe.pe9.into_alternate().internal_pull_down(true);
        let sv1_ctrl = gpioa.pa12.into_push_pull_output();
        let sv2_ctrl = gpioa.pa11.into_push_pull_output();
        let sv3_ctrl = gpioa.pa10.into_push_pull_output();
        let sv4_ctrl = gpioa.pa9.into_push_pull_output();
        let adc_in3 = gpioa.pa3.into_analog();
        let adc_in4 = gpioa.pa4.into_analog();
        let adc_in5 = gpioa.pa5.into_analog();
        let adc_in6 = gpioa.pa6.into_analog();
        let adc_in7 = gpiof.pf9.into_analog();
        // let adc_in8 = gpiof.pf10.into_analog();

        let mut spark_ctrl = p.TIM1.pwm_hz(spark_ctrl, 400.Hz(), &clocks).split();
        spark_ctrl.disable();
        spark_ctrl.set_duty(0);

        let ecu_control_pins = EcuControlPins {
            sv1_ctrl,
            sv2_ctrl,
            sv3_ctrl,
            sv4_ctrl,
            spark_ctrl,
        };

        let dma2 = StreamsTuple::new(p.DMA2);

        let adc_config = AdcConfig::default()
            .clock(Clock::Pclk2_div_6)
            .resolution(Resolution::Twelve)
            .dma(Dma::Continuous)
            .scan(Scan::Enabled);

        let mut adc1 = Adc::adc1(p.ADC1, true, adc_config);
        let mut adc2 = Adc::adc2(p.ADC2, false, adc_config);
        let mut adc3 = Adc::adc3(p.ADC3, false, adc_config);

        adc1.configure_channel(&Temperature, Sequence::One, SampleTime::Cycles_480);
        adc1.configure_channel(&adc_in4, Sequence::Two, SampleTime::Cycles_480);
        adc1.enable_temperature_and_vref();

        adc2.configure_channel(&adc_in5, Sequence::One, SampleTime::Cycles_480);
        adc2.configure_channel(&adc_in6, Sequence::Two, SampleTime::Cycles_480);

        adc3.configure_channel(&adc_in3, Sequence::One, SampleTime::Cycles_480);
        adc3.configure_channel(&adc_in7, Sequence::Two, SampleTime::Cycles_480);

        let adc1_buffer1 = cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap();
        let adc1_buffer2 = Some(cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap());

        let adc2_buffer1 = cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap();
        let adc2_buffer2 = Some(cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap());

        let adc3_buffer1 = cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap();
        let adc3_buffer2 = Some(cortex_m::singleton!(: [u16; 2] = [0;2]).unwrap());

        let dma_config = DmaConfig::default()
            .transfer_complete_interrupt(true)
            .memory_increment(true)
            .double_buffer(false);

        let mut adc1_transfer =
            Transfer::init_peripheral_to_memory(dma2.0, adc1, adc1_buffer1, None, dma_config);

        let mut adc2_transfer =
            Transfer::init_peripheral_to_memory(dma2.2, adc2, adc2_buffer1, None, dma_config);

        let mut adc3_transfer =
            Transfer::init_peripheral_to_memory(dma2.1, adc3, adc3_buffer1, None, dma_config);

        let eth_pins = stm32_eth::EthPins {
            ref_clk: gpioa.pa1,
            crs: gpioa.pa7,
            tx_en: gpiob.pb11,
            tx_d0: gpiob.pb12,
            tx_d1: gpiob.pb13,
            rx_d0: gpioc.pc4,
            rx_d1: gpioc.pc5,
        };

        let ethernet_parts = stm32_eth::PartsIn {
            dma: p.ETHERNET_DMA,
            mac: p.ETHERNET_MAC,
            mmc: p.ETHERNET_MMC,
            ptp: p.ETHERNET_PTP,
        };

        let stm32_eth::Parts {
            dma: eth_device_dma,
            mac: _,
            ptp: _,
        } = stm32_eth::new(ethernet_parts, ctx.local.rx_ring, ctx.local.tx_ring, clocks, eth_pins)
            .expect("Failed to intialize STM32 eth");

        eth_device_dma.enable_interrupt();

        heartbeat_blink_led::spawn().unwrap();
        ecu_update::spawn().unwrap();

        adc3_transfer.start(|adc| adc.start_conversion());
        adc2_transfer.start(|adc| adc.start_conversion());
        compiler_fence(Ordering::SeqCst);
        adc1_transfer.start(|adc| adc.start_conversion());

        let ecu_driver = ctx.local.ecu_driver.write(
            Stm32F407EcuDriver::new(ecu_control_pins),
        );

        let eth_device = ctx.local.eth_device.write(eth_device_dma);

        let smoltcp_interface = ctx.local.smoltcp_interface.write(
            SmoltcpInterface::new(
                [0x00, 0x80, 0xE1, 0x00, 0x00, 0x01],
                eth_device,
                [169, 254, 0, 6],
                [169, 254, 255, 255],
                16,
                ctx.local.smoltcp_interface_storage,
                0,
            ),
        );

        let mut rand_source = p.RNG.constrain(&clocks);

        let big_brother = ctx.local.big_brother.write(
            EcuBigBrother::new(
                NetworkAddress::EngineController(0),
                rand_source.next_u32(),
                NetworkAddress::Broadcast,
                [Some(smoltcp_interface), None],
            ),
        );

        let ecu = Ecu::new(ecu_driver, big_brother);

        (
            Shared {
                cpu_utilization: AtomicU32::new(0),
                ecu,
            },
            Local {
                blue_led,
                adc: ADCStorage {
                    adc1_transfer,
                    adc1_buffer: adc1_buffer2,
                    adc2_transfer,
                    adc2_buffer: adc2_buffer2,
                    adc3_transfer,
                    adc3_buffer: adc3_buffer2,
                },
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
