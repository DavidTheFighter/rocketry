use core::any::Any;

use crate::ecu_hal::{EcuDriver, EcuSensor, EcuBinaryOutput};
use strum::EnumCount;

pub struct EcuDriverMock {
    start_timestamp: f64,
    sparking: bool,
    binary_valves: [bool; EcuBinaryOutput::COUNT],
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
