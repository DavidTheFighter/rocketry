use std::sync::Mutex;

use rocket::serde::json::Value;

pub mod ecu_telemetry;
pub mod fcu_telemetry;

use rocket::serde::{json::{json, Json, serde_json::Map}, Serialize};

const GRAPH_DISPLAY_TIME_S: f32 = 20.0;
const VISUAL_UPDATES_PER_S: f32 = 10.0;
const GRAPH_MAX_DATA_POINTS: usize = (GRAPH_DISPLAY_TIME_S * VISUAL_UPDATES_PER_S) as usize;

fn populate_graph_data(
    endpoint_data_mutex: &Mutex<Option<Value>>,
    graph_data: Map<String, Value>,
) {
    let mut endpoint_data = endpoint_data_mutex
        .lock()
        .expect("Failed to lock ECU telemetry graph data")
        .clone()
        .unwrap_or(Value::Object(rocket::serde::json::serde_json::Map::new()));

    let endpoint_data_map = endpoint_data
        .as_object_mut()
        .expect("Failed to convert serde value to serde object");

    for (key, value) in graph_data.iter() {
        if !endpoint_data_map.contains_key(key) {
            endpoint_data_map.insert(
                key.clone(),
                Value::Array(vec![value.clone(); GRAPH_MAX_DATA_POINTS - 1]),
            );
        }

        let graph_data_vec = endpoint_data_map
            .get_mut(key)
            .expect("Failed to get graph data vec")
            .as_array_mut()
            .expect("Failed to convert graph data vec to array");

        graph_data_vec.push(value.clone());

        while graph_data_vec.len() >= GRAPH_MAX_DATA_POINTS {
            graph_data_vec.remove(0);
        }
    }

    endpoint_data_mutex
        .lock()
        .expect("Failed to lock ECU telemetry graph data")
        .replace(endpoint_data);
}
