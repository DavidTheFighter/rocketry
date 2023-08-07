mod connection;
mod webrtc;

use std::{sync::{Arc, RwLock}, time::Duration, net::Ipv4Addr};

use hal::comms_hal::{Packet, NetworkAddress};
use rocket::{State, serde::{json::Json, Serialize, Deserialize}};

use crate::{observer::{ObserverHandler, ObserverEvent, ObserverResponse}, process_is_running, commands::CommandResponse};

use self::{connection::CameraConnection, webrtc::WebRtcStream};

pub const CAMERA_CONNECTION_TIMEOUT: f64 = 5.0;
pub const CAMERA_STARTUP_TIMEOUT_GRACE: f64 = 10.0;
pub const CAMERA_CONNECTION_PORT_START: u16 = 5000;
pub const TRANSCODE_PORT_START: u16 = 5500;

pub struct CameraStreaming {
    observer_handler: Arc<ObserverHandler>,
    camera_connections: Vec<CameraConnection>,
    browser_streams: Arc<RwLock<Vec<WebRtcStream>>>,
    connection_port_counter: u16,
    transcode_port_counter: u16,
}

impl CameraStreaming {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            camera_connections: Vec::new(),
            browser_streams: Arc::new(RwLock::new(Vec::new())),
            connection_port_counter: CAMERA_CONNECTION_PORT_START,
            transcode_port_counter: TRANSCODE_PORT_START,
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            if let Some((event_id, event)) = self.get_event() {
                match event {
                    ObserverEvent::PacketReceived { address: _, packet } => {
                        self.handle_packet(packet);
                    },
                    ObserverEvent::SetupBrowserStream { camera_address, browser_session } => {
                        self.setup_browser_stream(event_id, camera_address, browser_session);
                    },
                    _ => {}
                }
            }

            // Drop any camera streams that have timed out
            self.camera_connections.retain_mut(|connection| {
                if connection.timed_out() {
                    print!("Dropping camera connection: {:?}...", connection.address);
                    connection.drop_connection();
                    println!(" done");
                    return false;
                }

                return true;
            });

            // Drop any browser streams that have timed out
            self.browser_streams.write().expect("browser_streams write lock").retain_mut(|stream| {
                if stream.stream_closed() {
                    println!("Dropping browser stream: {:?}... done", stream.name());
                    return false;
                }

                return true;
            });
        }

        for connection in &mut self.camera_connections {
            print!("Dropping camera connection: {:?}...", connection.address);
            connection.drop_connection();
            println!(" done");
        }

        for stream in &mut *self.browser_streams.write().expect("browser_streams write lock") {
            println!("Dropping browser stream: {:?}... done", stream.name());
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet {
            Packet::ComponentIpAddress { addr, ip } => {
                self.handle_ping(addr, Ipv4Addr::from(ip));
            },
            _ => {}
        }
    }

    fn setup_browser_stream(
        &mut self,
        event_id: u64,
        camera_address: NetworkAddress,
        browser_session: String,
    ) {
        match WebRtcStream::new(camera_address, browser_session) {
            Ok(stream) => {
                let session_desc = stream.get_session_desc();
                self.browser_streams.write().expect("browser_streams write lock").push(stream);

                self.observer_handler.notify(ObserverEvent::EventResponse(
                    event_id,
                    Ok(ObserverResponse::BrowserStream {
                        stream_session: session_desc,
                    }),
                ));
            },
            Err(err) => {
                self.observer_handler.notify(ObserverEvent::EventResponse(
                    event_id,
                    Err(format!("Failed to setup browser stream: {:?}", err)),
                ));
            }
        }
    }

    fn handle_ping(&mut self, address: NetworkAddress, connection_ip: Ipv4Addr) {
        let mut found = false;
        for connection in &mut self.camera_connections {
            if connection.address == address {
                found = true;
                break;
            }
        }

        if !found {
            self.setup_camera_connection(address, connection_ip);
        }
    }

    fn setup_camera_connection(&mut self, address: NetworkAddress, connection_ip: Ipv4Addr) {
        let connection_port = self.connection_port_counter;
        let transcode_port = self.transcode_port_counter;

        let camera_connection = CameraConnection::new(
            address,
            connection_ip,
            connection_port,
            transcode_port,
            self.browser_streams.clone(),
            self.observer_handler.clone(),
        );

        match camera_connection {
            Some(connection) => {
                self.camera_connections.push(connection);
            },
            None => {
                println!("Failed to start transcoding process for camera: {:?}", address);
                return;
            }
        }

        println!("New camera connection: {:?} @ {:?}:{}, transcoding on {}",
            address,
            connection_ip,
            connection_port,
            transcode_port,
        );

        self.connection_port_counter += 1;
        self.transcode_port_counter += 1;
    }

    fn get_event(&self) -> Option<(u64, ObserverEvent)> {
        let timeout = Duration::from_millis(10);

        if let Some((id, event)) = self.observer_handler.wait_event(timeout) {
            return Some((id, event));
        }

        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct BrowserStreamArgs {
    camera_index: u8,
    browser_session: String,
}

#[post("/browser-stream", data = "<args>")]
pub fn browser_stream(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<BrowserStreamArgs>,
) -> Json<CommandResponse> {
    let camera_index = args.camera_index;
    let browser_session = args.browser_session.clone();

    observer_handler.register_observer_thread();
    let event_id = observer_handler.notify(ObserverEvent::SetupBrowserStream {
        camera_address: NetworkAddress::GroundCamera(camera_index),
        browser_session,
    });
    let timeout = Duration::from_millis(1000);
    let response = observer_handler.get_response(event_id, timeout);

    match response {
        Some(result) => {
            match result {
                Ok(response) => {
                    if let ObserverResponse::BrowserStream { stream_session } = response {
                        Json(CommandResponse::new(
                            stream_session,
                            true,
                        ))
                    } else {
                        Json(CommandResponse::new(
                            String::from(format!("Failed to start browser stream, got wrong response")),
                            false,
                        ))
                    }
                },
                Err(err) => Json(CommandResponse::new(
                    String::from(format!("Failed to start browser stream, got {:?}", err)),
                    false,
                )),
            }
        },
        None => Json(CommandResponse::new(
            String::from(format!("Failed to start browser stream, got timeout")),
            false,
        )),
    }
}

pub fn camera_streaming_thread(observer_handler: Arc<ObserverHandler>) {
    observer_handler.register_observer_thread();

    CameraStreaming::new(observer_handler).run();
}