use std::collections::HashMap;
use pyo3::{prelude::*, types::PyDict};

use shared::ecu_hal::EcuSensor;
use strum::IntoEnumIterator;

use crate::ecu::EcuSil;

use super::noise::{LinearVoltagePressureTranducer, SensorNoise};

pub struct EcuSilSensorManager {
    sensors: HashMap<EcuSensor, Box<dyn SensorNoise>>,
}

impl EcuSilSensorManager {
    pub fn new(noise_config: &PyDict) -> Self {
        let mut manager = Self {
            sensors: HashMap::new(),
        };

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
                | EcuSensor::OxidizerPumpOutletPressure => {
                    manager.insert_pressure_sensor_noise(sensor_variant, noise_config);
                },
                EcuSensor::IgniterThroatTemperature
                | EcuSensor::EngineThroatTemperature => {
                    manager.insert_temperature_sensor_noise(sensor_variant, noise_config);
                },
            }
        }

        manager
    }

    pub fn update_from_ecu(&mut self, py: Python, ecu: &mut EcuSil, dt: f64) {
        for sensor in EcuSensor::iter() {
            let sensor_handler = self.sensors.get_mut(&sensor).unwrap();
            let direct_sensor_value = ecu.get_direct_sensor_value(py, sensor);

            let sensor_data = sensor_handler.update(direct_sensor_value, dt);

            ecu.ecu.update_sensor_data(sensor, &sensor_data);
        }
    }

    // fn update_ecu_sensor(&self, ecu: &mut EcuSil, variant: EcuSensor, value: f64, raw: u16) {
    //     match variant {
    //         EcuSensor::FuelTankPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::FuelTankPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::OxidizerTankPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::OxidizerTankPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::IgniterChamberPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::IgniterChamberPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::IgniterFuelInjectorPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::IgniterFuelInjectorPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::IgniterOxidizerInjectorPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::IgniterOxidizerInjectorPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::IgniterThroatTemperature => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::IgniterThroatTemperature(TemperatureData {
    //                 temperature_k: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::EngineChamberPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::EngineChamberPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::EngineFuelInjectorPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::EngineFuelInjectorPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::EngineOxidizerInjectorPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::EngineOxidizerInjectorPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::EngineThroatTemperature => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::EngineThroatTemperature(TemperatureData {
    //                 temperature_k: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::FuelPumpOutletPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::FuelPumpOutletPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //         EcuSensor::OxidizerPumpOutletPressure => {
    //             ecu.ecu.update_sensor_data(&EcuSensor::OxidizerPumpOutletPressure(PressureData {
    //                 pressure_pa: value as f32,
    //                 raw_data: raw,
    //             }));
    //         },
    //     }
    // }

    fn insert_pressure_sensor_noise(
        &mut self,
        sensor: EcuSensor,
        noise_config: &PyDict,
    ) {
        let sensor_noise_config = noise_config
            .get_item(format!("{:?}", sensor))
            .expect(&format!("Failed to get noise from noise_config for {:?}", sensor));

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

        self.sensors.insert(sensor, Box::new(LinearVoltagePressureTranducer::new(
            pressure_min,
            pressure_max,
            410,
            3685,
            std_dev,
        )));
    }

    fn insert_temperature_sensor_noise(
        &mut self,
        sensor: EcuSensor,
        noise_config: &PyDict,
    ) {
        let std_dev = noise_config
            .get_item(format!("{:?}_std_dev", sensor))
            .expect(&format!("Failed to get std_dev from noise_config for {:?}", sensor))
            .extract::<f64>()
            .expect(&format!("Failed to extract std_dev from noise_config for {:?}", sensor)) as f32;

        // self.sensors.insert(sensor, Box::new(LinearVoltagePressureTranducer::new(std_dev)));
    }
}
