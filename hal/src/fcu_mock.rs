use core::any::Any;

use crate::{fcu_hal::{OutputChannel, PwmChannel, FcuDriver}, comms_hal::{Packet, NetworkAddress}};
use strum::EnumCount;

#[derive(Debug)]
pub struct FcuDriverMock {
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
}

impl FcuDriver for FcuDriverMock {
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

    fn send_packet(&mut self, _packet: Packet, _destination: NetworkAddress) {
    }
}

impl FcuDriverMock {
    pub const fn new() -> Self {
        Self {
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
        }
    }
}