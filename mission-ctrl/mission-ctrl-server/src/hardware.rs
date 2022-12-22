use std::{net::UdpSocket, time::Duration, io::ErrorKind, sync::{Arc, mpsc::{Receiver, RecvTimeoutError, Sender}, atomic::Ordering}};
use hal::{comms_hal::Packet, ecu_hal::{ECUTelemetryFrame, ECU_SENSORS}, SensorConfig};
use rocket::serde::Deserialize;

use crate::{TelemetryQueue, recording::RecordingFrame, timestamp};

const BUFFER_SIZE: usize = 512;

pub fn hardware_thread(
    telem_queue: Arc<TelemetryQueue>, 
    packet_rx: Receiver<Packet>,
    recording_tx: Sender<RecordingFrame>
) {
    let socket = UdpSocket::bind("0.0.0.0:25565").unwrap();
    let mut buffer = [0_u8; BUFFER_SIZE];

    socket.set_read_timeout(Some(Duration::from_millis(1))).unwrap();

    for config_packet in read_hardware_config() {
        let size = config_packet.serialize(&mut buffer).unwrap();
        socket.send_to(&buffer[0..size], "169.254.0.6:25565").unwrap();
    }

    let mut last_telemetry_frame = ECUTelemetryFrame::default();
    let mut data_skip = 0_usize;

    let mut last_freq_time = timestamp();
    let mut num_telemetry_packets = 0_u32;
    let mut num_daq_packets = 0_u32;

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, _address)) => {
                let packet = Packet::deserialize(&mut buffer[0..size]);

                match packet {
                    Ok(packet) => {
                        match &packet {
                            Packet::ECUTelemetry(frame) => {
                                if data_skip % 3 == 0 {
                                    let mut queue = telem_queue.packets.lock().unwrap();
                                    queue.drain(0..1);
                                    queue.push(packet.clone());
                                }

                                data_skip += 1;
                                num_telemetry_packets += 1;
                                last_telemetry_frame = frame.clone();
                            },
                            Packet::ECUDAQ(daq_frames) => {
                                num_daq_packets += 10;

                                recording_tx.send(RecordingFrame {
                                    timestamp: timestamp(),
                                    telem: last_telemetry_frame.clone(),
                                    daq: (*daq_frames).clone(),
                                }).unwrap();
                            },
                            _ => {}
                        }
                    },
                    Err(err) => {
                        eprintln!("Failed to deserialize packet, got {:?}", err);
                    },
                }
            },
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut => {},
                _ => panic!("Failed to receive packet from UDP socket: {:?}", err),
            },
        }

        match packet_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(packet) => {
                let size = packet.serialize(&mut buffer).unwrap();

                socket.send_to(&buffer[0..size], "169.254.0.6:25565").unwrap();
            },
            Err(err) => match err {
                RecvTimeoutError::Timeout => {},
                _ => panic!("Failed to receive packet_send_rx: {:?}", err),
            },
        }

        if timestamp() - last_freq_time >= 1.0 {
            telem_queue.telem_rate.store(num_telemetry_packets, Ordering::Relaxed);
            telem_queue.daq_rate.store(num_daq_packets, Ordering::Relaxed);

            last_freq_time = timestamp();
            num_telemetry_packets = 0;
            num_daq_packets = 0;
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct HardwareConfigSensorMap {
    index: usize,
    premin: f32,
    premax: f32,
    postmin: f32,
    postmax: f32,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct HardwareConfig {
    sensor_mappings: Vec<HardwareConfigSensorMap>,
}

fn read_hardware_config() -> Vec<Packet> {
    let hardware_config = std::fs::read_to_string("hardware.json").expect("Couldn't load hardware config file!");
    let config: HardwareConfig = rocket::serde::json::from_str(&hardware_config).unwrap();
    let mut config_packets = Vec::new();

    for sensor_config in config.sensor_mappings.iter() {
        config_packets.push(Packet::ConfigureSensor { 
            sensor: ECU_SENSORS[sensor_config.index], 
            config: SensorConfig {
                premin: sensor_config.premin,
                premax: sensor_config.premax,
                postmin: sensor_config.postmin,
                postmax: sensor_config.postmax,
            },
        });
    }

    config_packets
}
