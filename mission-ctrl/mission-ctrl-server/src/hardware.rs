use std::{net::UdpSocket, time::{Duration, SystemTime, UNIX_EPOCH}, io::ErrorKind, sync::{Arc, mpsc::{Receiver, RecvTimeoutError}, atomic::Ordering}};
use hal::comms_hal::Packet;

use crate::TelemetryQueue;

const BUFFER_SIZE: usize = 512;

pub fn hardware_thread(telem_queue: Arc<TelemetryQueue>, send_rx: Receiver<Packet>) {
    let socket = UdpSocket::bind("0.0.0.0:25565").unwrap();
    let mut buffer = [0_u8; BUFFER_SIZE];

    socket.set_read_timeout(Some(Duration::from_millis(1))).unwrap();

    let timestamp = || -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64()
    };

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
                        match packet {
                            Packet::ECUTelemetry(_) => {
                                if data_skip % 3 == 0 {
                                    let mut queue = telem_queue.packets.lock().unwrap();
                                    queue.drain(0..1);
                                    queue.push(packet);
                                }

                                data_skip += 1;
                                num_telemetry_packets += 1;
                            },
                            Packet::ECUDAQ(_) => {
                                num_daq_packets += 10;
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

        match send_rx.recv_timeout(Duration::from_millis(0)) {
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