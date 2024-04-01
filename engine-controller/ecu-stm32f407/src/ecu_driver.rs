use core::sync::atomic::Ordering;

use shared::SensorConfig;
use shared::comms_hal::{Packet, NetworkAddress};
use shared::ecu_hal::{EcuBinaryOutput, EcuDriver, EcuSensor, EcuTelemetryFrame, IgniterState};
use rtic::Mutex;
use stm32f4xx_hal::gpio::{PA12, PA11, PA10, PA9, Output, PinState};
use stm32f4xx_hal::signature::{VtempCal30, VtempCal110};
use stm32f4xx_hal::timer::PwmChannel;
use stm32f4xx_hal::pac::TIM1;
use stm32f4xx_hal::prelude::*;
use strum::{IntoEnumIterator, EnumCount};

use crate::{app, now};

pub struct EcuControlPins {
    pub sv1_ctrl: PA12<Output>,
    pub sv2_ctrl: PA11<Output>,
    pub sv3_ctrl: PA10<Output>,
    pub sv4_ctrl: PA9<Output>,
    pub spark_ctrl: PwmChannel<TIM1, 0>,
}

pub struct Stm32F407EcuDriver {
    pins: EcuControlPins,
    sensor_configs: [SensorConfig; EcuSensor::COUNT],
    last_update_time: u64,
    cpu_utilization: u32,
    sensor_values: [f32; EcuSensor::COUNT],
    sensor_mins: [f32; EcuSensor::COUNT],
    sensor_maxs: [f32; EcuSensor::COUNT],
    solenoid_valve_states: [bool; EcuBinaryOutput::COUNT],
    sparking: bool,
}

impl Stm32F407EcuDriver {
    pub fn new(pins: EcuControlPins) -> Self {
        Self {
            pins,
            sensor_configs: [SensorConfig::default(); EcuSensor::COUNT],
            last_update_time: 0,
            cpu_utilization: 0,
            sensor_values: [0.0; EcuSensor::COUNT],
            sensor_mins: [0.0; EcuSensor::COUNT],
            sensor_maxs: [0.0; EcuSensor::COUNT],
            solenoid_valve_states: [false; EcuBinaryOutput::COUNT],
            sparking: false,
        }
    }
}

impl EcuDriver for Stm32F407EcuDriver {
    fn timestamp(&self) -> f32 {
        (crate::now() as f32) * 1e-3
    }

    fn set_binary_valve(&mut self, valve: EcuBinaryOutput, state: bool) {
        let pin_state = if state { PinState::High } else { PinState::Low };

        match valve {
            EcuBinaryOutput::IgniterFuelValve => self.pins.sv1_ctrl.set_state(pin_state),
            EcuBinaryOutput::IgniterOxidizerValve => self.pins.sv2_ctrl.set_state(pin_state),
            EcuBinaryOutput::FuelPressValve => self.pins.sv3_ctrl.set_state(pin_state),
            EcuBinaryOutput::FuelVentValve => self.pins.sv4_ctrl.set_state(pin_state),
            _ => {},
        }

        self.solenoid_valve_states[valve as usize] = state;
    }

    fn get_binary_valve(&self, valve: EcuBinaryOutput) -> bool {
        self.solenoid_valve_states[valve.index()]
    }

    fn set_sparking(&mut self, state: bool) {
        if state {
            self.pins.spark_ctrl.enable();
            self.pins
                .spark_ctrl
                .set_duty(self.pins.spark_ctrl.get_max_duty() / 8);
        } else {
            self.pins.spark_ctrl.disable();
            self.pins.spark_ctrl.set_duty(0);
        }

        self.sparking = state;
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

pub fn ecu_update(mut ctx: app::ecu_update::Context) {
    app::ecu_update::spawn_after(1.millis().into()).unwrap();

    let cpu_utilization = ctx.shared.cpu_utilization.load(Ordering::Relaxed);

    ctx.shared.ecu.lock(|ecu| {
        ecu.update(0.001);
    });
}

fn raw_board_temp_to_celsius(sample: u16) -> f32 {
    let cal30 = VtempCal30::get().read() as f32;
    let cal110 = VtempCal110::get().read() as f32;

    (110.0 - 30.0) * ((sample as f32) - cal30) / (cal110 - cal30) + 30.0
}
