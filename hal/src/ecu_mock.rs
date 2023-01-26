use core::any::Any;

use crate::{
    ecu_hal::{EcuTelemetryFrame, EcuDriver, EcuSensor, EcuSolenoidValve},
    SensorConfig,
};
use strum::EnumCount;

pub struct EcuDriverMock {
    sensor_configs: [SensorConfig; EcuSensor::COUNT],
    sparking: bool,
    solenoid_valves: [bool; EcuSolenoidValve::COUNT],
    sensors: [(f32, f32, f32); EcuSensor::COUNT],
}

impl EcuDriver for EcuDriverMock {
    fn set_solenoid_valve(&mut self, valve: EcuSolenoidValve, state: bool) {
        self.solenoid_valves[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        self.sparking = state;
    }

    fn get_solenoid_valve(&self, valve: EcuSolenoidValve) -> bool {
        self.solenoid_valves[valve as usize]
    }

    fn get_sensor(&self, sensor: EcuSensor) -> f32 {
        self.sensors[sensor as usize].0
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn generate_telemetry_frame(&self) -> EcuTelemetryFrame {
        todo!()
    }

    fn collect_daq_sensor_data(&mut self, sensor: EcuSensor) -> (f32, f32, f32) {
        let data = self.sensors[sensor as usize];

        self.sensors[sensor as usize] = (data.0, data.0, data.0);

        data
    }

    fn configure_sensor(&mut self, sensor: EcuSensor, config: SensorConfig) {
        self.sensor_configs[sensor as usize] = config;
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl EcuDriverMock {
    pub const fn new() -> Self {
        Self {
            sensor_configs: [SensorConfig::default(); EcuSensor::COUNT],
            sparking: false,
            solenoid_valves: [false; EcuSolenoidValve::COUNT],
            sensors: [(0_f32, 0_f32, 0_f32); EcuSensor::COUNT],
        }
    }

    pub fn update_sensor_value(&mut self, sensor: EcuSensor, value: f32) {
        let current_value = self.sensors[sensor as usize];

        self.sensors[sensor as usize] = (
            value,
            value.min(current_value.1),
            value.max(current_value.2),
        );
    }

    pub fn get_daq_sensor_collection(&self, sensor: EcuSensor) -> (f32, f32, f32) {
        self.sensors[sensor as usize]
    }
}
