use std::sync::{Arc, Mutex};
use std::time::Duration;

use hal::comms_hal::{Packet, DAQ_PACKET_FRAMES};
use hal::ecu_hal::{EcuTelemetryFrame, EcuSolenoidValve, EcuSensor};
use rocket::serde::{json::Json, Serialize};

use crate::observer::{ObserverHandler, ObserverEvent};
use crate::{timestamp, process_is_running};

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

static LATEST_TELEMETRY_STATE: Mutex<Option<TelemetryData>> = Mutex::new(None);

struct TelemetryHandler {
    observer_handler: Arc<ObserverHandler>,
    last_ecu_telem_frame: EcuTelemetryFrame,
    packet_queue: Vec<EcuTelemetryFrame>,
    data_refresh_time: f64,
    telemetry_rate_record_time: f64,
    current_telemetry_rate_hz: u32,
    current_daq_rate_hz: u32,
}

impl TelemetryHandler {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            last_ecu_telem_frame: EcuTelemetryFrame::default(),
            packet_queue: vec![EcuTelemetryFrame::default(); 333],
            data_refresh_time: 0.0333333334,
            telemetry_rate_record_time: 0.25,
            current_telemetry_rate_hz: 0,
            current_daq_rate_hz: 0,
        }
    }

    pub fn run(&mut self) {
        let mut telemetry_counter = 0;
        let mut daq_counter = 0;
        let mut last_refresh_time = timestamp();
        let mut last_rate_record_time = timestamp();

        while process_is_running() {
            if let Some(packet) = self.get_packet() {
                match packet {
                    Packet::EcuTelemetry(frame) => {
                        self.last_ecu_telem_frame = frame;
                        telemetry_counter += 1;
                    },
                    Packet::EcuDAQ(_) => {
                        daq_counter += DAQ_PACKET_FRAMES;
                    },
                    _ => {}
                }
            }

            let now = timestamp();
            if now - last_refresh_time >= self.data_refresh_time {
                last_refresh_time = now;

                self.packet_queue.drain(0..1);
                self.packet_queue.push(self.last_ecu_telem_frame.clone());
                self.last_ecu_telem_frame = EcuTelemetryFrame::default();

                self.update_telemetry_queue();
            }

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

    fn update_telemetry_queue(&self) {
        let mut telem = TelemetryData::default();
        let last_frame = self.packet_queue.last().unwrap();

        telem.igniter_state = format!("{:?}", last_frame.igniter_state);
        telem.tank_state = format!("{:?}", last_frame.fuel_tank_state);
        telem.cpu_utilization = last_frame.cpu_utilization;
        telem.telemetry_rate = self.current_telemetry_rate_hz;
        telem.daq_rate = self.current_daq_rate_hz;

        let fmt_valve_state = |valve: bool, flipped: bool| -> HardwareState {
            HardwareState {
                state: String::from(if valve { "Open" } else { "Closed" }),
                in_default_state: if flipped { valve } else { !valve },
            }
        };

        telem.igniter_fuel_valve = fmt_valve_state(
            last_frame.solenoid_valves[EcuSolenoidValve::IgniterFuelMain as usize],
            false,
        );
        telem.igniter_gox_valve = fmt_valve_state(
            last_frame.solenoid_valves[EcuSolenoidValve::IgniterGOxMain as usize],
            false,
        );
        telem.fuel_press_valve = fmt_valve_state(
            last_frame.solenoid_valves[EcuSolenoidValve::FuelPress as usize],
            false,
        );
        telem.fuel_vent_valve = fmt_valve_state(
            last_frame.solenoid_valves[EcuSolenoidValve::FuelVent as usize],
            true,
        );
        telem.sparking = HardwareState {
            state: String::from(if last_frame.sparking { "On" } else { "Off" }),
            in_default_state: !last_frame.sparking,
        };

        for frame in self.packet_queue.iter() {
            telem
                .igniter_fuel_pressure
                .push(frame.sensors[EcuSensor::IgniterFuelInjectorPressure as usize]);
            telem
                .igniter_gox_pressure
                .push(frame.sensors[EcuSensor::IgniterGOxInjectorPressure as usize]);
            telem
                .igniter_chamber_pressure
                .push(frame.sensors[EcuSensor::IgniterChamberPressure as usize]);
            telem
                .fuel_tank_pressure
                .push(frame.sensors[EcuSensor::FuelTankPressure as usize]);
            telem
                .ecu_board_temp
                .push(frame.sensors[EcuSensor::ECUBoardTemp as usize]);
            telem
                .igniter_throat_temp
                .push(frame.sensors[EcuSensor::IgniterThroatTemp as usize]);
        }

        LATEST_TELEMETRY_STATE
            .lock()
            .expect("Failed to lock telemetry state")
            .replace(telem);
    }

    fn get_packet(&self) -> Option<Packet> {
        let timeout = Duration::from_millis(1);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            if let ObserverEvent::PacketReceived { address: _, packet } = event {
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
pub fn ecu_telemetry_endpoint() -> Json<TelemetryData> {
    let latest_telemetry = LATEST_TELEMETRY_STATE.lock().expect("Failed to lock telemetry state");

    if let Some(latest_telemetry) = latest_telemetry.as_ref() {
        Json(latest_telemetry.clone())
    } else {
        Json(TelemetryData::default())
    }
}