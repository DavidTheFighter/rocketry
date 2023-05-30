use hal::fcu_hal::{OutputChannel, PwmChannel, FcuDriver};
use hal::comms_hal::{Packet, NetworkAddress};
use strum::EnumCount;

use crate::app;

#[derive(Debug)]
pub struct Stm32F407FcuDriver {
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
}

impl FcuDriver for Stm32F407FcuDriver {
    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
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
}

impl Stm32F407FcuDriver {
    pub const fn new() -> Self {
        Self {
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
        }
    }
}