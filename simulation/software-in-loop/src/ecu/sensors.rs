use pyo3::prelude::*;
use strum::IntoEnumIterator;
use std::{collections::HashMap, hash::Hash};

use pyo3::types::PyDict;
use shared::ecu_hal::EcuSensor;

use crate::sensors::{LinearVoltagePressureTranducer, LinearVoltageTemperatureSensor, SensorNoise};

use super::EcuSil;

pub fn initialize_sensors(noise_config: &PyDict) -> HashMap<EcuSensor, Box<dyn SensorNoise>> {
    let mut sensors = HashMap::new();

    for sensor_variant in EcuSensor::iter() {
        match sensor_variant {
            EcuSensor::FuelTankPressure
            | EcuSensor::OxidizerTankPressure
            | EcuSensor::IgniterChamberPressure
            | EcuSensor::IgniterFuelInjectorPressure
            | EcuSensor::IgniterOxidizerInjectorPressure
            | EcuSensor::EngineChamberPressure
            | EcuSensor::EngineFuelInjectorPressure
            | EcuSensor::EngineOxidizerInjectorPressure
            | EcuSensor::FuelPumpOutletPressure
            | EcuSensor::FuelPumpInletPressure
            | EcuSensor::FuelPumpInducerPressure
            | EcuSensor::OxidizerPumpOutletPressure
            | EcuSensor::OxidizerPumpInletPressure
            | EcuSensor::OxidizerPumpInducerPressure
            => {
                setup_pressure_sensor_noise(&mut sensors, sensor_variant, noise_config);
            },
            EcuSensor::IgniterThroatTemperature
            | EcuSensor::EngineThroatTemperature => {
                setup_temperature_sensor_noise(&mut sensors, sensor_variant, noise_config);
            },
        }
    }

    sensors
}

impl EcuSil {
    pub fn update_sensors(&mut self, py: Python, dt: f64) {
        for sensor in EcuSensor::iter() {
            let direct_sensor_value = self.get_direct_sensor_value(py, sensor);

            if let Some(sensor_handler) = self.sensors.get_mut(&sensor) {
                if let Some(sensor_data) = sensor_handler.update(direct_sensor_value, dt) {
                    self.ecu.update_sensor_data(sensor, &sensor_data);
                }
            }
        }
    }
}

pub fn setup_pressure_sensor_noise(
    sensors: &mut HashMap<EcuSensor, Box<dyn SensorNoise>>,
    sensor: EcuSensor,
    noise_config: &PyDict,
) {
    if let Some(sensor_noise_config) = noise_config.get_item(format!("{:?}", sensor)) {
        let rate = sensor_noise_config
            .get_item("rate")
            .expect(&format!("Failed to get rate from noise_config for {:?}", sensor))
            .extract::<f64>()
            .expect(&format!("Failed to extract rate from noise_config for {:?}", sensor));

        let std_dev = sensor_noise_config
            .get_item("std_dev")
            .expect(&format!("Failed to get std_dev from noise_config for {:?}", sensor))
            .extract::<f64>()
            .expect(&format!("Failed to extract std_dev from noise_config for {:?}", sensor));

        let pressure_min = sensor_noise_config
            .get_item("pressure_min")
            .expect(&format!("Failed to get pressure_min from noise_config for {:?}", sensor))
            .extract::<f64>()
            .expect(&format!("Failed to extract pressure_min from noise_config for {:?}", sensor));

        let pressure_max = sensor_noise_config
            .get_item("pressure_max")
            .expect(&format!("Failed to get pressure_max from noise_config for {:?}", sensor))
            .extract::<f64>()
            .expect(&format!("Failed to extract pressure_max from noise_config for {:?}", sensor));

        sensors.insert(sensor, Box::new(LinearVoltagePressureTranducer::new(
            rate,
            pressure_min,
            pressure_max,
            410,
            3685,
            std_dev,
        )));
    } else {
        eprintln!("Missing noise config for {:?}", sensor);
    }
}

pub fn setup_temperature_sensor_noise(
    sensors: &mut HashMap<EcuSensor, Box<dyn SensorNoise>>,
    sensor: EcuSensor,
    noise_config: &PyDict,
) {
    sensors.insert(sensor, Box::new(LinearVoltageTemperatureSensor::new(
        0.1,
        0.0,
        1.0,
        0,
        4095,
        0.1,
    )));
}
