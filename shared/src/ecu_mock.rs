use core::any::Any;

use crate::ecu_hal::{EcuBinaryOutput, EcuDriver, EcuLinearOutput, EcuSensor};
use strum::EnumCount;

pub struct EcuDriverMock {
    start_timestamp: f64,
    sparking: bool,
    binary_valves: [bool; EcuBinaryOutput::COUNT],
    linear_outputs: [f32; EcuLinearOutput::COUNT],
    sensors: [(f32, f32, f32); EcuSensor::COUNT],
}

impl EcuDriver for EcuDriverMock {
    fn timestamp(&self) -> f32 {
        (get_timestamp() - self.start_timestamp) as f32
    }

    fn set_sparking(&mut self, state: bool) {
        self.sparking = state;
    }

    fn set_binary_valve(&mut self, valve: EcuBinaryOutput, state: bool) {
        self.binary_valves[valve.index()] = state;
    }

    fn get_binary_valve(&self, valve: EcuBinaryOutput) -> bool {
        self.binary_valves[valve.index()]
    }

    fn set_linear_output(&mut self, output: EcuLinearOutput, value: f32) {
        self.linear_outputs[output.index()] = value;
    }

    fn get_linear_output(&self, output: EcuLinearOutput) -> f32 {
        self.linear_outputs[output.index()]
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl EcuDriverMock {
    pub fn new() -> Self {
        Self {
            start_timestamp: get_timestamp(),
            sparking: false,
            binary_valves: [false; EcuBinaryOutput::COUNT],
            linear_outputs: [0.0; EcuLinearOutput::COUNT],
            sensors: [(0_f32, 0_f32, 0_f32); EcuSensor::COUNT],
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
    panic!("ecu_mock.rs: get_timestamp() should only be called in tests")
}
