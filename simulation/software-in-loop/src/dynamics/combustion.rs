use super::{tank::GAS_CONSTANT, Scalar};
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone)]
pub struct CombustionData {
    #[pyo3(get, set)]
    pub mixture_ratio: Scalar, // Oxidizer to fuel ratio (O/F)
    #[pyo3(get, set)]
    pub molar_mass_kg_mol: Scalar,
    #[pyo3(get, set)]
    pub specific_heat_ratio: Scalar,
    #[pyo3(get, set)]
    pub chamber_temperature_k: Scalar,
}

#[pymethods]
impl CombustionData {
    #[new]
    pub fn new(
        mixture_ratio: Scalar,
        molar_mass_kg_mol: Scalar,
        specific_heat_ratio: Scalar,
        chamber_temperature_k: Scalar,
    ) -> Self {
        Self {
            mixture_ratio,
            molar_mass_kg_mol,
            specific_heat_ratio,
            chamber_temperature_k,
        }
    }
}

pub fn calc_chamber_pressure(
    mass_flow_kg_s: Scalar,
    throat_area_m2: Scalar,
    combustion_data: &CombustionData,
) -> Scalar {
    let throat_temp_k = combustion_data.chamber_temperature_k
        / (1.0
            + combustion_data.specific_heat_ratio * (combustion_data.specific_heat_ratio - 1.0)
                / 2.0);

    let throat_pressure_pa = mass_flow_kg_s
        * (GAS_CONSTANT * throat_temp_k
            / (combustion_data.molar_mass_kg_mol * combustion_data.specific_heat_ratio))
            .sqrt()
        / throat_area_m2;

    // println!("{:.2} = {} * {} / {}", throat_pressure_pa, mass_flow_kg_s, (GAS_CONSTANT * throat_temp_k / (combustion_data.molar_mass_kg_mol * combustion_data.specific_heat_ratio)).sqrt(), throat_area_m2);
    // println!("\t{} * {} / ({} * {})", GAS_CONSTANT, throat_temp_k, combustion_data.molar_mass_kg_mol, combustion_data.specific_heat_ratio);

    throat_pressure_pa
        / (1.0 + (combustion_data.specific_heat_ratio - 1.0) * 0.5).powf(
            -combustion_data.specific_heat_ratio / (combustion_data.specific_heat_ratio - 1.0),
        )
}

// pub const GAS_CONSTANT_UNIT: AvailableEnergy<> = 8.31446261815324;

// pub fn calc_chamber_pressure_si(
//     mass_flow_kg_s: MassRate<Scalar>,
//     throat_area_m2: Area<Scalar>,
//     combustion_data: &CombustionData,
// ) -> Pressure<Scalar> {
//     let throat_temp_k: TemperatureInterval<Scalar> = TemperatureInterval::new::<kelvin>(
//         combustion_data.chamber_temperature_k / (1.0 + combustion_data.specific_heat_ratio * (combustion_data.specific_heat_ratio - 1.0) / 2.0)
//     );

//     let throat_pressure_pa = mass_flow_kg_s
//         * (GAS_CONSTANT * throat_temp_k / (combustion_data.molar_mass_kg_mol * combustion_data.specific_heat_ratio)).sqrt()
//         / throat_area_m2;
// }
