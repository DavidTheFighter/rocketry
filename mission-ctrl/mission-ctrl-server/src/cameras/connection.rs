use std::{process::{Child, Command, Stdio}, net::{Ipv4Addr, UdpSocket}, sync::{Arc, atomic::{AtomicBool, Ordering, AtomicU64}, RwLock}, time::Duration};

use shared::comms_hal::{NetworkAddress, Packet};

use crate::{timestamp, observer::{ObserverHandler, ObserverEvent}};

use super::{webrtc::WebRtcStream, CAMERA_CONNECTION_TIMEOUT, CAMERA_STARTUP_TIMEOUT_GRACE};


pub struct CameraConnection {
    pub address: NetworkAddress,
    last_ping: Arc<AtomicU64>,
    transcode_process: Child,
    observer_handler: Arc<ObserverHandler>,
    alive: Arc<AtomicBool>,
}

impl CameraConnection {
    pub fn new(
        address: NetworkAddress,
        connection_ip: Ipv4Addr,
        connection_port: u16,
        transcode_port: u16,
        browser_streams: Arc<RwLock<Vec<WebRtcStream>>>,
        observer_handler: Arc<ObserverHandler>,
    ) -> Option<CameraConnection> {
        if let Some(transcode_process) = Self::start_transcode_process(
            connection_ip,
            connection_port,
            transcode_port,
        ) {
            observer_handler.notify(ObserverEvent::SendPacket {
                address,
                packet: Packet::StartCameraStream { port: connection_port },
            });

            let alive = Arc::new(AtomicBool::new(true));

            let start_timestamp = timestamp_u64() + (CAMERA_STARTUP_TIMEOUT_GRACE * 1e3) as u64;
            let last_ping: Arc<AtomicU64> = Arc::new(AtomicU64::new(start_timestamp));

            let alive_ref = alive.clone();
            let last_ping_ref = last_ping.clone();
            std::thread::spawn(move || {
                Self::transcode_thread(
                    alive_ref,
                    last_ping_ref,
                    address,
                    transcode_port,
                    browser_streams
                );
            });

            return Some(Self {
                address,
                last_ping,
                transcode_process,
                observer_handler,
                alive,
            });
        }

        eprintln!("Failed to start transcoding process for camera: {:?}", address);

        None
    }

    pub fn timed_out(&self) -> bool {
        let last_ping = (self.last_ping.load(Ordering::Relaxed) as f64) * 1e-3;

        timestamp() - last_ping > CAMERA_CONNECTION_TIMEOUT
    }

    pub fn drop_connection(&mut self) {
        self.transcode_process.kill().expect("Failed to kill transcoding process");
        self.alive.store(false, Ordering::Relaxed);
        self.observer_handler.notify(ObserverEvent::SendPacket {
            address: self.address,
            packet: Packet::StopCameraStream,
        });
    }

    fn transcode_thread(
        alive: Arc<AtomicBool>,
        last_ping: Arc<AtomicU64>,
        camera_address: NetworkAddress,
        transcode_port: u16,
        browser_streams: Arc<RwLock<Vec<WebRtcStream>>>,
    ) {
        let mut buffer = vec![0; 1600];
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", transcode_port))
            .expect("Failed to bind to transcode socket");
        socket.set_read_timeout(Some(Duration::from_millis(100)))
            .expect("Failed to set transcode socket timeout");

        while alive.load(Ordering::Relaxed) {
            if let Ok((size, _)) = socket.recv_from(&mut buffer) {
                if size > 0 {
                    last_ping.store(timestamp_u64(), Ordering::Relaxed);

                    let streams = browser_streams
                        .read()
                        .expect("Failed to unlock browser streams vector");

                    for stream in streams.iter() {
                        if stream.camera_address == camera_address {
                            stream.send_data(Vec::from(&buffer[..size]));
                        }
                    }
                }
            }
        }
    }

    fn start_transcode_process(connection_ip: Ipv4Addr, connection_port: u16, transcode_port: u16) -> Option<Child> {
        Command::new("ffmpeg")
            .args(["-i", format!("udp://{}:{}", connection_ip, connection_port).as_str()])
            .args(["-c:v", "copy"])
            .args(["-cpu-used", "5"])
            .args(["-deadline", "1"])
            .args(["-g", "10"])
            .args(["-error-resilient", "1"])
            .args(["-auto-alt-ref", "1"])
            .args(["-f", "rtp"])
            .arg(format!("rtp://127.0.0.1:{}?pkt_size=1200", transcode_port))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .ok()
    }
}

fn timestamp_u64() -> u64 {
    (timestamp() * 1e3) as u64
}