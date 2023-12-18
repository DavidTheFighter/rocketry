use core::any::Any;

use crate::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{FcuDriver, OutputChannel, PwmChannel, FcuHardwareData},
};
use strum::EnumCount;

#[derive(Debug)]
pub struct FcuDriverMock {
    start_timestamp: f64,
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
}

impl FcuDriver for FcuDriverMock {
    fn timestamp(&self) -> f32 {
        (get_timestamp() - self.start_timestamp) as f32
    }

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        self.outputs[0] = state;
    }

    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32) {
        self.pwm[channel as usize] = duty_cycle;
    }

    fn get_output_channel(&self, channel: OutputChannel) -> bool {
        self.outputs[0]
    }

    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool {
        self.continuities[0]
    }

    fn get_pwm_channel(&self, channel: PwmChannel) -> f32 {
        self.pwm[channel as usize]
    }

    fn hardware_data(&self) -> FcuHardwareData {
        FcuHardwareData {
            cpu_utilization: 0.0,
        }
    }

    // fn log_data_point(&mut self, _datapoint: DataPoint) {}

    fn erase_flash_chip(&mut self) {
        todo!()
    }

    fn enable_logging_to_flash(&mut self) {
        todo!()
    }

    fn disable_logging_to_flash(&mut self) {
        todo!()
    }

    fn retrieve_log_flash_page(&mut self, _addr: u32) {
        todo!()
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl FcuDriverMock {
    pub fn new() -> Self {
        Self {
            start_timestamp: get_timestamp(),
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
        }
    }
}

#[cfg(test)]
fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}

#[cfg(not(test))]
fn get_timestamp() -> f64 {
    panic!("fcu_mock.rs: get_timestamp() should only be called in tests")
}
