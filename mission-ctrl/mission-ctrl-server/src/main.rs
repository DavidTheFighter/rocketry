mod cameras;
mod commands;
mod comms;
mod config;
mod input;
mod logging;
pub(crate) mod observer;
mod telemetry;
mod terminal;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cameras::browser_stream;
use telemetry::ecu_telemetry::telemetry_thread;
use telemetry::fcu_telemetry::fcu_telemetry_thread;
use input::input_thread;
use observer::ObserverHandler;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Build, Request, Response, Rocket};

use crate::cameras::camera_streaming_thread;
use crate::comms::comms_thread;
use crate::config::config_thread;

#[macro_use]
extern crate rocket;

static PROCESS_RUNNING: AtomicBool = AtomicBool::new(true);

fn rocket(observer_handler: Arc<ObserverHandler>) -> Rocket<Build> {
    rocket::build()
        .attach(CORS)
        .manage(observer_handler)
        .mount(
            "/",
            routes![
                all_options,
                browser_stream,
                commands::send_packet,
                commands::packet_proxy,
            ],
        )
        .mount("/commands", commands::get_routes())
        .mount("/", telemetry::get_routes())
}

#[rocket::main]
async fn main() {
    let observer_handler = Arc::new(ObserverHandler::new());
    let rocket = rocket(observer_handler.clone()).ignite().await.unwrap();
    let shutdown_handle = rocket.shutdown();
    rocket::tokio::spawn(rocket.launch());

    let observer_handler_ref = observer_handler.clone();
    let comms_join_handle = thread::spawn(move || {
        comms_thread(observer_handler_ref);
    });

    // Ensure that the comms thread is running so we can send and receive data
    while observer_handler.get_num_observers() < 1 {
        thread::sleep(Duration::from_millis(10));
    }

    let mut join_handles = Vec::new();

    let observer_handler_ref = observer_handler.clone();
    join_handles.push(thread::spawn(move || {
        telemetry_thread(observer_handler_ref);
    }));

    let observer_handler_ref = observer_handler.clone();
    join_handles.push(thread::spawn(move || {
        fcu_telemetry_thread(observer_handler_ref);
    }));

    let observer_handler_ref = observer_handler.clone();
    join_handles.push(thread::spawn(move || {
        config_thread(observer_handler_ref);
    }));

    let observer_handler_ref = observer_handler.clone();
    join_handles.push(thread::spawn(move || {
        camera_streaming_thread(observer_handler_ref);
    }));

    join_handles.push(thread::spawn(move || {
        terminal::terminal_thread();
    }));

    let observer_handler_ref = observer_handler.clone();
    join_handles.push(thread::spawn(move || {
        logging::logging_thread(observer_handler_ref);
    }));

    let shutdown_handle_ref = shutdown_handle.clone();
    join_handles.push(thread::spawn(move || {
        input_thread(shutdown_handle_ref);
    }));

    // Wait for the server to shut down before exiting
    shutdown_handle.await;

    // Stop all threads
    for join_handle in join_handles {
        if let Err(err) = join_handle.join() {
            println!("Error joining thread: {:?}", err);
        }
    }

    // Add a small delay to ensure all remaining packets are handled
    thread::sleep(Duration::from_millis(250));

    comms_join_handle
        .join()
        .expect("Error joining comms thread");

    println!("Shut down gracefully!");
}

pub struct CORS;

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}

pub(crate) fn process_is_running() -> bool {
    PROCESS_RUNNING.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn stop_process() {
    PROCESS_RUNNING.store(false, std::sync::atomic::Ordering::Relaxed)
}
