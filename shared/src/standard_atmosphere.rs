
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

pub fn convert_altitude_to_pressure(altitude_m: f32, temperature_c: f32) -> f32 {
    let temperature_k = temperature_c + 273.15;
    let temperature_ratio = temperature_k / 288.15;

    let a = altitude_m / (44330.0 * powf(temperature_ratio, -0.03416));
    let b = 1.0 - a;
    let c = powf(b, 1.0 / 0.1903);

    let pressure_mbar = c * 1013.25;
    let pressure_pa = pressure_mbar * 100.0;

    pressure_pa
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_pressure_to_altitude() {
        let pressure_pa = 101325.0;
        let temperature_c = 15.0;

        let altitude_m = convert_pressure_to_altitude(pressure_pa, temperature_c);

        assert!((altitude_m - 0.0).abs() < 1e-3);
    }

    #[test]
    fn test_convert_altitude_to_pressure() {
        let altitude_m = 0.0;
        let temperature_c = 15.0;

        let pressure_pa = convert_altitude_to_pressure(altitude_m, temperature_c);

        println!("pressure_pa: {}", pressure_pa);
        assert!((pressure_pa - 101325.0).abs() < 1e-3);
    }
}