use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shared::comms_hal::{Packet, NetworkAddress};
use shared::fcu_hal::{FcuTelemetryFrame, FcuDebugInfo};
use rocket::serde::{json::{Json, Value, json, serde_json::Map}, Serialize};

use crate::observer::{ObserverHandler, ObserverEvent};
use crate::{timestamp, process_is_running};

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DatasetEntry<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct FcuGraphData {
    altitude: VecDeque<f32>,
    y_velocity: VecDeque<f32>,
}

const FCU_GRAPH_DISPLAY_TIME_S: f32 = 20.0;
const FCU_VISUAL_UPDATES_PER_S: f32  = 5.0;
pub const FCU_GRAPH_MAX_DATA_POINTS: usize = (FCU_GRAPH_DISPLAY_TIME_S * FCU_VISUAL_UPDATES_PER_S) as usize;

static FCU_TELEMETRY_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);
static FCU_DEBUG_INFO_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);
static FCU_GRAPH_ENDPOINT_DATA: Mutex<Option<Value>> = Mutex::new(None);

struct FcuTelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    last_fcu_telemetry: FcuTelemetryFrame,
    last_debug_info: FcuDebugInfo,
    telemetry_rate_record_time: f64,
    last_telemetry_timestamp: f64,
    current_telemetry_rate_hz: u32,
}

impl FcuTelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            last_fcu_telemetry: FcuTelemetryFrame::default(),
            last_debug_info: FcuDebugInfo::default(),
            telemetry_rate_record_time: 1.0,
            last_telemetry_timestamp: timestamp(),
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
                    Packet::FcuTelemetry(frame) => {
                        self.last_fcu_telemetry = frame;
                        self.last_telemetry_timestamp = timestamp();
                        telemetry_counter += 1;
                    },
                    Packet::FcuDebugInfo(debug_info) => {
                        self.last_debug_info = debug_info;

                        let mut endpoint_data = FCU_DEBUG_INFO_ENDPOINT_DATA
                            .lock()
                            .expect("Failed to lock debug info");

                        if endpoint_data.is_none() {
                            endpoint_data.replace(Value::Object(rocket::serde::json::serde_json::Map::new()));
                        }

                        self.populate_debug_info(endpoint_data.as_mut().unwrap());
                    },
                    _ => {}
                }
            }

            let now = timestamp();
            if now - last_graph_update_time >= (1.0 / FCU_VISUAL_UPDATES_PER_S) as f64 {
                last_graph_update_time = now;

                FCU_TELEMETRY_ENDPOINT_DATA
                    .lock()
                    .expect("Failed to lock telemetry state")
                    .replace(self.populate_telemetry_endpoint());

                let mut graph_data = FCU_GRAPH_ENDPOINT_DATA
                    .lock()
                    .expect("Failed to lock FCU telemetry graph data")
                    .clone()
                    .unwrap_or(Value::Object(rocket::serde::json::serde_json::Map::new()));

                let graph_data_map = graph_data
                    .as_object_mut()
                    .expect("Failed to convert serde value to serde object");

                let graph_data_insert = self.populate_graph_frame();

                for (key, value) in graph_data_insert.iter() {
                    if !graph_data_map.contains_key(key) {
                        graph_data_map.insert(key.clone(), Value::Array(vec![value.clone(); FCU_GRAPH_MAX_DATA_POINTS - 1]));
                    }

                    let graph_data_vec = graph_data_map
                        .get_mut(key)
                        .expect("Failed to get graph data vec")
                        .as_array_mut()
                        .expect("Failed to convert graph data vec to array");

                    graph_data_vec.push(value.clone());

                    while graph_data_vec.len() >= FCU_GRAPH_MAX_DATA_POINTS {
                        graph_data_vec.remove(0);
                    }
                }

                FCU_GRAPH_ENDPOINT_DATA
                    .lock()
                    .expect("Failed to lock FCU telemetry graph data")
                    .replace(graph_data);
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
        let mut telemetry_frame = rocket::serde::json::to_value(&self.last_fcu_telemetry)
            .expect("Failed to convert telemetry frame to serde value");

        let telemetry_frame_map = telemetry_frame
            .as_object_mut()
            .expect("Failed to convert serde value to serde object");

        let telemetry_delta_t = (timestamp() - self.last_telemetry_timestamp) as i32;

        telemetry_frame_map.insert(String::from("telemetry_rate"), Value::Number(self.current_telemetry_rate_hz.into()));
        telemetry_frame_map.insert(String::from("telemetry_delta_t"), Value::Number(telemetry_delta_t.into()));

        telemetry_frame

    }

    fn populate_debug_info(&self, existing_data: &mut Value) {
        let debug_info = rocket::serde::json::to_value(&self.last_debug_info)
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

    fn populate_graph_frame(&self) -> Map<String, Value> {
        let mut graph_data = rocket::serde::json::serde_json::Map::new();

        graph_data.insert(String::from("altitude"), json!(self.last_fcu_telemetry.position.y));
        graph_data.insert(String::from("y_velocity"), json!(self.last_fcu_telemetry.velocity.y));

        graph_data
    }

    fn get_packet(&self) -> Option<Packet> {
        let timeout = Duration::from_millis(1);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::PacketReceived { address: _, ip: _, packet } = event {
                return Some(packet);
            }
        }

        None
    }
}

pub fn fcu_telemetry_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();

    FcuTelemetryHandler::new(observer_handler).run();
}

#[get("/fcu-telemetry")]
pub fn fcu_telemetry_endpoint<'a>() -> Json<Value> {
    let latest_telemetry = FCU_TELEMETRY_ENDPOINT_DATA.lock().expect("Failed to lock telemetry state");

    if let Some(latest_telemetry) = latest_telemetry.as_ref() {
        let telem = latest_telemetry.clone();
        Json(telem.clone())
    } else {
        Json(Value::Null)
    }
}

#[get("/fcu-telemetry/graph")]
pub fn fcu_telemetry_graph() -> Json<Value> {
    let latest_graph = FCU_GRAPH_ENDPOINT_DATA.lock().expect("Failed to lock FCU telemetry graph data");

    if let Some(latest_graph) = latest_graph.as_ref() {
        let graph_data = latest_graph.clone();
        Json(graph_data.clone())
    } else {
        Json(Value::Null)
    }
}

#[get("/fcu-telemetry/debug-data")]
pub fn fcu_debug_data() -> Json<Value> {
    let latest_debug_info = FCU_DEBUG_INFO_ENDPOINT_DATA.lock().expect("Failed to lock debug info");

    if let Some(debug_info) = latest_debug_info.as_ref() {
        let debug_info = debug_info.clone();
        Json(debug_info.clone())
    } else {
        Json(Value::Null)
    }
}
