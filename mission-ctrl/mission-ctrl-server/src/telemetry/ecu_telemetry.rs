use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rocket::futures::stream::FusedStream;
use rocket::futures::SinkExt;
use rocket::serde::json::Value;
use rocket::serde::json::{json, serde_json::Map};
use rocket::State;
use shared::alerts::{self, AlertBitmaskType};
use shared::comms_hal::{NetworkAddress, Packet};
use shared::ecu_hal::{EcuAlert, EcuTelemetry};

use strum::{EnumProperty, IntoEnumIterator};

use crate::observer::{ObserverEvent, ObserverHandler};
use crate::{process_is_running, timestamp};

struct EcuTelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    telemetry_publish_rate_s: f64,
    telemetry_rate_record_time: f64,
    current_telemetry_rate_hz: u32,
    telemetry_counter: u32,
}

impl EcuTelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            telemetry_publish_rate_s: 0.033,
            telemetry_rate_record_time: 0.5,
            current_telemetry_rate_hz: 0,
            telemetry_counter: 0,
        }
    }

    pub fn run(&mut self) {
        let mut last_rate_record_time = timestamp();

        let mut current_telemetry_data = HashMap::new();

        self.observer_handler.register_observer_thread();

        while process_is_running() {
            let now = timestamp();
            if now - last_rate_record_time >= self.telemetry_rate_record_time {
                last_rate_record_time = now;

                let telem_rate = (self.telemetry_counter as f64) / self.telemetry_rate_record_time;

                self.current_telemetry_rate_hz = telem_rate as u32;
                self.telemetry_counter = 0;
            }

            let display_fields = json!({
                "telemetry_rate_hz": self.current_telemetry_rate_hz,
            });

            let telemetry = match self.gather_telemetry(
                Duration::from_secs_f64(self.telemetry_publish_rate_s),
                &mut current_telemetry_data,
                &display_fields,
            ) {
                Ok(telemetry) => telemetry,
                Err(err) => {
                    eprintln!("Failed to gather telemetry data: {}", err);
                    continue;
                }
            };

            current_telemetry_data = telemetry.clone();

            for (ecu_index, telemetry_data) in telemetry {
                let json_str = rocket::serde::json::to_string(&telemetry_data)
                    .expect("Failed to convert telemetry data to JSON string");

                self.observer_handler
                    .notify(ObserverEvent::AggregateTelemetry {
                        controller: NetworkAddress::EngineController(ecu_index),
                        json: json_str,
                    });
            }
        }
    }

    fn gather_telemetry(
        &mut self,
        duration: Duration,
        previous_telemetry_data: &mut HashMap<u8, Map<String, Value>>,
        display_fields: &Value,
    ) -> Result<HashMap<u8, Map<String, Value>>, String> {
        let mut telemetry_data = previous_telemetry_data.clone();
        let mut ecu_sensor_datas = HashMap::new();
        let mut received_data = false;

        let start_time = std::time::Instant::now();

        // Run loop for duration of time
        while start_time.elapsed() < duration {
            if let Some((remote, packet)) = self.get_packet() {
                let recv_ecu_index = if let NetworkAddress::EngineController(index) = remote {
                    index
                } else {
                    return Err(String::from("Received packet from non-ECU address"));
                };

                let ecu_data: &mut Map<String, Value> = telemetry_data
                    .entry(recv_ecu_index)
                    .or_insert(rocket::serde::json::serde_json::Map::new());

                let sensor_data: &mut Map<String, Value> = ecu_sensor_datas
                    .entry(recv_ecu_index)
                    .or_insert(rocket::serde::json::serde_json::Map::new());

                match packet {
                    Packet::EcuTelemetry(EcuTelemetry::Telemetry(frame)) => {
                        let telemetry_value = rocket::serde::json::to_value(&frame)
                            .expect("Failed to convert telemetry frame to serde value");

                        ecu_data.insert(String::from("telemetry"), telemetry_value);

                        self.telemetry_counter += 1;
                    }
                    Packet::EcuTelemetry(EcuTelemetry::TankTelemetry(frame)) => {
                        let telemetry_value = rocket::serde::json::to_value(&frame)
                            .expect("Failed to convert telemetry frame to serde value");

                        ecu_data.insert(String::from("tank_telemetry"), telemetry_value);
                    }
                    Packet::EcuTelemetry(EcuTelemetry::DebugInfo(debug_info)) => {
                        let debug_info_value = rocket::serde::json::to_value(&debug_info)
                            .expect("Failed to convert telemetry frame to serde value");

                        ecu_data.insert(String::from("debug_info"), debug_info_value);
                    }
                    Packet::EcuTelemetry(EcuTelemetry::DebugSensorMeasurement((sensor, data))) => {
                        match data {
                            shared::SensorData::Pressure {
                                pressure_pa,
                                raw_data: _,
                            } => {
                                sensor_data.insert(format!("{:?}", sensor), json!(pressure_pa));
                            }
                            shared::SensorData::Temperature {
                                temperature_k,
                                raw_data: _,
                            } => {
                                sensor_data
                                    .insert(format!("{:?}", sensor), json!(temperature_k + 273.15));
                            }
                        }
                    }
                    Packet::AlertBitmask(bitmask) => {
                        let alert_conditions = ecu_data
                            .entry(String::from("alert_conditions"))
                            .or_insert(Value::Array(Vec::new()))
                            .as_array_mut()
                            .expect("Failed to get alert conditions as array");

                        alert_conditions.clear();

                        for condition in EcuAlert::iter() {
                            if alerts::is_condition_set(bitmask, condition as AlertBitmaskType) {
                                let mut alert_value = rocket::serde::json::serde_json::Map::new();
                                alert_value.insert(
                                    String::from("alert"),
                                    json!(format!("{:?}", condition)),
                                );
                                alert_value.insert(
                                    String::from("severity"),
                                    json!(condition.get_str("severity").unwrap()),
                                );
                                alert_conditions.push(Value::Object(alert_value));
                            }
                        }
                    }
                    _ => continue,
                }

                received_data = true;
            }
        }

        if !received_data {
            telemetry_data.clear();
            let ecu_data = telemetry_data
                .entry(0)
                .or_insert(rocket::serde::json::serde_json::Map::new());

            ecu_data.insert(String::from("display_fields"), display_fields.clone());
            ecu_data.insert(
                String::from("noHistoryFields"),
                json!(vec![String::from("display_fields"),]),
            );

            return Ok(telemetry_data);
        }

        for (ecu_index, sensor_data) in ecu_sensor_datas {
            let ecu_data = telemetry_data.get_mut(&ecu_index).unwrap();

            ecu_data.insert(String::from("sensors"), Value::Object(sensor_data));
            ecu_data.insert(String::from("display_fields"), display_fields.clone());
            ecu_data.insert(
                String::from("noHistoryFields"),
                json!(vec![
                    String::from("alert_conditions"),
                    String::from("display_fields"),
                ]),
            );
        }

        Ok(telemetry_data)
    }

    fn get_packet(&self) -> Option<(NetworkAddress, Packet)> {
        let timeout = Duration::from_millis(1);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::PacketReceived {
                address,
                ip: _,
                packet,
            } = event
            {
                return Some((address, packet));
            }
        }

        None
    }
}

