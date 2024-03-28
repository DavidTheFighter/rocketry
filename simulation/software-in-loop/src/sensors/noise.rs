use rand_distr::Distribution;
use shared::SensorData;


pub trait SensorNoise {
    fn update(&mut self, value: f64, dt: f64) -> Option<SensorData>;
}

pub struct LinearVoltagePressureTranducer {
    sensor_value: f64,
    time_since_last_update: f64,
    time_per_update_s: f64,
    pressure_min: f64,
    pressure_max: f64,
    raw_min: u16,
    raw_max: u16,
    pressure_noise_std_dev: f64,
    normal_distr: rand_distr::Normal<f64>,
}

impl LinearVoltagePressureTranducer {
    pub fn new(
        time_per_update_s: f64,
        pressure_min: f64,
        pressure_max: f64,
        raw_min: u16,
        raw_max: u16,
        pressure_noise_std_dev: f64,
    ) -> Self {
        Self {
            sensor_value: 0.0,
            time_since_last_update: 0.0,
            time_per_update_s,
            pressure_min,
            pressure_max,
            raw_min,
            raw_max,
            pressure_noise_std_dev,
            normal_distr: rand_distr::Normal::new(0.0, pressure_noise_std_dev).unwrap(),
        }
    }
}

impl SensorNoise for LinearVoltagePressureTranducer {
    fn update(&mut self, value: f64, dt: f64) -> Option<SensorData> {
        self.sensor_value = value;

        self.time_since_last_update += dt;
        if self.time_since_last_update < self.time_per_update_s {
            return None;
        }

        let noisy_value = self.sensor_value + self.normal_distr.sample(&mut rand::thread_rng());

        self.time_since_last_update = 0.0;
        let lerp = (noisy_value- self.pressure_min) / (self.pressure_max - self.pressure_min);
        let raw = (lerp * ((self.raw_max - self.raw_min) as f64)) as u16 + self.raw_min;

        Some(SensorData::Pressure { pressure_pa: noisy_value as f32, raw_data: raw })
    }
}

pub struct LinearVoltageTemperatureSensor {
    sensor_value: f64,
    time_since_last_update: f64,
    time_per_update_s: f64,
    temperature_min: f64,
    temperature_max: f64,
    raw_min: u16,
    raw_max: u16,
    temperature_noise_std_dev: f64,
}

impl LinearVoltageTemperatureSensor {
    pub fn new(
        time_per_update_s: f64,
        temperature_min: f64,
        temperature_max: f64,
        raw_min: u16,
        raw_max: u16,
        temperature_noise_std_dev: f64,
    ) -> Self {
        Self {
            sensor_value: 0.0,
            time_since_last_update: 0.0,
            time_per_update_s,
            temperature_min,
            temperature_max,
            raw_min,
            raw_max,
            temperature_noise_std_dev,
        }
    }
}

impl SensorNoise for LinearVoltageTemperatureSensor {
    fn update(&mut self, value: f64, dt: f64) -> Option<SensorData> {
        self.sensor_value = value;

        self.time_since_last_update += dt;
        if self.time_since_last_update < self.time_per_update_s {
            return None;
        }

        self.time_since_last_update = 0.0;
        let lerp = (self.sensor_value - self.temperature_min) / (self.temperature_max - self.temperature_min);
        let raw = (lerp * ((self.raw_max - self.raw_min) as f64)) as u16 + self.raw_min;

        Some(SensorData::Temperature { temperature_k: self.sensor_value as f32, raw_data: raw })
    }
}
