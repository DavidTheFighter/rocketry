#![no_main]
#![no_std]

mod drivers;
// mod comms;
mod fcu_driver;
mod sensors;
mod logging;

use defmt_brtt as _;

use core::panic::PanicInfo;
use cortex_m_rt::{exception, ExceptionFrame};
use stm32f4xx_hal::{pac, prelude::*};

pub(crate) fn now() -> u64 {
    app::monotonics::now().duration_since_epoch().ticks()
}

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [CAN2_TX, CAN2_RX0, CAN2_RX1, CAN2_SCE, USART6])]
mod app {
    use core::{
        mem::MaybeUninit,
        sync::atomic::{compiler_fence, AtomicU32, Ordering},
        cell::RefCell,
    };
    use bmi088_rs::{AccelFilterBandwidth, AccelDataRate, AccelRange, GyroRange, GyroBandwidth, Bmi088Accelerometer, Bmi088Gyroscope, Bmi088PinMode, Bmi088PinBehavior};
    use big_brother::interface::smoltcp_interface::{SmoltcpInterface, SmoltcpInterfaceStorage};
    use cortex_m::peripheral::DWT;
    use cortex_m::interrupt::Mutex;
    use flight_controller_rs::{Fcu, FcuBigBrother};
    use shared::comms_hal::{NetworkAddress, Packet};
    use stm32f4::stm32f407::{I2C1, DMA2, ADC1};
    use stm32f4xx_hal::{
        gpio::{self, Input, Output, PC3, PC14, PC15, PE4, PE5, PE8, PE9, Edge, Pin, Alternate, PinState},
        prelude::*,
        spi,
        pac::{SPI1, USART2, UART4},
        serial::{self, Serial}, dma::{Channel1, Transfer, StreamsTuple, config::DmaConfig, self}, adc::{Adc, config::{AdcConfig, Sequence, SampleTime, Dma, Scan}},
    };
    use systick_monotonic::Systick;
    use rand_core::RngCore;

    use crate::fcu_driver::{Stm32F407FcuDriver, FcuControlPins, fcu_update};
    use crate::drivers::{bmm150, w25x05};
    // use crate::comms::{send_packet, eth_interrupt, init_comms, NetworkingStorage};
    use crate::sensors::{bmi088_interrupt, bmm150_interrupt, ms5611_update, ublox_update, adc1_dma2_stream0_interrupt};
    use crate::logging::{self, DataLoggerType};

    const CRYSTAL_FREQ: u32 = 25_000_000;
    pub const MCU_FREQ: u32 = 75_000_000;
    const PCLK1_FREQ: u32 = 37_500_000;
    const PCLK2_FREQ: u32 = 37_500_000;

    const CPU_USAGE_RATE_MS: u64 = 250;
    const PACKET_QUEUE_SIZE: usize = 16;

    type LogSpi1 = spi::Spi<SPI1, (Pin<'A', 5, Alternate<5>>, Pin<'B', 4, Alternate<5>>, Pin<'B', 5, Alternate<5>>), false>;
    // type Usart2Type = Serial<USART2, (Pin<'D', 5, Alternate<7>>, Pin<'D', 6, Alternate<7>>)>;
    type I2C1Type = stm32f4xx_hal::i2c::I2c<I2C1, (Pin<'B', 6, Alternate<4, gpio::OpenDrain>>, Pin<'B', 7, Alternate<4, gpio::OpenDrain>>)>;
    type I2C1BusType = shared_bus::BusManager<Mutex<RefCell<I2C1Type>>>;
    type SharedI2C1Type = shared_bus::I2cProxy<'static, Mutex<RefCell<I2C1Type>>>;

