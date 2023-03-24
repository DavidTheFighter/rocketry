mod commands;
mod comms;
mod config;
mod input;
pub(crate) mod observer;
mod telemetry;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use input::input_thread;
use observer::ObserverHandler;
use comms::recv::recv_thread;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
// use rocket::serde::{json::Json, Serialize};
use rocket::{Request, Response, Rocket, Build};
use comms::send::send_thread;
use telemetry::{telemetry_thread, ecu_telemetry_endpoint};

use crate::config::config_thread;

#[macro_use]
extern crate rocket;

static PROCESS_RUNNING: AtomicBool = AtomicBool::new(true);

fn rocket(observer_handler: Arc<ObserverHandler>) -> Rocket<Build> {
    rocket::build()
        .attach(CORS)
        .manage(observer_handler)
        .mount("/", routes![
            all_options,
            ecu_telemetry_endpoint,
        ])
        .mount("/commands", commands::get_routes())
}

#[rocket::main]
async fn main() {
    let observer_handler = Arc::new(ObserverHandler::new());
    let rocket = rocket(observer_handler.clone()).ignite().await.unwrap();
    let shutdown_handle = rocket.shutdown();
    rocket::tokio::spawn(rocket.launch());

    let observer_handler_ref = observer_handler.clone();
    thread::spawn(move || {
        recv_thread(observer_handler_ref);
    });

    let observer_handler_ref = observer_handler.clone();
    thread::spawn(move || {
        send_thread(observer_handler_ref);
    });

    // Ensure that the recv and send threads are running so we can send and receive data
    while observer_handler.get_num_observers() < 2 {
        thread::sleep(Duration::from_millis(10));
    }

    let observer_handler_ref = observer_handler.clone();
    thread::spawn(move || {
        telemetry_thread(observer_handler_ref);
    });

    let observer_handler_ref = observer_handler.clone();
    thread::spawn(move || {
        config_thread(observer_handler_ref);
    });

    let shutdown_handle_ref = shutdown_handle.clone();
    thread::spawn(move || {
        input_thread(shutdown_handle_ref);
    });

    // Wait for the server to shut down before exiting
    shutdown_handle.await;

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