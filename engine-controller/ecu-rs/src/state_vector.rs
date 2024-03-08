use serde::Serialize;
use shared::ecu_hal::EcuSensor;

#[derive(Debug, Clone, Serialize)]
pub struct SensorData {
    pub fuel_tank_pressure_pa: f32, // Pa
    pub oxidizer_tank_pressure_pa: f32, // Pa
    pub igniter_chamber_pressure_pa: f32, // Pa
    pub igniter_fuel_injector_pressure_pa: Option<f32>, // Pa
    pub igniter_oxidizer_injector_pressure_pa: Option<f32>, // Pa
}

#[derive(Debug, Clone, Serialize)]
pub struct StateVector {
    pub(crate) sensor_data: SensorData,
}

impl StateVector {
    pub fn new() -> Self {
        Self {
            sensor_data: SensorData {
                fuel_tank_pressure_pa: 0.0,
                oxidizer_tank_pressure_pa: 0.0,
                igniter_chamber_pressure_pa: 0.0,
                igniter_fuel_injector_pressure_pa: None,
                igniter_oxidizer_injector_pressure_pa: None,
            },
        }
    }

    pub fn update_sensor_data(&mut self, data: &EcuSensor) {
        match data {
            EcuSensor::FuelTankPressure(data) => {
                self.sensor_data.fuel_tank_pressure_pa = data.pressure_pa;
            },
            EcuSensor::OxidizerTankPressure(data) => {
                self.sensor_data.oxidizer_tank_pressure_pa = data.pressure_pa;
            },
            EcuSensor::IgniterChamberPressure(data) => {
                self.sensor_data.igniter_chamber_pressure_pa = data.pressure_pa;
            },
            EcuSensor::IgniterFuelInjectorPressure(data) => {
                self.sensor_data.igniter_fuel_injector_pressure_pa = Some(data.pressure_pa);
            },
            EcuSensor::IgniterOxidizerInjectorPressure(data) => {
                self.sensor_data.igniter_oxidizer_injector_pressure_pa = Some(data.pressure_pa);
            },
            _ => {},
        }
    }
}
