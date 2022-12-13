#![no_main]
#![no_std]

mod comms;
mod daq;
// mod periphals;

use core::panic::PanicInfo;
use cortex_m_rt::{ExceptionFrame, exception};
use daq::DAQFrame;
use stm32f4xx_hal::{pac, prelude::*};

fn now_fn() -> smoltcp::time::Instant {
    let time = app::monotonics::now().duration_since_epoch().ticks();
    smoltcp::time::Instant::from_millis(time as i64)
}

pub enum ECUPacket {
    FireEngine,
    DAQDataFrame { daq_frames: [DAQFrame; 10] }
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use stm32f4xx_hal::{
        prelude::*,
        pac::{ADC1, ADC2, ADC3, DMA2},
        gpio::{Output, PA12, PE5, PE9},
        adc::{
            Adc,
            config::{AdcConfig, Resolution, Clock, Dma, Scan, Sequence, SampleTime},
        }, 
        dma::{Transfer, Stream0, Stream1, Stream2, StreamsTuple, PeripheralToMemory, config::DmaConfig},
    };
    use stm32_eth::{
        RxRingEntry,
        TxRingEntry,
        EthPins,
        EthernetDMA,
    };
    use smoltcp::{iface, wire, socket::UdpSocket};
    use systick_monotonic::Systick;
    use core::sync::atomic::compiler_fence;
    use core::sync::atomic::Ordering;

    use crate::{comms::{
        NetworkingStorage, 
        RX_RING_ENTRY_DEFAULT, 
        TX_RING_ENTRY_DEFAULT, 
        init_comms,
    }, daq::DAQHandler};
    use crate::now_fn;

    const CRYSTAL_FREQ: u32 = 25_000_000;
    const MPU_FREQ: u32 = 150_000_000;
    const PCLK1_FREQ: u32 = 37_500_000;
    const PCLK2_FREQ: u32 = 37_500_000;

    #[local]
    struct Local {
        blue_led: PE5<Output>,
        spark_ctrl: PE9<Output>,
        sv1_ctrl: PA12<Output>,
        adc1_transfer: Transfer<Stream0<DMA2>, 0, Adc<ADC1>, PeripheralToMemory, &'static mut [u16; 2]>,
        adc1_buffer: Option<&'static mut [u16; 2]>,
        adc2_transfer: Transfer<Stream2<DMA2>, 1, Adc<ADC2>, PeripheralToMemory, &'static mut [u16; 2]>,
        adc2_buffer: Option<&'static mut [u16; 2]>,
        adc3_transfer: Transfer<Stream1<DMA2>, 2, Adc<ADC3>, PeripheralToMemory, &'static mut [u16; 2]>,
        adc3_buffer: Option<&'static mut [u16; 2]>,
    }

    #[shared]
    struct Shared {
        interface: iface::Interface<'static, &'static mut EthernetDMA<'static, 'static>>,
        udp_socket_handle: iface::SocketHandle,

        #[lock_free]
        daq: DAQHandler,
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<1000>;

