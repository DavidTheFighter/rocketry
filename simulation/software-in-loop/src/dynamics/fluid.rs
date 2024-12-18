use pyo3::prelude::*;

#[derive(Debug, Clone)]
#[pyclass]
pub struct GasDefinition {
    pub name: String,
    #[pyo3(get)]
    pub molecular_weight_kg: f64,
    #[pyo3(get)]
    pub specific_heat_ratio: f64,
}

#[pymethods]
impl GasDefinition {
    #[new]
    pub fn new(name: String, molecular_weight_g: f64, specific_heat_ratio: f64) -> Self {
        Self {
            name,
            molecular_weight_kg: molecular_weight_g / 1000.0,
            specific_heat_ratio,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass]
pub struct LiquidDefinition {
    pub name: String,
    #[pyo3(get)]
    pub density_kg_m3: f64,
    #[pyo3(get)]
    pub vapor_pressure_pa: f64,
}

#[pymethods]
impl LiquidDefinition {
    #[new]
    pub fn new(name: String, density_kg_m3: f64, vapor_pressure_pa: f64) -> Self {
        Self {
            name,
            density_kg_m3,
            vapor_pressure_pa,
        }
    }
}
