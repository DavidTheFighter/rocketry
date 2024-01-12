pub mod dynamics;
pub mod fcu;
pub mod logging;
pub mod mission_ctrl;
pub mod network;
pub mod ser;

use pyo3::prelude::*;

#[pymodule]
fn software_in_loop(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<fcu::FcuSil>()?;
    m.add_class::<mission_ctrl::MissionControl>()?;

    m.add_class::<dynamics::SilVehicleDynamics>()?;
    m.add_class::<logging::Logger>()?;

    m.add_class::<network::SilNetwork>()?;
    m.add_class::<network::SilNetworkPhy>()?;
    m.add_class::<network::SilNetworkIface>()?;

    m.add_function(wrap_pyfunction!(logging::load_logs_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(fcu::convert_altitude_to_pressure, m)?)?;
    m.add_function(wrap_pyfunction!(fcu::convert_pressure_to_altitude, m)?)?;

    Ok(())
}
