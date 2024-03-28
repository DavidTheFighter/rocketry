use serde::Serialize;
use shared::{ecu_hal::EcuSensor, SensorData};

#[derive(Debug, Clone, Serialize)]
pub struct SensorDataVector {
    pub fuel_tank_pressure_pa: f32, // Pa
    pub oxidizer_tank_pressure_pa: f32, // Pa
    pub igniter_chamber_pressure_pa: f32, // Pa
    pub igniter_fuel_injector_pressure_pa: Option<f32>, // Pa
    pub igniter_oxidizer_injector_pressure_pa: Option<f32>, // Pa
}

#[derive(Debug, Clone, Serialize)]
pub struct StateVector {
    pub(crate) sensor_data: SensorDataVector,
}

impl StateVector {
    pub fn new() -> Self {
        Self {
            sensor_data: SensorDataVector {
                fuel_tank_pressure_pa: 0.0,
                oxidizer_tank_pressure_pa: 0.0,
                igniter_chamber_pressure_pa: 0.0,
                igniter_fuel_injector_pressure_pa: None,
                igniter_oxidizer_injector_pressure_pa: None,
            },
        }
    }

    pub fn update_sensor_data(&mut self, sensor: EcuSensor, data: &SensorData) {
        match sensor {
            EcuSensor::FuelTankPressure => {
                if let SensorData::Pressure { pressure_pa, .. } = data {
                    self.sensor_data.fuel_tank_pressure_pa = *pressure_pa;
                }
            },
            EcuSensor::OxidizerTankPressure => {
                if let SensorData::Pressure { pressure_pa, .. } = data {
                    self.sensor_data.oxidizer_tank_pressure_pa = *pressure_pa;
                }
            },
            EcuSensor::IgniterChamberPressure => {
                if let SensorData::Pressure { pressure_pa, .. } = data {
                    self.sensor_data.igniter_chamber_pressure_pa = *pressure_pa;
                }
            },
            EcuSensor::IgniterFuelInjectorPressure => {
                if let SensorData::Pressure { pressure_pa, .. } = data {
                    self.sensor_data.igniter_fuel_injector_pressure_pa = Some(*pressure_pa);
                }
            },
            EcuSensor::IgniterOxidizerInjectorPressure => {
                if let SensorData::Pressure { pressure_pa, .. } = data {
                    self.sensor_data.igniter_oxidizer_injector_pressure_pa = Some(*pressure_pa);
                }
            },
            _ => {},
        }
    }
}
