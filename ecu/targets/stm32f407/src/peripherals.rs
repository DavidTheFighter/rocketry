use core::sync::atomic::{compiler_fence, Ordering};

use crate::app;
use hal::{ecu_hal::ECUDAQFrame, comms_hal::{Packet, NetworkAddress}};
use rtic::Mutex;

pub fn adc_dma(mut ctx: app::adc_dma::Context) {
    let adc1_buffer = ctx.local.adc1_transfer
        .next_transfer(ctx.local.adc1_buffer.take().unwrap())
        .unwrap().0;

    let adc2_buffer = ctx.local.adc2_transfer
        .next_transfer(ctx.local.adc2_buffer.take().unwrap())
        .unwrap().0;

    let adc3_buffer = ctx.local.adc3_transfer
        .next_transfer(ctx.local.adc3_buffer.take().unwrap())
        .unwrap().0;

    let daq_frame = ECUDAQFrame {
        sensor_values: [
            adc1_buffer[0],
            adc1_buffer[1],
            adc2_buffer[0],
            adc2_buffer[1],
            0,
            adc3_buffer[1],
        ],
    };

    ctx.shared.current_daq_frame.lock(|current_daq_frame| {
        *current_daq_frame = daq_frame;
    });

    *ctx.local.adc1_buffer = Some(adc1_buffer);
    *ctx.local.adc2_buffer = Some(adc2_buffer);
    *ctx.local.adc3_buffer = Some(adc3_buffer);

    if ctx.local.daq.add_daq_frame(daq_frame) {
        let daq_frame = Packet::ECUDAQ(*ctx.local.daq.get_inactive_buffer());

        app::send_packet::spawn(daq_frame, NetworkAddress::MissionControl).ok();
    }

    ctx.local.adc1_transfer.start(|adc| adc.start_conversion());
    ctx.local.adc2_transfer.start(|adc| adc.start_conversion());
    compiler_fence(Ordering::SeqCst);
    ctx.local.adc3_transfer.start(|adc| adc.start_conversion());
}
