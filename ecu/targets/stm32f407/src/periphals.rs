use stm32f4::stm32f407::Peripherals;
use stm32f4xx_hal::{
    prelude::*,
    gpio::{Output, PE5},
};
use stm32_eth::{
    hal::gpio::GpioExt,
    hal::rcc::RccExt,
    RxRingEntry,
    TxRingEntry,
    EthPins,
    EthernetDMA,
};

fn init_peripherals<'a>(p: Peripherals) -> (
    PE5<Output>,
    EthernetDMA<'a, 'a>,
) {
    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();
    let gpioe = p.GPIOE.split();

    let blue_led = gpioe.pe5.into_push_pull_output();

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

    (eth_dma, )
}
