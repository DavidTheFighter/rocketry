use std::{process::{Child, Command, Stdio}, net::{Ipv4Addr, UdpSocket}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use hal::comms_hal::{NetworkAddress, Packet};

use crate::{timestamp, observer::{ObserverHandler, ObserverEvent}};

use super::webrtc::WebRtcStream;


pub struct CameraConnection {
    pub address: NetworkAddress,
    pub last_ping: f64,
    connection_ip: Ipv4Addr,
    transcode_process: Child,
    observer_handler: Arc<ObserverHandler>,
    browser_streams: Vec<WebRtcStream>,
    alive: Arc<AtomicBool>,
}

impl CameraConnection {
    pub fn new(
        address: NetworkAddress,
        connection_ip: Ipv4Addr,
        connection_port: u16,
        transcode_port: u16,
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
            let alive_ref = alive.clone();
            std::thread::spawn(move || {
                Self::transcode_thread(alive_ref, transcode_port);
            });

            return Some(Self {
                address,
                last_ping: timestamp(),
                connection_ip,
                transcode_process,
                observer_handler,
                browser_streams: Vec::new(),
                alive,
            });
        }

        eprintln!("Failed to start transcoding process for camera: {:?}", address);

        None
    }

    pub fn setup_browser_stream(&mut self, browser_session: String) {
        
    }

    pub fn ping(&mut self) {
        self.last_ping = timestamp();
    }

    pub fn drop_connection(&mut self) {
        self.transcode_process.kill().expect("Failed to kill transcoding process");
        self.alive.store(false, Ordering::Relaxed);
        self.observer_handler.notify(ObserverEvent::SendPacket {
            address: self.address,
            packet: Packet::StopCameraStream,
        });
    }

    fn transcode_thread(alive: Arc<AtomicBool>, transcode_port: u16) {
        let mut buffer = vec![0; 1600];
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", transcode_port))
            .expect("Failed to bind to transcode socket");
        socket.set_read_timeout(Some(Duration::from_millis(100)))
            .expect("Failed to set transcode socket timeout");

        while alive.load(Ordering::Relaxed) {
            if let Ok((size, _)) = socket.recv_from(&mut buffer) {
                if size > 0 {
                    // TODO
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