use std::sync::Arc;

use hal::{
    comms_hal::{Packet, NetworkAddress},
};
use rocket::{
    serde::json::Json,
    State,
};

use crate::{observer::ObserverHandler, commands::CommandResponse};

use super::send_command;

#[post("/start-stream")]
pub fn start_stream(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::GroundCamera(0),
        Packet::StartCameraStream { port: 25570 },
    )
}

#[post("/stop-stream")]
pub fn stop_stream(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::GroundCamera(0),
        Packet::StopCameraStream,
    )
}