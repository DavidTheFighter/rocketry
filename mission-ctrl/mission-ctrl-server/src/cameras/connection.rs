use std::{process::{Child, Command, Stdio}, net::Ipv4Addr, sync::Arc};

use hal::comms_hal::{NetworkAddress, Packet};

use crate::{timestamp, observer::{ObserverHandler, ObserverEvent}};


pub struct CameraConnection {
    pub address: NetworkAddress,
    pub last_ping: f64,
    connection_ip: Ipv4Addr,
    transcode_process: Child,
    observer_handler: Arc<ObserverHandler>,
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

            return Some(Self {
                address,
                last_ping: timestamp(),
                connection_ip,
                transcode_process,
                observer_handler,
            });
        }

        eprintln!("Failed to start transcoding process for camera: {:?}", address);

        None
    }

    pub fn ping(&mut self) {
        self.last_ping = timestamp();
    }

    pub fn drop_connection(&mut self) {
        self.transcode_process.kill().expect("Failed to kill transcoding process");
        self.observer_handler.notify(ObserverEvent::SendPacket {
            address: self.address,
            packet: Packet::StopCameraStream,
        });
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