    #[local]
    struct Local {
        blue_led: PC14<Output>,
        bmi088_accel: Bmi088Accelerometer<SharedI2C1Type>,
        bmi088_gyro: Bmi088Gyroscope<SharedI2C1Type>,
        ms5611: ms5611_rs::Ms5611<SharedI2C1Type>,
        bmm150: bmm150::Bmm150,
        accel_int_pin: PE8<Input>,
        gyro_int_pin: PE9<Input>,
        mag_int_pin: PC3<Input>,
        usart2_tx: serial::Tx<USART2>,
        usart2_rx: serial::Rx<USART2>,
        uart4: Serial<UART4, (Pin<'C', 10, Alternate<8>>, Pin<'C', 11, Alternate<8>>)>,
        adc1_transfer: Transfer<dma::StreamX<DMA2, 0>, 0, Adc<ADC1>, dma::PeripheralToMemory, &'static mut [u16; 8]>,
        adc1_other_buffer: Option<&'static mut [u16; 8]>,
        dwt: DWT,
    }

    #[shared]
    struct Shared {
        red_led: PC15<Output>,
        fcu: Fcu<'static>,
        #[lock_free]
        w25x05: w25x05::W25X05<PE4<Output>, PE5<Output>>,
        #[lock_free]
        spi1: LogSpi1,
        cpu_utilization: AtomicU32,
    }

    #[task(local = [blue_led], priority = 1)]
    fn heartbeat_blink_led(ctx: heartbeat_blink_led::Context) {
        heartbeat_blink_led::spawn_after(1000.millis().into()).unwrap();
        ctx.local.blue_led.toggle();
    }

