mod connection;
mod webrtc;

use std::{sync::Arc, time::Duration, net::Ipv4Addr};

use hal::comms_hal::{Packet, NetworkAddress};
use rocket::{State, serde::json::Json};

use crate::{observer::{ObserverHandler, ObserverEvent, ObserverResponse}, process_is_running, timestamp, commands::CommandResponse};

use self::connection::CameraConnection;

pub const CAMERA_CONNECTION_TIMEOUT: f64 = 5.0;
pub const CAMERA_CONNECTION_PORT_START: u16 = 5000;
pub const TRANSCODE_PORT_START: u16 = 5500;

pub struct CameraStreaming {
    observer_handler: Arc<ObserverHandler>,
    active_connections: Vec<CameraConnection>,
    connection_port_counter: u16,
    transcode_port_counter: u16,
}

impl CameraStreaming {
    pub fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
            active_connections: Vec::new(),
            connection_port_counter: CAMERA_CONNECTION_PORT_START,
            transcode_port_counter: TRANSCODE_PORT_START,
        }
    }

    pub fn run(&mut self) {
        while process_is_running() {
            if let Some(event) = self.get_event() {
                match event {
                    ObserverEvent::PacketReceived { address: _, packet } => {
                        self.handle_packet(packet);
                    },
                    ObserverEvent::SetupBrowserStream { camera_address, browser_session } => {
                        for connection in &mut self.active_connections {
                            if connection.address == camera_address {
                                connection.setup_browser_stream(browser_session);
                                break;
                            }
                        }
                    },
                    _ => {}
                }
            }

            self.active_connections.retain_mut(|connection| {
                if timestamp() - connection.last_ping > CAMERA_CONNECTION_TIMEOUT {
                    print!("Dropping camera connection: {:?}...", connection.address);
                    connection.drop_connection();
                    println!(" done");
                    return false;
                }

                return true;
            });
        }

        for connection in &mut self.active_connections {
            print!("Dropping camera connection: {:?}...", connection.address);
            connection.drop_connection();
            println!(" done");
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

    fn handle_ping(&mut self, address: NetworkAddress, connection_ip: Ipv4Addr) {
        let mut found = false;
        for connection in &mut self.active_connections {
            if connection.address == address {
                found = true;
                connection.ping();
                break;
            }
        }

        if !found {
            let connection_port = self.connection_port_counter;
            let transcode_port = self.transcode_port_counter;
            match CameraConnection::new(
                address,
                connection_ip,
                connection_port,
                transcode_port,
                self.observer_handler.clone(),
            ) {
                Some(connection) => {
                    self.active_connections.push(connection);
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
    }

    fn get_event(&self) -> Option<ObserverEvent> {
        let timeout = Duration::from_millis(10);

        if let Some((_, event)) = self.observer_handler.wait_event(timeout) {
            return Some(event);
        }

        None
    }
}

#[get("/browser-stream", data = "<args>")]
pub fn browser_stream(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() != 2 {
        return Json(CommandResponse::new(
            String::from(format!("Failed to start browser stream, got wrong number of arguments")),
            false,
        ));
    }

    let camera_index = match args[0].parse::<u8>() {
        Ok(index) => index,
        Err(err) => {
            return Json(CommandResponse::new(
                String::from(format!("Failed to start browser stream, wrong args[0]: {:?}", err)),
                false,
            ));
        }
    };

    let browser_session = match args[1].parse::<String>() {
        Ok(session) => session,
        Err(err) => {
            return Json(CommandResponse::new(
                String::from(format!("Failed to start browser stream, wrong args[1]: {:?}", err)),
                false,
            ));
        }
    };

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