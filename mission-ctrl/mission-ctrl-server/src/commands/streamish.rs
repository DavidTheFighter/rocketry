use std::sync::Arc;

use rocket::{serde::json::Json, State};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    streamish_hal::StreamishCommand,
};

use crate::{commands::CommandResponse, observer::ObserverHandler};

use super::send_command;

#[post("/start-stream")]
pub fn start_stream(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::Camera(0),
        Packet::StreamishCommand(StreamishCommand::StartCameraStream { port: 25570 }),
    )
}

#[post("/stop-stream")]
pub fn stop_stream(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::Camera(0),
        Packet::StreamishCommand(StreamishCommand::StopCameraStream),
    )
}
