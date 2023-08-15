use std::sync::{Arc, Mutex};
use std::time::Duration;

use hal::comms_hal::Packet;
use hal::fcu_hal::FcuTelemetryFrame;
use rocket::serde::{json::Json, Serialize};

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
pub struct FcuTelemetryGraphData {
    altitude: Vec<f32>,
    y_velocity: Vec<f32>,
}

#[derive(Debug, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct FcuTelemetryData<'a> {
    vehicle_state: String,
    telemetry_rate: u32,
    telemetry_delta_t: f32,
    orientation: Vec<f32>,
    acceleration: Vec<f32>,
    angular_velocity: Vec<f32>,
    magnetic_field: Vec<f32>,
    speed: f32,
    output_channels: Vec<bool>,
    pwm_channels: Vec<f32>,
    battery_voltage: f32,
    bytes_logged: u32,
    graph_data: FcuTelemetryGraphData,
    problems: Vec<DatasetEntry<'a>>,
}

static LATEST_FCU_TELEMETRY_STATE: Mutex<Option<FcuTelemetryData>> = Mutex::new(None);

struct FcuTelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    last_fcu_telem_frame: FcuTelemetryFrame,
    packet_queue: Vec<FcuTelemetryFrame>,
    data_refresh_time: f64,
    telemetry_rate_record_time: f64,
    last_telemetry_timestamp: f64,
    current_telemetry_rate_hz: u32,
}

impl FcuTelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            last_fcu_telem_frame: FcuTelemetryFrame::default(),
            packet_queue: vec![FcuTelemetryFrame::default(); 1000],
            data_refresh_time: 1.0 / 30.0,
            telemetry_rate_record_time: 1.0,
            last_telemetry_timestamp: timestamp(),
            current_telemetry_rate_hz: 0,
        }
    }

    pub fn run(&mut self) {
        let mut telemetry_counter = 0;
        let mut last_refresh_time = timestamp();
        let mut last_rate_record_time = timestamp();

        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                match packet {
                    Packet::FcuTelemetry(frame) => {
                        self.last_fcu_telem_frame = frame;
                        self.last_telemetry_timestamp = timestamp();
                        telemetry_counter += 1;
                    },
                    _ => {}
                }
            }

            let now = timestamp();
            if now - last_refresh_time >= self.data_refresh_time {
                last_refresh_time = now;

                self.packet_queue.drain(0..1);
                self.packet_queue.push(self.last_fcu_telem_frame.clone());

                self.update_telemetry_queue();
            }

            if now - last_rate_record_time >= self.telemetry_rate_record_time {
                last_rate_record_time = now;

                let telem_rate = (telemetry_counter as f64) / self.telemetry_rate_record_time;

                self.current_telemetry_rate_hz = telem_rate as u32;
                telemetry_counter = 0;
            }
        }
    }

    fn update_telemetry_queue(&self) {
        let mut telem = FcuTelemetryData::default();
        let last_frame = self.packet_queue.last().unwrap();

        telem.vehicle_state = format!("{:?}", last_frame.vehicle_state);
        telem.telemetry_rate = self.current_telemetry_rate_hz;
        telem.telemetry_delta_t = (timestamp() - self.last_telemetry_timestamp) as f32;
        telem.output_channels = Vec::from(last_frame.output_channels);
        telem.pwm_channels = Vec::from(last_frame.pwm_channels);
        telem.battery_voltage = last_frame.battery_voltage;
        telem.bytes_logged = last_frame.data_logged_bytes;
        telem.speed = (
            last_frame.velocity.x.powi(2) +
            last_frame.velocity.y.powi(2) +
            last_frame.velocity.z.powi(2)
        ).sqrt();
        telem.orientation = vec![
            last_frame.orientation.v.x,
            last_frame.orientation.v.y,
            last_frame.orientation.v.z,
            last_frame.orientation.s,
        ];
        telem.acceleration = vec![
            last_frame.acceleration.x,
            last_frame.acceleration.y,
            last_frame.acceleration.z,
        ];
        telem.angular_velocity = vec![
            last_frame.angular_velocity.x,
            last_frame.angular_velocity.y,
            last_frame.angular_velocity.z,
        ];

        if telem.telemetry_rate > 0 {
            telem.graph_data.altitude.push(last_frame.position.y);
            telem.graph_data.y_velocity.push(last_frame.velocity.y);
        }

        telem.problems = vec![
            DatasetEntry { name: "🟩 Squat", value: "No tracked problems" },
            DatasetEntry { name: "🟩 Igni", value: "No tracked problems" },
            DatasetEntry { name: "🟩 GSE", value: "No tracked problems" },
        ];

        LATEST_FCU_TELEMETRY_STATE
            .lock()
            .expect("Failed to lock telemetry state")
            .replace(telem);
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
pub fn fcu_telemetry_endpoint<'a>() -> Json<FcuTelemetryData<'a>> {
    let mut latest_telemetry = LATEST_FCU_TELEMETRY_STATE.lock().expect("Failed to lock telemetry state");

    if let Some(latest_telemetry) = latest_telemetry.as_mut() {
        let telem = latest_telemetry.clone();
        latest_telemetry.graph_data = FcuTelemetryGraphData::default();
        Json(telem.clone())
    } else {
        Json(FcuTelemetryData::default())
    }
}