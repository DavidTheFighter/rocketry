use core::sync::atomic::{compiler_fence, Ordering};

use crate::app;
use hal::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::ECUDAQFrame,
};
use rtic::Mutex;
use stm32_eth::stm32::{ADC1, ADC2, ADC3, DMA2};
use stm32f4xx_hal::{
    adc::Adc,
    dma::{PeripheralToMemory, Stream0, Stream1, Stream2, Transfer},
};

pub struct ADCStorage {
    pub adc1_transfer:
        Transfer<Stream0<DMA2>, 0, Adc<ADC1>, PeripheralToMemory, &'static mut [u16; 2]>,
    pub adc1_buffer: Option<&'static mut [u16; 2]>,
    pub adc2_transfer:
        Transfer<Stream2<DMA2>, 1, Adc<ADC2>, PeripheralToMemory, &'static mut [u16; 2]>,
    pub adc2_buffer: Option<&'static mut [u16; 2]>,
    pub adc3_transfer:
        Transfer<Stream1<DMA2>, 2, Adc<ADC3>, PeripheralToMemory, &'static mut [u16; 2]>,
    pub adc3_buffer: Option<&'static mut [u16; 2]>,
}

pub fn adc_dma(mut ctx: app::adc_dma::Context) {
    let storage = ctx.local.adc;

    let adc1_buffer = storage
        .adc1_transfer
        .next_transfer(storage.adc1_buffer.take().unwrap())
        .unwrap()
        .0;

    let adc2_buffer = storage
        .adc2_transfer
        .next_transfer(storage.adc2_buffer.take().unwrap())
        .unwrap()
        .0;

    let adc3_buffer = storage
        .adc3_transfer
        .next_transfer(storage.adc3_buffer.take().unwrap())
        .unwrap()
        .0;

    let daq_frame = ECUDAQFrame {
        sensor_values: [
            adc3_buffer[0],
            adc1_buffer[1],
            adc2_buffer[0],
            adc2_buffer[1],
            adc1_buffer[0],
            adc3_buffer[1],
        ],
    };

    ctx.shared.daq.lock(|daq| {
        if daq.add_daq_frame(daq_frame) {
            let daq_frame = Packet::ECUDAQ(*daq.get_inactive_buffer());

            app::send_packet::spawn(daq_frame, NetworkAddress::MissionControl).ok();
        }
    });

    storage.adc1_buffer = Some(adc1_buffer);
    storage.adc2_buffer = Some(adc2_buffer);
    storage.adc3_buffer = Some(adc3_buffer);

    storage.adc3_transfer.start(|adc| adc.start_conversion());
    storage.adc2_transfer.start(|adc| adc.start_conversion());
    compiler_fence(Ordering::SeqCst);
    storage.adc1_transfer.start(|adc| adc.start_conversion());
}