    #[init(local = [
        rx_ring: [RxRingEntry; 16] = [RX_RING_ENTRY_DEFAULT; 16],
        tx_ring: [TxRingEntry; 16] = [TX_RING_ENTRY_DEFAULT; 16],
        net_storage: NetworkingStorage = NetworkingStorage::new(),
        dma: core::mem::MaybeUninit<EthernetDMA<'static, 'static>> = core::mem::MaybeUninit::uninit(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let core = ctx.core;
        let p = ctx.device;

        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr
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
        let spark_ctrl = gpioe.pe9.into_push_pull_output();
        let sv1_ctrl = gpioa.pa12.into_push_pull_output();
        let adc1_in3 = gpioa.pa3.into_analog();
        let adc1_in4 = gpioa.pa4.into_analog();
        let adc2_in5 = gpioa.pa5.into_analog();
        let adc2_in6 = gpioa.pa6.into_analog();
        let adc3_in7 = gpiof.pf9.into_analog();
        let adc3_in8 = gpiof.pf10.into_analog();

        let dma2 = StreamsTuple::new(p.DMA2);

        let adc_config = AdcConfig::default()
            .clock(Clock::Pclk2_div_8)
            .resolution(Resolution::Twelve)
            .dma(Dma::Continuous)
            .scan(Scan::Enabled);

        let mut adc1 = Adc::adc1(p.ADC1, true, adc_config);
        adc1.configure_channel(&adc1_in3, Sequence::One, SampleTime::Cycles_480);
        adc1.configure_channel(&adc1_in4, Sequence::Two, SampleTime::Cycles_480);

        let mut adc2 = Adc::adc2(p.ADC2, true, adc_config);
        adc2.configure_channel(&adc2_in5, Sequence::One, SampleTime::Cycles_480);
        adc2.configure_channel(&adc2_in6, Sequence::Two, SampleTime::Cycles_480);

        let mut adc3 = Adc::adc3(p.ADC3, true, adc_config);
        adc3.configure_channel(&adc3_in7, Sequence::One, SampleTime::Cycles_480);
        adc3.configure_channel(&adc3_in8, Sequence::Two, SampleTime::Cycles_480);

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

        let mut adc1_transfer = Transfer::init_peripheral_to_memory(
            dma2.0, 
            adc1, 
            adc1_buffer1, 
            None, 
            dma_config,
        );

        let mut adc2_transfer = Transfer::init_peripheral_to_memory(
            dma2.2, 
            adc2, 
            adc2_buffer1, 
            None, 
            dma_config,
        );

        let mut adc3_transfer = Transfer::init_peripheral_to_memory(
            dma2.1, 
            adc3, 
            adc3_buffer1, 
            None, 
            dma_config,
        );

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
        spark_toggle::spawn().unwrap();

        adc1_transfer.start(|adc| adc.start_conversion());
        adc2_transfer.start(|adc| adc.start_conversion());
        compiler_fence(Ordering::SeqCst);
        adc3_transfer.start(|adc| adc.start_conversion());

        (
            Shared {
                interface,
                udp_socket_handle,
                daq: DAQHandler::new(),
            },
            Local {
                blue_led,
                spark_ctrl,
                sv1_ctrl,
                adc1_transfer,
                adc1_buffer: adc1_buffer2,
                adc2_transfer,
                adc2_buffer: adc2_buffer2,
                adc3_transfer,
                adc3_buffer: adc3_buffer2,
            },
            init::Monotonics(mono)
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(local = [blue_led, sv1_ctrl], priority=1)]
    fn heartbeat_blink_led(ctx: heartbeat_blink_led::Context) {
        heartbeat_blink_led::spawn_after(1000.millis().into()).unwrap();
        ctx.local.blue_led.toggle();
        ctx.local.sv1_ctrl.toggle();
    }

    #[task(local = [spark_ctrl])]
    fn spark_toggle(ctx: spark_toggle::Context) {
        spark_toggle::spawn_after(2.millis().into()).unwrap();
        ctx.local.spark_ctrl.toggle();
    }

    #[task(shared = [interface, udp_socket_handle])]
    fn udp_send_slice(ctx: udp_send_slice::Context, packet: crate::ECUPacket) {
        
    }

    #[task(binds = DMA2_STREAM1, 
        local = [adc1_transfer, adc1_buffer, adc2_transfer, adc2_buffer, adc3_transfer, adc3_buffer], 
        shared = [daq, interface, udp_socket_handle], 
        priority=13
    )]
    fn dma(ctx: dma::Context) {
        let dma::Context { mut shared, local } = ctx;

        let adc1_buffer = local.adc1_transfer
            .next_transfer(local.adc1_buffer.take().unwrap())
            .unwrap().0;

        let adc2_buffer = local.adc2_transfer
            .next_transfer(local.adc2_buffer.take().unwrap())
            .unwrap().0;

        let adc3_buffer = local.adc3_transfer
            .next_transfer(local.adc3_buffer.take().unwrap())
            .unwrap().0;

        let daq_frame = crate::daq::DAQFrame {
            adc1_in3: adc1_buffer[0],
            adc1_in4: adc1_buffer[1],
            adc2_in5: adc2_buffer[0],
            adc2_in6: adc2_buffer[1],
            adc3_in7: adc3_buffer[0],
            adc3_in8: adc3_buffer[1],
        };

        *local.adc1_buffer = Some(adc1_buffer);
        *local.adc2_buffer = Some(adc2_buffer);
        *local.adc3_buffer = Some(adc3_buffer);

        // let iface = shared.interface;
        // let udp = shared.udp_socket_handle;

        // (iface, udp).lock(|iface, udp_handle| {
        //     let udp_socket = iface.get_socket::<UdpSocket>(*udp_handle);

        //     if udp_socket.can_send() {
        //         let data = [adc1_in3, adc1_in4, adc2_in5, adc2_in6, adc3_in7, adc3_in8];
        //         let (prefix, data, suffix) = unsafe { data.align_to::<u8>() };
        //         assert!(prefix.is_empty() && suffix.is_empty() &&
        //                 core::mem::align_of::<u8>() <= core::mem::align_of::<u16>(),
        //                 "Expected u8 alignment to be no stricter than u16 alignment");

        //         let ip_addr = wire::Ipv4Address::new(169, 254, 0, 5);
        //         let endpoint = wire::IpEndpoint::new(ip_addr.into(), 25565);

        //         udp_socket.send_slice(data, endpoint).unwrap();
        //         iface.poll(now_fn()).ok();
        //     }
        // });

        if shared.daq.add_daq_frame(daq_frame) {

        }

        local.adc1_transfer.start(|adc| adc.start_conversion());
        local.adc2_transfer.start(|adc| adc.start_conversion());
        compiler_fence(Ordering::SeqCst);
        local.adc3_transfer.start(|adc| adc.start_conversion());
    }

    #[task(
        binds = ETH, 
        local = [data: [u8; 512] = [0u8; 512]],
        shared = [interface, udp_socket_handle],
        priority = 12,
    )]
    fn eth_interrupt(ctx: eth_interrupt::Context) {
        let iface = ctx.shared.interface;
        let udp = ctx.shared.udp_socket_handle;

        (iface, udp).lock(|iface, udp_handle| {
            iface.device_mut().interrupt_handler();
            iface.poll(now_fn()).ok();

            let buffer = ctx.local.data;
            let udp_socket = iface.get_socket::<UdpSocket>(*udp_handle);

            if let Ok((_recv_bytes, sender)) = udp_socket.recv_slice(buffer) {
                let new_data = [buffer[0] + 1];
                        
                if udp_socket.can_send() {
                    udp_socket.send_slice(&new_data, sender).unwrap();
                }
            }

            iface.poll(now_fn()).ok();
        });
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