use std::sync::Arc;

use rocket::{serde::json::Json, State};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::EcuCommand,
};

use crate::{commands::CommandResponse, observer::ObserverHandler};

use super::send_command;

#[post("/test-fire-igniter")]
pub fn test_fire_igniter(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::EcuCommand(EcuCommand::FireIgniter),
    )
}