    #[task(
        binds = ETH,
        shared = [fcu],
        priority = 12,
    )]
    fn eth_interrupt(mut ctx: eth_interrupt::Context) {
        stm32_eth::eth_interrupt_handler();

        ctx.shared.fcu.lock(|fcu| {
            fcu.poll_interfaces();
        });
    }

    extern "Rust" {
        #[task(
            shared = [fcu, &cpu_utilization],
            local = [adc1_transfer, adc1_other_buffer],
            priority = 7,
        )]
        fn fcu_update(mut ctx: fcu_update::Context);

        #[task(
            binds = DMA2_STREAM0,
            local = [],
            shared = [],
            priority = 11,
        )]
        fn adc1_dma2_stream0_interrupt(ctx: adc1_dma2_stream0_interrupt::Context);

        #[task(
            binds = EXTI9_5,
            local = [bmi088_accel, bmi088_gyro, accel_int_pin, gyro_int_pin],
            shared = [fcu],
            priority = 11,
        )]
        fn bmi088_interrupt(ctx: bmi088_interrupt::Context);

        #[task(
            binds = EXTI3,
            local = [bmm150, mag_int_pin],
            shared = [fcu],
            priority = 13,
        )]
        fn bmm150_interrupt(ctx: bmm150_interrupt::Context);

        #[task(
            local = [ms5611],
            shared = [fcu],
            priority = 3,
        )]
        fn ms5611_update(ctx: ms5611_update::Context);

        #[task(
            local = [uart4],
            shared = [],
            priority = 3,
        )]
        fn ublox_update(ctx: ublox_update::Context);
    }

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<1000>;

    #[init(local = [
        eth_device: MaybeUninit<stm32_eth::dma::EthernetDMA<'static,'static> > = MaybeUninit::uninit(),
        rx_ring: [stm32_eth::dma::RxRingEntry; 4] = [stm32_eth::dma::RxRingEntry::INIT; 4],
        tx_ring: [stm32_eth::dma::TxRingEntry; 4] = [stm32_eth::dma::TxRingEntry::INIT; 4],
        smoltcp_interface_storage: SmoltcpInterfaceStorage<'static> = SmoltcpInterfaceStorage::new(),
        smoltcp_interface: MaybeUninit<SmoltcpInterface<'static, &'static mut stm32_eth::dma::EthernetDMA<'static, 'static>>> = MaybeUninit::uninit(),
        fcu_driver: MaybeUninit<Stm32F407FcuDriver> = MaybeUninit::uninit(),
        big_brother: MaybeUninit<FcuBigBrother<'static>> = MaybeUninit::uninit(),
        data_logger: MaybeUninit<DataLoggerType<'static>> = MaybeUninit::uninit(),
        logger_buffer0: [u8; logging::PAGE_SIZE] = [0u8; logging::PAGE_SIZE],
        logger_buffer1: [u8; logging::PAGE_SIZE] = [0u8; logging::PAGE_SIZE],
        i2c1_bus: MaybeUninit<I2C1BusType> = MaybeUninit::uninit(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut core = ctx.core;
        let mut p = ctx.device;

        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();
        let gpioc = p.GPIOC.split();
        let gpiod = p.GPIOD.split();
        let gpioe = p.GPIOE.split();

        let blue_led = gpioc.pc14.into_push_pull_output();
        let red_led = gpioc.pc15.into_push_pull_output();

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

        let mut syscfg = p.SYSCFG.constrain();
        let mono = Systick::new(core.SYST, clocks.hclk().raw());

        core.DCB.enable_trace();
        core.DWT.enable_cycle_counter();

        let output1_ctrl = gpioe.pe0.into_push_pull_output_in_state(PinState::Low);
        let output2_ctrl = gpioe.pe1.into_push_pull_output_in_state(PinState::Low);
        let output3_ctrl = gpioe.pe2.into_push_pull_output_in_state(PinState::Low);
        let output4_ctrl = gpioe.pe3.into_push_pull_output_in_state(PinState::Low);

        let output1_cont = gpioa.pa3.into_analog();
        let output2_cont = gpioa.pa4.into_analog();
        let output3_cont = gpioa.pa6.into_analog();
        let output4_cont = gpiob.pb0.into_analog();

        let i2c1_scl = gpiob.pb6.into_alternate_open_drain();
        let i2c1_sda = gpiob.pb7.into_alternate_open_drain();

        let spi1_sck = gpioa.pa5.into_alternate();
        let spi1_miso = gpiob.pb4.into_alternate();
        let spi1_mosi = gpiob.pb5.into_alternate();

        let log_flash_csn = gpioe.pe4.into_push_pull_output_in_state(PinState::High);
        let log_flash_hold = gpioe.pe5.into_push_pull_output_in_state(PinState::High);

        let usart2_tx = gpiod.pd5.into_alternate();
        let usart2_rx = gpiod.pd6.into_alternate();

        let uart4_tx = gpioc.pc10.into_alternate();
        let uart4_rx = gpioc.pc11.into_alternate();

        let dma2 = StreamsTuple::new(p.DMA2);

        let mut adc1 = Adc::adc1(p.ADC1, true, AdcConfig::default().dma(Dma::Continuous).scan(Scan::Enabled));
        adc1.configure_channel(&output1_cont, Sequence::One, SampleTime::Cycles_144);
        adc1.configure_channel(&output2_cont, Sequence::Two, SampleTime::Cycles_144);
        adc1.configure_channel(&output3_cont, Sequence::Three, SampleTime::Cycles_144);
        adc1.configure_channel(&output4_cont, Sequence::Four, SampleTime::Cycles_144);

        let adc1_buffer0 = cortex_m::singleton!(: [u16; 8] = [0; 8]).unwrap();
        let adc1_buffer1 = cortex_m::singleton!(: [u16; 8] = [0; 8]).unwrap();

        let adc1_transfer = Transfer::init_peripheral_to_memory(
            dma2.0,
            adc1,
            adc1_buffer0,
            None,
            DmaConfig::default().transfer_complete_interrupt(false).memory_increment(true).double_buffer(false),
        );

        let tim2_pwm = p.TIM2.pwm_hz((
            gpioa.pa15.into_alternate(),
            gpiob.pb3.into_alternate(),
        ), 50.Hz(), &clocks);

        let (mut ch1, mut ch2) = tim2_pwm.split();

        let ch1_duty = ((ch1.get_max_duty() as f32) * 0.075) as u16;

        ch1.set_duty(ch1_duty);
        ch1.enable();

        ch2.set_duty(ch2.get_max_duty() / 3);
        ch2.enable();

        let fcu_control_pins = FcuControlPins {
            output1_ctrl,
            output2_ctrl,
            output3_ctrl,
            output4_ctrl,
            output1_cont,
            output2_cont,
            output3_cont,
            output4_cont,
        };

        let i2c1_bus = ctx.local.i2c1_bus.write(shared_bus::BusManagerCortexM::new(
            p.I2C1.i2c(
                (i2c1_scl, i2c1_sda),
                400.kHz(),
                &clocks,
            ),
        ));

        let spi1: spi::Spi<SPI1, (gpio::Pin<'A', 5, gpio::Alternate<5>>, gpio::Pin<'B', 4, gpio::Alternate<5>>, gpio::Pin<'B', 5, gpio::Alternate<5>>), false> = p.SPI1.spi(
            (spi1_sck, spi1_miso, spi1_mosi),
            spi::Mode {
                polarity: spi::Polarity::IdleLow,
                phase: spi::Phase::CaptureOnFirstTransition,
            },
            25.MHz(),
            &clocks,
        );

        let mut usart2 = Serial::new(
            p.USART2,
            (usart2_tx, usart2_rx),
            serial::config::Config::default().baudrate(115_200.bps()),
            &clocks,
        ).unwrap().with_u8_data();
        usart2.listen(serial::Event::Rxne);

        let uart4 = Serial::new(
            p.UART4,
            (uart4_tx, uart4_rx),
            serial::config::Config::default().baudrate(115_200.bps()).parity_none().stopbits(serial::config::StopBits::STOP1),
            &clocks,
        ).unwrap().with_u8_data();

        let (usart2_tx, usart2_rx) = usart2.split();

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

        let mut bmi088_accel: Bmi088Accelerometer<SharedI2C1Type> = bmi088_rs::Bmi088Accelerometer::new(i2c1_bus.acquire_i2c(), 0x18);
        let mut bmi088_gyro: Bmi088Gyroscope<SharedI2C1Type> = bmi088_rs::Bmi088Gyroscope::new(i2c1_bus.acquire_i2c(), 0x68);
        let bmm150 = bmm150::Bmm150::new(
            0x10,
            bmm150::MagDataRate::Hz20,
        );
        let mut ms5611 = ms5611_rs::Ms5611::new(i2c1_bus.acquire_i2c(), 0x77);
        let w25x05 = w25x05::W25X05::new(log_flash_csn, log_flash_hold);

        bmi088_accel.reset().unwrap();
        bmi088_gyro.reset().unwrap();
        ms5611.reset().expect("MS5611 failed init");
        // bmm150.reset(&mut i2c1);
        // Delay for 100ms
        cortex_m::asm::delay((MCU_FREQ * 1) / 10);

        bmi088_accel.set_on(true).unwrap();
        bmi088_gyro.set_on(true).unwrap();
        ms5611.read_prom().expect("MS5611 failed read prom");
        // bmm150.turn_on(&mut i2c1);

        // Delay for 50ms
        cortex_m::asm::delay((MCU_FREQ * 5) / 100);

        bmi088_accel.set_bandwidth(AccelFilterBandwidth::Normal).unwrap();
        bmi088_accel.set_data_rate(AccelDataRate::Hz50).unwrap();
        bmi088_accel.set_range(AccelRange::G6).unwrap();
        bmi088_accel.configure_int1_pin(Bmi088PinMode::Output, Bmi088PinBehavior::PushPull, true, true).unwrap();

        bmi088_gyro.set_range(GyroRange::Deg2000).unwrap();
        bmi088_gyro.set_bandwidth(GyroBandwidth::Data200Filter23).unwrap();
        bmi088_gyro.configure_int3_pin(Bmi088PinBehavior::PushPull, true, true).unwrap();

        let mut accel_int_pin = gpioe.pe8.into_pull_down_input();
        accel_int_pin.make_interrupt_source(&mut syscfg);
        accel_int_pin.enable_interrupt(&mut p.EXTI);
        accel_int_pin.trigger_on_edge(&mut p.EXTI, Edge::Rising);

        let mut gyro_int_pin = gpioe.pe9.into_pull_down_input();
        gyro_int_pin.make_interrupt_source(&mut syscfg);
        gyro_int_pin.enable_interrupt(&mut p.EXTI);
        gyro_int_pin.trigger_on_edge(&mut p.EXTI, Edge::Rising);

        let mut mag_int_pin = gpioc.pc3.into_pull_down_input();
        mag_int_pin.make_interrupt_source(&mut syscfg);
        mag_int_pin.enable_interrupt(&mut p.EXTI);
        mag_int_pin.trigger_on_edge(&mut p.EXTI, Edge::Rising);

        heartbeat_blink_led::spawn().unwrap();

        let fcu_driver = ctx.local.fcu_driver.write(
            Stm32F407FcuDriver::new(fcu_control_pins),
        );

        let eth_device = ctx.local.eth_device.write(eth_device_dma);

        let smoltcp_interface = ctx.local.smoltcp_interface.write(
            SmoltcpInterface::new(
                [0x00, 0x80, 0xE1, 0x00, 0x00, 0x01],
                eth_device,
                [169, 254, 0, 7],
                16,
                ctx.local.smoltcp_interface_storage,
                0,
            ),
        );

        let mut rand_source = p.RNG.constrain(&clocks);

        let big_brother = ctx.local.big_brother.write(
            FcuBigBrother::new(
                NetworkAddress::FlightController,
                rand_source.next_u32(),
                [255, 255, 255, 255],
                NetworkAddress::Broadcast,
                [Some(smoltcp_interface), None],
            ),
        );

        let data_logger = ctx.local.data_logger.write(
            DataLoggerType::new(
                ctx.local.logger_buffer0,
                ctx.local.logger_buffer1,
                Some(logging::full_page_callback),
            ),
        );

        let fcu = Fcu::new(fcu_driver, big_brother, data_logger);

        fcu_update::spawn().unwrap();
        ms5611_update::spawn().unwrap();
        // ublox_update::spawn().unwrap();

        // Initiate a first read to get the data sequence going. I found that if I don't add this
        // then the first interrupt never gets triggered, likely because the line is already high
        // so the MCU never sees a rising edge
        // bmm150.read_mag(&mut i2c1);

        defmt::info!("Init complete!");

        (
            Shared {
                red_led,
                fcu,
                w25x05,
                spi1,
                cpu_utilization: AtomicU32::new(0),
            },
            Local {
                blue_led,
                bmi088_accel,
                bmi088_gyro,
                ms5611,
                bmm150,
                accel_int_pin,
                gyro_int_pin,
                mag_int_pin,
                usart2_tx,
                usart2_rx,
                uart4,
                adc1_transfer,
                adc1_other_buffer: Some(adc1_buffer1),
                dwt: core.DWT,
            },
            init::Monotonics(mono),
        )
    }

    #[idle(local = [dwt], shared = [&cpu_utilization])]
    fn idle(ctx: idle::Context) -> ! {
        let mut last_report_time = crate::now();
        let mut accum_idle_cycles = 0;

        loop {
            rtic::export::interrupt::free(|_cs| {
                let before = ctx.local.dwt.cyccnt.read();
                compiler_fence(Ordering::SeqCst);
                rtic::export::wfi();
                compiler_fence(Ordering::SeqCst);
                let after = ctx.local.dwt.cyccnt.read();

                let elapsed = after.wrapping_sub(before);
                accum_idle_cycles += elapsed;

                let current_time = crate::now();
                if current_time - last_report_time >= CPU_USAGE_RATE_MS {
                    let total_cycles = ((current_time - last_report_time) as u32) * (MCU_FREQ / 1000);
                    let cpu_util = (100 * (total_cycles - accum_idle_cycles)) / total_cycles;

                    ctx.shared.cpu_utilization.store(cpu_util, Ordering::Relaxed);
                    last_report_time = current_time;
                    accum_idle_cycles = 0;
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
fn red_led_panic(_: &PanicInfo) -> ! {
    let dp = unsafe { pac::Peripherals::steal() };
    let gpioc = dp.GPIOC.split();
    let mut red_led = gpioc.pc15.into_push_pull_output();

    loop {
        red_led.set_low();
        cortex_m::asm::delay(3_000_000);
        red_led.set_high();
        cortex_m::asm::delay(3_000_000);
    }
}
