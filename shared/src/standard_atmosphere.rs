
use libm::powf;

pub fn convert_pressure_to_altitude(pressure_pa: f32, temperature_c: f32) -> f32 {
    let pressure_mbar = pressure_pa / 100.0;
    let temperature_k = temperature_c + 273.15;

    let pressure_ratio = pressure_mbar / 1013.25;
    let temperature_ratio = temperature_k / 288.15;

    let altitude_m = 44330.0 * (1.0 - powf(pressure_ratio, 0.1903));
    let altitude_m = altitude_m * powf(temperature_ratio, -0.03416);

    altitude_m
}
