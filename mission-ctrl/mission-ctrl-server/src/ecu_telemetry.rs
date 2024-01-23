use std::sync::{Arc, Mutex};
use std::time::Duration;

use rocket::serde::json::Value;
use rocket::serde::{json::Json, Serialize};
use shared::comms_hal::Packet;

use crate::observer::{ObserverEvent, ObserverHandler};
use crate::{process_is_running, timestamp};

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct HardwareState {
    state: String,
    in_default_state: bool,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct TelemetryData {
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
    cpu_utilization: u32,
    daq_rate: u32,
}

static TELEMETRY_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);

struct TelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    telemetry_rate_record_time: f64,
    current_telemetry_rate_hz: u32,
    current_daq_rate_hz: u32,
}

impl TelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            telemetry_rate_record_time: 0.25,
            current_telemetry_rate_hz: 0,
            current_daq_rate_hz: 0,
        }
    }

    pub fn run(&mut self) {
        let mut telemetry_counter = 0;
        let mut daq_counter = 0;
        let mut last_rate_record_time = timestamp();

        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                match packet {
                    Packet::EcuTelemetry(frame) => {
                        telemetry_counter += 1;
                    }
                    _ => {}
                }
            }

            let now = timestamp();

            if now - last_rate_record_time >= self.telemetry_rate_record_time {
                last_rate_record_time = now;

                let telem_rate = (telemetry_counter as f64) / self.telemetry_rate_record_time;
                let daq_rate = (daq_counter as f64) / self.telemetry_rate_record_time;

                self.current_telemetry_rate_hz = telem_rate as u32;
                self.current_daq_rate_hz = daq_rate as u32;
                telemetry_counter = 0;
                daq_counter = 0;
            }
        }
    }

    fn get_packet(&self) -> Option<Packet> {
        let timeout = Duration::from_millis(1);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::PacketReceived {
                address: _,
                ip: _,
                packet,
            } = event
            {
                return Some(packet);
            }
        }

        None
    }
}

pub fn telemetry_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();

    TelemetryHandler::new(observer_handler).run();
}

#[get("/ecu-telemetry")]
pub fn ecu_telemetry_endpoint() -> Json<Value> {
    let telemetry = TELEMETRY_ENDPOINT_DATA
        .lock()
        .expect("Failed to lock telemetry state");

    if let Some(telemetry) = telemetry.as_ref() {
        Json(telemetry.clone())
    } else {
        Json(Value::String(String::from("No telemetry data")))
    }
}
