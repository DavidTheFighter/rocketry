use std::{io::Write, sync::Arc, time::Duration};

use shared::comms_hal::Packet;

use crate::{
    observer::{ObserverEvent, ObserverHandler},
    process_is_running,
};

const ENABLED: bool = false;

struct LoggingThread {
    observer_handler: Arc<ObserverHandler>,
    sensor_data_file: std::fs::File,
    bytes_logged: usize,
    time_since_last_print: std::time::Instant,
    start_time: std::time::SystemTime,
}

impl LoggingThread {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        let filename = format!(
            "{}-sensor-data.txt",
            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S")
        );
        Self {
            observer_handler,
            sensor_data_file: std::fs::File::create(filename).unwrap(),
            bytes_logged: 0,
            time_since_last_print: std::time::Instant::now(),
            start_time: std::time::SystemTime::now(),
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            loop {
                if let Some(packet) = self.get_packet() {
                    self.handle_packet(packet);
                } else {
                    break;
                }
            }

            if self.time_since_last_print.elapsed().as_secs() >= 1 {
                println!(
                    "logging_thread: Logged {} bytes ({} MiB)",
                    self.bytes_logged,
                    (self.bytes_logged as f32) / 1024.0 / 1024.0
                );
                self.time_since_last_print = std::time::Instant::now();
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        if let Packet::FcuDebugSensorMeasurement(sensor) = &packet {
            let log_str = format!("{}: {:?}\n", self.timestamp(), sensor);
            let log_bytes = log_str.as_bytes();
            self.sensor_data_file.write_all(log_bytes).unwrap();
            self.bytes_logged += log_bytes.len();
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

    fn timestamp(&self) -> u128 {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(self.start_time).unwrap();
        duration.as_micros()
    }
}

pub fn logging_thread(observer_handler: Arc<ObserverHandler>) {
    if ENABLED {
        observer_handler.register_observer_thread();
        let mut logging_thread = LoggingThread::new(observer_handler);
        logging_thread.run();
    }
}
