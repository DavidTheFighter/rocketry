#![no_main]
#![no_std]

mod comms;
mod daq;
mod ecu;
mod peripherals;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use stm32f4xx_hal::{pac, prelude::*};

fn now_fn() -> smoltcp::time::Instant {
    let time = app::monotonics::now().duration_since_epoch().ticks();
    smoltcp::time::Instant::from_millis(time as i64)
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use crate::comms::{eth_interrupt, send_packet};
    use crate::ecu::{ecu_init, ecu_update, ECUControlPins, ECUState};
    use crate::peripherals::{adc_dma, ADCStorage};
    use core::{
        mem::MaybeUninit,
        sync::atomic::{compiler_fence, Ordering},
    };
    use hal::comms_hal::{NetworkAddress, Packet};
    use hal::ecu_hal::ECUDAQFrame;
    use rtic::export::Queue;
    use smoltcp::iface;
    use stm32_eth::{EthPins, EthernetDMA, RxRingEntry, TxRingEntry};
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

    use crate::{
        comms::{init_comms, NetworkingStorage, RX_RING_ENTRY_DEFAULT, TX_RING_ENTRY_DEFAULT},
        daq::DAQHandler,
    };

    const CRYSTAL_FREQ: u32 = 25_000_000;
    const MPU_FREQ: u32 = 150_000_000;
    const PCLK1_FREQ: u32 = 37_500_000;
    const PCLK2_FREQ: u32 = 37_500_000;

    const PACKET_QUEUE_SIZE: usize = 16;

    #[local]
    struct Local {
        blue_led: PE5<Output>,
        ecu_control_pins: ECUControlPins,
        adc: ADCStorage,
        daq: DAQHandler,
        ecu_state: ECUState,
    }

    #[shared]
    struct Shared {
        interface: iface::Interface<'static, &'static mut EthernetDMA<'static, 'static>>,
        udp_socket_handle: iface::SocketHandle,
        current_daq_frame: ECUDAQFrame,
        packet_queue: Queue<Packet, PACKET_QUEUE_SIZE>,
    }

    #[task(local = [blue_led], priority = 1)]
    fn heartbeat_blink_led(ctx: heartbeat_blink_led::Context) {
        heartbeat_blink_led::spawn_after(1000.millis().into()).unwrap();
        ctx.local.blue_led.toggle();
    }

    extern "Rust" {
        #[task(
            shared = [current_daq_frame, packet_queue],
            local = [ecu_state, ecu_control_pins],
            capacity = 8,
            priority = 7,
        )]
        fn ecu_update(mut ctx: ecu_update::Context);

        #[task(
            local = [data: [u8; 512] = [0u8; 512]],
            shared = [interface, udp_socket_handle],
            capacity = 8,
            priority = 12,
        )]
        fn send_packet(ctx: send_packet::Context, packet: Packet, address: NetworkAddress);

        #[task(binds = DMA2_STREAM0,
            local = [daq, adc],
            shared = [current_daq_frame, interface, udp_socket_handle],
            priority = 10
        )]
        fn adc_dma(mut ctx: adc_dma::Context);

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
        rx_ring: [RxRingEntry; 16] = [RX_RING_ENTRY_DEFAULT; 16],
        tx_ring: [TxRingEntry; 16] = [TX_RING_ENTRY_DEFAULT; 16],
        net_storage: NetworkingStorage = NetworkingStorage::new(),
        dma: MaybeUninit<EthernetDMA<'static, 'static>> = MaybeUninit::uninit(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let core = ctx.core;
        let p = ctx.device;

        let rcc = p.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(CRYSTAL_FREQ.Hz())
            .require_pll48clk()
            .sysclk(MPU_FREQ.Hz())
            .hclk(MPU_FREQ.Hz())
            .pclk1(PCLK1_FREQ.Hz())
            .pclk2(PCLK2_FREQ.Hz())
            .freeze();

        let mono = Systick::new(core.SYST, clocks.hclk().raw());

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
        // let adc_in7 = gpiof.pf9.into_analog();
        let adc_in8 = gpiof.pf10.into_analog();

        let mut spark_ctrl = p.TIM1.pwm_hz(spark_ctrl, 250.Hz(), &clocks).split();
        spark_ctrl.disable();
        spark_ctrl.set_duty(0);

        let mut ecu_state = ECUState::default();
        let mut ecu_control_pins = ECUControlPins {
            sv1_ctrl,
            sv2_ctrl,
            sv3_ctrl,
            sv4_ctrl,
            spark_ctrl,
        };

        ecu_init(&mut ecu_state, &mut ecu_control_pins);

        let dma2 = StreamsTuple::new(p.DMA2);

        let adc_config = AdcConfig::default()
            .clock(Clock::Pclk2_div_8)
            .resolution(Resolution::Twelve)
            .dma(Dma::Continuous)
            .scan(Scan::Enabled);

        let mut adc1 = Adc::adc1(p.ADC1, true, adc_config);
        let mut adc2 = Adc::adc2(p.ADC2, false, adc_config);
        let mut adc3 = Adc::adc3(p.ADC3, false, adc_config);

        adc1.enable_temperature_and_vref();
        adc1.configure_channel(&Temperature, Sequence::One, SampleTime::Cycles_480);
        adc1.configure_channel(&adc_in4, Sequence::Two, SampleTime::Cycles_480);

        adc2.configure_channel(&adc_in5, Sequence::One, SampleTime::Cycles_480);
        adc2.configure_channel(&adc_in6, Sequence::Two, SampleTime::Cycles_480);

        adc3.configure_channel(&adc_in3, Sequence::One, SampleTime::Cycles_480);
        adc3.configure_channel(&adc_in8, Sequence::Two, SampleTime::Cycles_480);

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
        ecu_update::spawn().unwrap();

        adc3_transfer.start(|adc| adc.start_conversion());
        adc2_transfer.start(|adc| adc.start_conversion());
        compiler_fence(Ordering::SeqCst);
        adc1_transfer.start(|adc| adc.start_conversion());

        send_packet::spawn(Packet::DeviceBooted, NetworkAddress::MissionControl).unwrap();

        (
            Shared {
                interface,
                udp_socket_handle,
                current_daq_frame: ECUDAQFrame::default(),
                packet_queue: Queue::new(),
            },
            Local {
                blue_led,
                ecu_control_pins,
                adc: ADCStorage {
                    adc1_transfer,
                    adc1_buffer: adc1_buffer2,
                    adc2_transfer,
                    adc2_buffer: adc2_buffer2,
                    adc3_transfer,
                    adc3_buffer: adc3_buffer2,
                },
                daq: DAQHandler::new(),
                ecu_state,
            },
            init::Monotonics(mono),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
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
