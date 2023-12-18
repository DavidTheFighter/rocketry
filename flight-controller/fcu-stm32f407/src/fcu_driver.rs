use core::sync::atomic::Ordering;

use rtic::Mutex;
use shared::fcu_hal::{OutputChannel, PwmChannel, FcuDriver, FcuHardwareData};
use shared::comms_hal::{Packet, NetworkAddress};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::gpio::{PE0, PE1, PE2, PE3, Output, PinState, PA3, PA6, PA4, PB0, Analog};
use strum::EnumCount;

use crate::{app, logging};

#[derive(Debug)]
pub struct FcuControlPins {
    pub output1_ctrl: PE0<Output>,
    pub output2_ctrl: PE1<Output>,
    pub output3_ctrl: PE2<Output>,
    pub output4_ctrl: PE3<Output>,
    pub output1_cont: PA3<Analog>,
    pub output2_cont: PA4<Analog>,
    pub output3_cont: PA6<Analog>,
    pub output4_cont: PB0<Analog>,
}

#[derive(Debug)]
pub struct Stm32F407FcuDriver {
    pins: FcuControlPins,
    outputs: [bool; 4],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; 4],
    hardware_data: FcuHardwareData,
}

impl FcuDriver for Stm32F407FcuDriver {
    fn timestamp(&self) -> f32 {
        (crate::now() as f32) * 1e-3
    }

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        let pin_state = if state { PinState::High } else { PinState::Low };
        match channel {
            OutputChannel::SolidMotorIgniter => self.pins.output1_ctrl.set_state(pin_state),
            OutputChannel::Extra { index: _ } => {},
            // OutputChannel::OutputChannel1 => self.pins.output2_ctrl.set_state(pin_state),
            // OutputChannel::OutputChannel2 => self.pins.output3_ctrl.set_state(pin_state),
            // OutputChannel::OutputChannel3 => self.pins.output4_ctrl.set_state(pin_state),
        }

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

    fn erase_flash_chip(&mut self) {
        logging::erase_flash_chip();
    }

    fn enable_logging_to_flash(&mut self) {
        // app::set_data_logging_state::spawn(true).unwrap();
    }

    fn disable_logging_to_flash(&mut self) {
        // app::set_data_logging_state::spawn(false).unwrap();
    }

    fn retrieve_log_flash_page(&mut self, addr: u32) {
        defmt::info!("Retrieving log flash page {}", addr);
        // app::read_log_page_and_transfer::spawn(addr).unwrap();
    }

    fn hardware_data(&self) -> FcuHardwareData {
        self.hardware_data.clone()
    }

    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

pub fn fcu_update(mut ctx: app::fcu_update::Context) {
    app::fcu_update::spawn_after(10.millis().into()).unwrap();

    let sample_to_millivolts = ctx.local.adc1_transfer.peripheral().make_sample_to_millivolts();

        let adc1_result = ctx.local.adc1_transfer
            .next_transfer(ctx.local.adc1_other_buffer.take().unwrap());

    ctx.shared.fcu.lock(|fcu| {
        let driver = fcu.driver.as_mut_any().downcast_mut::<Stm32F407FcuDriver>().unwrap();

        driver.hardware_data.cpu_utilization = ctx.shared.cpu_utilization.load(Ordering::Relaxed) as f32;

        if let Ok((buffer, _)) = adc1_result {
            let output1_cont = sample_to_millivolts(buffer[0]);
            let output2_cont = sample_to_millivolts(buffer[1]);
            let output3_cont = sample_to_millivolts(buffer[2]);
            let output4_cont = sample_to_millivolts(buffer[3]);

            driver.continuities[OutputChannel::SolidMotorIgniter.index()] = output1_cont > 50;
            driver.continuities[OutputChannel::Extra { index: 0 }.index()] = output2_cont > 50;
            driver.continuities[OutputChannel::Extra { index: 1 }.index()] = output3_cont > 50;
            driver.continuities[OutputChannel::Extra { index: 2 }.index()] = output4_cont > 50;

            ctx.local.adc1_other_buffer.replace(buffer);
        }

        fcu.update(0.01);
    });

    ctx.local.adc1_transfer.start(|adc1| {
        adc1.start_conversion();
    });
}

impl Stm32F407FcuDriver {
    pub fn new(pins: FcuControlPins) -> Self {
        Self {
            pins,
            outputs: [false; 4],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; 4],
            hardware_data: FcuHardwareData::default(),
        }
    }
}