pub fn telemetry_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();

    EcuTelemetryHandler::new(observer_handler).run();
}

#[get("/ecu-telemetry-stream/<ecu_id>")]
pub fn ecu_telemetry_stream(
    ws: ws::WebSocket,
    ecu_id: u8,
    observer_handler: &State<Arc<ObserverHandler>>,
) -> ws::Channel<'static> {
    let observer_handler = observer_handler.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let filter_telemetry_fn = move |event: &ObserverEvent| {
                if let ObserverEvent::AggregateTelemetry {
                    controller,
                    json: _,
                } = event
                {
                    if let NetworkAddress::EngineController(ecu_index) = controller {
                        return *ecu_index == ecu_id;
                    }
                }

                false
            };

            while process_is_running() {
                if observer_handler.register_observer_thread() {
                    observer_handler
                        .register_subscription_filter("ecu_telemetry_stream", filter_telemetry_fn);
                }

                if let Some((_, event)) = observer_handler.wait_event(Duration::from_millis(1)) {
                    if !filter_telemetry_fn(&event) {
                        continue;
                    }

                    if let ObserverEvent::AggregateTelemetry {
                        controller: _,
                        json,
                    } = event
                    {
                        let result = stream.send(ws::Message::Text(json)).await;

                        if result.is_err() || stream.is_terminated() {
                            return Ok(());
                        }
                    }
                }
            }

            Ok(())
        })
    })
}
