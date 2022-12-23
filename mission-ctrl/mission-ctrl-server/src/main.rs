mod commands;
mod hardware;
mod recording;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, Arc, mpsc};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use commands::{valve, testvalve, testspark, pressurize, depressurize, fire};
use hal::comms_hal;
use hal::ecu_hal::{ECUSolenoidValve, ECUTelemetryFrame, ECUSensor};
use hardware::hardware_thread;
use recording::recording_thread;
use rocket::serde::{json::Json, Serialize};
use rocket::http::Header;
use rocket::{Request, Response, State};
use rocket::fairing::{Fairing, Info, Kind};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct HardwareState {
    state: String,
    in_default_state: bool,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct TelemetryData {
    igniter_fuel_pressure: Vec<f32>,
    igniter_gox_pressure: Vec<f32>,
    igniter_chamber_pressure: Vec<f32>,
    fuel_tank_pressure: Vec<f32>,
    ecu_board_temp: Vec<f32>,
    igniter_throat_temp: Vec<f32>,
    igniter_fuel_valve: HardwareState,
    igniter_gox_valve: HardwareState,
    fuel_press_valve: HardwareState,
    fuel_vent_valve: HardwareState,
    sparking: HardwareState,
    igniter_state: String,
    tank_state: String,
    telemetry_rate: u32,
    daq_rate: u32,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TelemetryQueue {
    packets: Mutex<Vec<comms_hal::Packet>>,
    telem_rate: AtomicU32,
    daq_rate: AtomicU32,
}

pub struct InitData {
}

#[macro_use] extern crate rocket;

#[get("/telemetry")]
fn telemetry(packets: &State<Arc<TelemetryQueue>>) -> Json<TelemetryData> {
    let packet_queue = packets.packets.lock().unwrap();
    let mut telem = TelemetryData {
        igniter_gox_pressure: Vec::new(),
        igniter_fuel_pressure: Vec::new(),
        igniter_chamber_pressure: Vec::new(),
        fuel_tank_pressure: Vec::new(),
        ecu_board_temp: Vec::new(),
        igniter_throat_temp: Vec::new(),
        igniter_fuel_valve: HardwareState { state: String::new(), in_default_state: false },
        igniter_gox_valve: HardwareState { state: String::new(), in_default_state: false },
        fuel_press_valve: HardwareState { state: String::new(), in_default_state: false },
        fuel_vent_valve: HardwareState { state: String::new(), in_default_state: false },
        sparking: HardwareState { state: String::new(), in_default_state: false },
        igniter_state: String::new(),
        tank_state: String::new(),
        telemetry_rate: packets.telem_rate.load(Ordering::Relaxed),
        daq_rate: packets.daq_rate.load(Ordering::Relaxed),
    };

    let valve_state = |valve: bool, flipped: bool| -> HardwareState {
        HardwareState { 
            state: String::from(if valve { "Open" } else { "Closed" }), 
            in_default_state: if flipped { valve } else { !valve },
        }
    };

    if let comms_hal::Packet::ECUTelemetry(frame) = packet_queue.last().unwrap() {
        telem.igniter_fuel_valve = valve_state(frame.solenoid_valves[ECUSolenoidValve::IgniterFuelMain as usize], false);
        telem.igniter_gox_valve = valve_state(frame.solenoid_valves[ECUSolenoidValve::IgniterGOxMain as usize], false);
        telem.fuel_press_valve = valve_state(frame.solenoid_valves[ECUSolenoidValve::FuelPress as usize], false);
        telem.fuel_vent_valve = valve_state(frame.solenoid_valves[ECUSolenoidValve::FuelVent as usize], true);
        telem.sparking = HardwareState {
            state: String::from(if frame.sparking { "On" } else { "Off" }), 
            in_default_state: !frame.sparking,
        };
        telem.igniter_state = String::from(format!("{:?}", frame.igniter_state));
        telem.tank_state = String::from(format!("{:?}", frame.fuel_tank_state));
    } else {
        panic!("Got incorrect packet in last index of telemetry packet queue: {:?}", packet_queue.last().unwrap());
    }

    for packet in packet_queue.iter() {
        if let comms_hal::Packet::ECUTelemetry(frame) = packet {
            telem.igniter_fuel_pressure.push(frame.sensors[ECUSensor::IgniterFuelInjectorPressure as usize]);
            telem.igniter_gox_pressure.push(frame.sensors[ECUSensor::IgniterGOxInjectorPressure as usize]);
            telem.igniter_chamber_pressure.push(frame.sensors[ECUSensor::IgniterChamberPressure as usize]);
            telem.fuel_tank_pressure.push(frame.sensors[ECUSensor::FuelTankPressure as usize]);
            telem.ecu_board_temp.push(frame.sensors[ECUSensor::ECUBoardTemp as usize]);
            telem.igniter_throat_temp.push(frame.sensors[ECUSensor::IgniterThroatTemp as usize]);
        } else {
            panic!("Got incorrect packet in telemetry packet queue: {:?}", packet_queue.last().unwrap());
        }
    }

    return Json(telem);
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
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    let packet_queue = Arc::new(TelemetryQueue { 
        packets: Mutex::new(vec!(comms_hal::Packet::ECUTelemetry(ECUTelemetryFrame::default()); 333)),
        telem_rate: AtomicU32::new(0),
        daq_rate: AtomicU32::new(0),
    });

    let (packet_tx, packet_rx) = mpsc::channel();
    let (recording_tx, recording_rx) = mpsc::channel();

    let packet_queue_ref = packet_queue.clone();
    thread::spawn(move || {
        hardware_thread(packet_queue_ref, packet_rx, recording_tx);
    });

    thread::spawn(move || {
        recording_thread(recording_rx);
    });
    
    rocket::build()
        .attach(CORS)
        .manage(InitData { })
        .manage(packet_queue.clone())
        .manage(Arc::new(Mutex::new(packet_tx)))
        .mount("/", routes![all_options, telemetry])
        .mount("/commands", routes![valve, testvalve, testspark, pressurize, depressurize, fire])
}

pub fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}