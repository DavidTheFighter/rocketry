use std::io;

use rocket::Shutdown;

use crate::stop_process;

pub fn input_thread(shutdown_handle: Shutdown) {
    let mut buffer = String::new();
    let stdin = io::stdin();

    println!("Press enter to exit");

    stdin.read_line(&mut buffer).unwrap();

    println!("Shutting down...");

    stop_process();

    shutdown_handle.notify();
}
