use shared::fcu_hal::{OutputChannel, PwmChannel, FcuDriver};
use shared::comms_hal::{Packet, NetworkAddress};
use rtic::mutex_prelude::TupleExt03;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::gpio::{PE0, PE1, PE2, PE3, Output, PinState};
use strum::EnumCount;

use crate::app;

#[derive(Debug)]
pub struct FcuControlPins {
    pub output1_ctrl: PE0<Output>,
    pub output2_ctrl: PE1<Output>,
    pub output3_ctrl: PE2<Output>,
    pub output4_ctrl: PE3<Output>,
}

#[derive(Debug)]
pub struct Stm32F407FcuDriver {
    pins: FcuControlPins,
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
}

impl FcuDriver for Stm32F407FcuDriver {
    fn timestamp(&self) -> f32 {
        (crate::now() as f32) * 1e-3
    }

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        let pin_state = if state { PinState::High } else { PinState::Low };
        match channel {
            OutputChannel::OutputChannel0 => self.pins.output1_ctrl.set_state(pin_state),
            OutputChannel::OutputChannel1 => self.pins.output2_ctrl.set_state(pin_state),
            OutputChannel::OutputChannel2 => self.pins.output3_ctrl.set_state(pin_state),
            OutputChannel::OutputChannel3 => self.pins.output4_ctrl.set_state(pin_state),
        }

        self.outputs[channel as usize] = state;
    }

    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32) {
        self.pwm[channel as usize] = duty_cycle;
    }

    fn get_output_channel(&self, channel: OutputChannel) -> bool {
        self.outputs[channel as usize]
    }

    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool {
        self.continuities[channel as usize]
    }

    fn get_pwm_channel(&self, channel: PwmChannel) -> f32 {
        self.pwm[channel as usize]
    }

    fn erase_flash_chip(&mut self) {
        app::erase_data_log_flash::spawn().unwrap();
    }

    fn enable_logging_to_flash(&mut self) {
        app::set_data_logging_state::spawn(true).unwrap();
    }

    fn disable_logging_to_flash(&mut self) {
        app::set_data_logging_state::spawn(false).unwrap();
    }

    fn retrieve_log_flash_page(&mut self, addr: u32) {
        defmt::info!("Retrieving log flash page {}", addr);
        // app::read_log_page_and_transfer::spawn(addr).unwrap();
    }

    fn send_packet(&mut self, packet: Packet, destination: NetworkAddress) {
        app::send_packet::spawn(packet, destination).unwrap();
    }

    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

pub fn fcu_update(ctx: app::fcu_update::Context) {
    app::fcu_update::spawn_after(10.millis().into()).unwrap();

    let fcu = ctx.shared.fcu;
    let packet_queue = ctx.shared.packet_queue;
    let data_logger = ctx.shared.data_logger;

    (fcu, packet_queue, data_logger).lock(|fcu, packet_queue, data_logger| {
        fcu.update_data_logged_bytes(data_logger.get_bytes_logged());

        let mut packet_array_len = 0;
        let mut packet_array = empty_packet_array();

        while let Some(packet) = packet_queue.dequeue() {
            packet_array[packet_array_len] = packet;
            packet_array_len += 1;

            if packet_array_len == packet_array.len() {
                break;
            }
        }

        fcu.update(0.01, &packet_array[0..packet_array_len]);
    });
}

impl Stm32F407FcuDriver {
    pub const fn new(pins: FcuControlPins) -> Self {
        Self {
            pins,
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
        }
    }
}

fn empty_packet_array() -> [(NetworkAddress, Packet); 16] {
    [
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
    ]
}