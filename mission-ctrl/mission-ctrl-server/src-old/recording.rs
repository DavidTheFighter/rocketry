use chrono::{Datelike, FixedOffset, TimeZone, Timelike, Utc};
use hal::ecu_hal::{ECUSensor, ECUSolenoidValve};
use hal::{
    comms_hal::DAQ_PACKET_FRAMES,
    ecu_hal::{ECUDAQFrame, ECUTelemetryFrame, IgniterState},
};
use std::io::prelude::*;
use std::{fs::File, sync::mpsc::Receiver};
use strum::IntoEnumIterator;

use crate::timestamp;

pub const RECORD_PADDING_SECONDS: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct RecordingFrame {
    pub timestamp: f64,
    pub telem: ECUTelemetryFrame,
    pub daq: [ECUDAQFrame; DAQ_PACKET_FRAMES],
}

pub fn recording_thread(recording_rx: Receiver<RecordingFrame>) {
    let mut current_firing_file = None;
    let mut end_timestamp = None;

    let mut frame_buffer: Vec<RecordingFrame> = Vec::new();

    loop {
        let frame = recording_rx.recv().unwrap();

        if frame.telem.igniter_state != IgniterState::Idle && current_firing_file.is_none() {
            let mut file = File::create(format!("firing_{}.csv", now_str())).unwrap();
            file.write_all(csv_columns().as_bytes()).unwrap();

            for frame in &frame_buffer {
                file.write_all(format_frame(frame.clone()).as_bytes())
                    .unwrap();
            }

            current_firing_file = Some(file);
        } else if frame.telem.igniter_state == IgniterState::Idle
            && current_firing_file.is_some()
            && end_timestamp.is_none()
        {
            end_timestamp = Some(timestamp());
        }

        if let Some(file) = &mut current_firing_file {
            file.write_all(format_frame(frame).as_bytes()).unwrap();
        } else {
            frame_buffer.push(frame);
        }

        frame_buffer.retain(|value| timestamp() - value.timestamp <= RECORD_PADDING_SECONDS);

        if let Some(end_timestamp_value) = end_timestamp {
            if timestamp() - end_timestamp_value >= RECORD_PADDING_SECONDS {
                current_firing_file = None;
                end_timestamp = None;
            }
        }
    }
}

fn format_frame(frame: RecordingFrame) -> String {
    let mut builder = String::new();

    for daq_frame in &frame.daq {
        builder = [
            builder,
            format!(
                "{:?},{:?},{}",
                frame.telem.igniter_state, frame.telem.fuel_tank_state, frame.telem.sparking,
            ),
        ]
        .concat();

        for sensor in ECUSensor::iter() {
            builder = [
                builder,
                format!(",{}", daq_frame.sensor_values[sensor as usize]),
            ]
            .concat();
        }

        for sv in ECUSolenoidValve::iter() {
            builder = [
                builder,
                format!(",{}", frame.telem.solenoid_valves[sv as usize]),
            ]
            .concat();
        }

        builder = [builder, String::from("\n")].concat();
    }

    builder
}

fn csv_columns() -> String {
    let mut builder = String::from("igniter_state,fuel_tank_state,spark");

    for sensor in ECUSensor::iter() {
        builder = [builder, format!(",{:?}", sensor)].concat();
    }

    for sv in ECUSolenoidValve::iter() {
        builder = [builder, format!(",{:?}", sv)].concat();
    }

    [builder, String::from("\n")].concat()
}

fn now_str() -> String {
    let tz_offset = FixedOffset::west_opt(5 * 60 * 60).unwrap();
    let now = tz_offset.from_utc_datetime(&Utc::now().naive_utc());

    format!(
        "{:02}_{:02}_{:02}-{:02}_{:02}_{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    )
}
