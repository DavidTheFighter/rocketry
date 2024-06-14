pub mod dynamics;
pub mod ecu;
pub mod fcu;
// pub mod glue;
pub mod logging;
pub mod mission_ctrl;
pub mod network;
pub mod sensors;
pub mod ser;
pub mod simulation_manager;

use pyo3::prelude::*;

extern crate uom;

#[pymodule]
fn software_in_loop(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_class::<glue::SilGlue>()?;
    m.add_class::<ecu::EcuSil>()?;
    m.add_class::<fcu::FcuSil>()?;
    m.add_class::<mission_ctrl::MissionControl>()?;

    m.add_class::<dynamics::DynamicsManager>()?;
    m.add_class::<logging::Logger>()?;

    m.add_class::<dynamics::SilTankDynamics>()?;
    m.add_class::<dynamics::SilTankFeedConfig>()?;
    m.add_class::<dynamics::SilVehicleDynamics>()?;
    m.add_class::<dynamics::combustion::CombustionData>()?;
    m.add_class::<dynamics::igniter::SilIgniterDynamics>()?;
    m.add_class::<dynamics::engine::SilEngineDynamics>()?;
    m.add_class::<dynamics::pump::SilPumpDynamics>()?;

    m.add_class::<dynamics::InjectorConfig>()?;
    m.add_class::<dynamics::pipe::FluidConnection>()?;
    m.add_class::<dynamics::pipe_splitter::FluidSplitter>()?;
    m.add_class::<dynamics::fluid::GasDefinition>()?;
    m.add_class::<dynamics::fluid::LiquidDefinition>()?;

    m.add_class::<network::SilNetwork>()?;
    m.add_class::<network::SilNetworkPhy>()?;
    m.add_class::<network::SilNetworkIface>()?;
    m.add_class::<network::SimBridgeIface>()?;

    m.add_function(wrap_pyfunction!(
        simulation_manager::simulate_app_replay,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(logging::load_logs_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(fcu::convert_altitude_to_pressure, m)?)?;
    m.add_function(wrap_pyfunction!(fcu::convert_pressure_to_altitude, m)?)?;

    m.add("ATMOSPHERIC_PRESSURE_PA", dynamics::ATMOSPHERIC_PRESSURE_PA)?;

    Ok(())
}
