use pyo3::prelude::*;

#[pyfunction]
pub fn simulate_app_replay(py: Python, simulation: PyObject, timestep_callback: PyObject) {
    println!("Starting simulation");

    let start_time = std::time::Instant::now();

    while simulation
        .call_method0(py, "advance_timestep")
        .expect("Failed to call advance_timestep on sim")
        .is_true(py)
        .unwrap_or(false)
    {
        if !timestep_callback.is_none(py) {
            if !timestep_callback
                .call1(py, (&simulation,))
                .expect("Failed to call timestep callback")
                .is_true(py)
                .unwrap_or(false)
            {
                break;
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let sim_elapsed = simulation
        .getattr(py, "t")
        .expect("Failed to get time t from sim")
        .extract::<f64>(py)
        .expect("Failed to extract time t as f64 from sim");

    println!(
        "Simulation finished in {:.2} seconds ({:.2}s simtime/realtime)",
        elapsed,
        sim_elapsed / elapsed
    );
    println!("Done! Replaying...");
    simulation
        .call_method0(py, "replay")
        .expect("Failed to call replay method on simulation");
    println!("Replay finished");
}
