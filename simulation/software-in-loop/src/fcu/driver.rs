use std::any::Any;

use shared::fcu_hal::{FcuDriver, FcuHardwareData, OutputChannel, PwmChannel};
use strum::EnumCount;

#[derive(Debug)]
pub struct FcuDriverSim {
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
    pub current_sim_timestamp: f32,
    pub last_sim_timestamp_update_timestamp: f64,
}

impl FcuDriver for FcuDriverSim {
    fn timestamp(&self) -> f32 {
        let elapsed = get_timestamp() - self.last_sim_timestamp_update_timestamp;

        self.current_sim_timestamp + (elapsed as f32)
    }

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        self.outputs[channel.index()] = state;
    }

    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32) {
        self.pwm[channel as usize] = duty_cycle;
    }

    fn get_output_channel(&self, channel: OutputChannel) -> bool {
        self.outputs[channel.index()]
    }

    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool {
        self.continuities[channel.index()]
    }

    fn get_pwm_channel(&self, channel: PwmChannel) -> f32 {
        self.pwm[channel as usize]
    }

    fn hardware_data(&self) -> FcuHardwareData {
        FcuHardwareData::default()
    }

    fn reset_mcu(&mut self) {
        // Nothing
    }

    fn erase_flash_chip(&mut self) {
        // Nothing
    }

    fn enable_logging_to_flash(&mut self) {
        // Nothing
    }

    fn disable_logging_to_flash(&mut self) {
        // Nothing
    }

    fn retrieve_log_flash_page(&mut self, _addr: u32) {
        // Nothing
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl FcuDriverSim {
    pub fn new() -> Self {
        Self {
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
            current_sim_timestamp: 0.0,
            last_sim_timestamp_update_timestamp: get_timestamp(),
        }
    }

    pub fn update_timestamp(&mut self, sim_time: f32) {
        self.current_sim_timestamp = sim_time;
        self.last_sim_timestamp_update_timestamp = get_timestamp();
    }

    pub fn set_output_channel_continuity(&mut self, channel: OutputChannel, state: bool) {
        self.continuities[channel.index()] = state;
    }
}

fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}
