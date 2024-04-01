use core::sync::atomic::{compiler_fence, Ordering};

use crate::app;
use rtic::Mutex;
use shared::{ecu_hal::EcuSensor, SensorData};
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

const PMAP_MIN: f32 = 410.0;
const PMAP_MAX: f32 = 3686.0;

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

    ctx.shared.ecu.lock(|ecu| {
        ecu.update_sensor_data(
            EcuSensor::FuelPumpOutletPressure,
            &SensorData::Pressure {
                pressure_pa: (((adc1_buffer[0] as f32) - PMAP_MIN) / (PMAP_MAX - PMAP_MIN)) * 300.0,
                raw_data: adc1_buffer[0],
            },
        );

        ecu.update_sensor_data(
            EcuSensor::FuelPumpInletPressure,
            &SensorData::Pressure {
                pressure_pa: (((adc1_buffer[1] as f32) - PMAP_MIN) / (PMAP_MAX - PMAP_MIN)) * 300.0,
                raw_data: adc1_buffer[1],
            },
        );

        ecu.update_sensor_data(
            EcuSensor::FuelPumpInducerPressure,
            &SensorData::Pressure {
                pressure_pa: (((adc2_buffer[0] as f32) - PMAP_MIN) / (PMAP_MAX - PMAP_MIN)) * 300.0,
                raw_data: adc2_buffer[0],
            },
        );
    });

    storage.adc1_buffer = Some(adc1_buffer);
    storage.adc2_buffer = Some(adc2_buffer);
    storage.adc3_buffer = Some(adc3_buffer);

    storage.adc3_transfer.start(|adc| adc.start_conversion());
    storage.adc2_transfer.start(|adc| adc.start_conversion());
    compiler_fence(Ordering::SeqCst);
    storage.adc1_transfer.start(|adc| adc.start_conversion());
}
