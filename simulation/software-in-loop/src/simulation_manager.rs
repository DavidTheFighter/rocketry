use pyo3::prelude::*;

pub const REALTIME_WAIT_INTERVAL: f64 = 0.01;

#[pyfunction]
pub fn simulate_app(py: Python, simulation: PyObject, timestep_callback: PyObject, realtime: bool) {
    println!("Starting simulation");

    let start_time = std::time::Instant::now();
    let mut realtime_wait_counter = 0.0;
    let mut last_time = start_time;

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

        // Check for KeyboardInterrupt
        if let Some(err) = PyErr::take(py) {
            err.print(py);
            break;
        }

        if realtime {
            simulation.call_method0(py, "realtime_wait")
                .expect("Failed to call realtime_wait method on simulation");
        }

        if false && realtime {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_time).as_secs_f64();
            realtime_wait_counter += elapsed;

            if realtime_wait_counter > REALTIME_WAIT_INTERVAL {
                // Busy wait
                while std::time::Instant::now().duration_since(now).as_secs_f64()
                    < REALTIME_WAIT_INTERVAL
                {
                    // Busy wait
                }

                realtime_wait_counter -= REALTIME_WAIT_INTERVAL;
            }

            // let now = std::time::Instant::now();
            // let elapsed = now.duration_since(last_realtime_wait_time).as_secs_f64();

            // if elapsed > REALTIME_WAIT_INTERVAL {
            //     // Busy wait
            //     while std::time::Instant::now().duration_since(last_realtime_wait_time).as_secs_f64()
            //         < REALTIME_WAIT_INTERVAL
            //     {
            //         // Busy wait
            //     }

            //     last_realtime_wait_time = now + std::time::Duration::from_secs_f64(elapsed - REALTIME_WAIT_INTERVAL);
            // }
        }

        last_time = std::time::Instant::now();
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
