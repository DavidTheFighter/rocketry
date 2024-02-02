use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rocket::serde::json::Value;
use rocket::serde::{json::{json, Json, serde_json::Map}, Serialize};
use shared::comms_hal::Packet;
use shared::ecu_hal::{EcuDebugInfo, EcuTelemetryFrame};

use crate::observer::{ObserverEvent, ObserverHandler};
use crate::{process_is_running, timestamp};

use super::{populate_graph_data, VISUAL_UPDATES_PER_S};

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct HardwareState {
    state: String,
    in_default_state: bool,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DatasetEntry<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct EcuGraphData {
    fuel_tank_pressure: VecDeque<f32>,
    oxygen_tank_pressure: VecDeque<f32>,
    igniter_chamber_pressure: VecDeque<f32>,
}

static TELEMETRY_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);
static DEBUG_INFO_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);
static GRAPH_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);

struct TelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    last_ecu_telemetry: Option<EcuTelemetryFrame>,
    last_debug_info: Option<EcuDebugInfo>,
    telemetry_rate_record_time: f64,
    current_telemetry_rate_hz: u32,
}

impl TelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            last_ecu_telemetry: None,
            last_debug_info: None,
            telemetry_rate_record_time: 0.25,
            current_telemetry_rate_hz: 0,
        }
    }

    pub fn run(&mut self) {
        let mut telemetry_counter = 0;
        let mut last_graph_update_time = timestamp();
        let mut last_rate_record_time = timestamp();

        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                match packet {
                    Packet::EcuTelemetry(frame) => {
                        telemetry_counter += 1;
                        self.last_ecu_telemetry = Some(frame);
                    },
                    Packet::EcuDebugInfo(debug_info) => {
                        self.last_debug_info = Some(debug_info);

                        let mut endpoint_data = DEBUG_INFO_ENDPOINT_DATA
                            .lock()
                            .expect("Failed to lock debug info");

                        if endpoint_data.is_none() {
                            endpoint_data.replace(Value::Object(
                                rocket::serde::json::serde_json::Map::new(),
                            ));
                        }

                        self.populate_debug_info(endpoint_data.as_mut().unwrap());
                    },
                    _ => {}
                }
            }

            let now = timestamp();
            if now - last_graph_update_time >= (1.0 / VISUAL_UPDATES_PER_S) as f64 {
                last_graph_update_time = now;

                TELEMETRY_ENDPOINT_DATA
                    .lock()
                    .expect("Failed to lock telemetry state")
                    .replace(self.populate_telemetry_endpoint());

                populate_graph_data(&GRAPH_ENDPOINT_DATA, self.populate_graph_frame());
            }

            if now - last_rate_record_time >= self.telemetry_rate_record_time {
                last_rate_record_time = now;

                let telem_rate = (telemetry_counter as f64) / self.telemetry_rate_record_time;

                self.current_telemetry_rate_hz = telem_rate as u32;
                telemetry_counter = 0;
            }
        }
    }

    fn populate_telemetry_endpoint(&self) -> Value {
        if let Some(last_ecu_telemetry) = &self.last_ecu_telemetry {
            let mut telemetry_frame = rocket::serde::json::to_value(last_ecu_telemetry)
                .expect("Failed to convert telemetry frame to serde value");

            let telemetry_frame_map = telemetry_frame
                .as_object_mut()
                .expect("Failed to convert serde value to serde object");

            telemetry_frame_map.insert(
                String::from("telemetry_rate"),
                Value::Number(self.current_telemetry_rate_hz.into()),
            );

            telemetry_frame
        } else {
            Value::Null
        }
    }

    fn populate_debug_info(&self, existing_data: &mut Value) {
        if let Some(last_debug_info) = &self.last_debug_info {
            let debug_info = rocket::serde::json::to_value(last_debug_info)
                .expect("Failed to convert debug info to serde value");

            let existing_data = existing_data
                .as_object_mut()
                .expect("Failed to convert serde value to serde object");

            for value in debug_info.as_object().unwrap().values() {
                for (key, value) in value.as_object().unwrap() {
                    existing_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    fn populate_graph_frame(&self) -> Map<String, Value> {
        let mut graph_data = rocket::serde::json::serde_json::Map::new();

        if let Some(last_ecu_telemetry) = &self.last_ecu_telemetry {
            graph_data.insert(
                String::from("fuel_tank_pressure_psi"),
                json!(last_ecu_telemetry.fuel_tank_pressure_pa / 6894.75729),
            );
            graph_data.insert(
                String::from("oxidizer_tank_pressure_psi"),
                json!(last_ecu_telemetry.oxidizer_tank_pressure_pa / 6894.75729),
            );
            graph_data.insert(
                String::from("igniter_chamber_pressure_psi"),
                json!(last_ecu_telemetry.igniter_chamber_pressure_pa / 6894.75729),
            );
        }

        graph_data
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

#[get("/ecu-telemetry/graph")]
pub fn ecu_telemetry_graph() -> Json<Value> {
    let latest_graph = GRAPH_ENDPOINT_DATA
        .lock()
        .expect("Failed to lock FCU telemetry graph data");

    if let Some(latest_graph) = latest_graph.as_ref() {
        let graph_data = latest_graph.clone();
        Json(graph_data.clone())
    } else {
        Json(Value::Null)
    }
}

#[get("/ecu-telemetry/debug-data")]
pub fn ecu_debug_data() -> Json<Value> {
    let latest_debug_info = DEBUG_INFO_ENDPOINT_DATA
        .lock()
        .expect("Failed to lock debug info");

    if let Some(debug_info) = latest_debug_info.as_ref() {
        let debug_info = debug_info.clone();
        Json(debug_info.clone())
    } else {
        Json(Value::Null)
    }
}
