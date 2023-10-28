use std::sync::Arc;

use shared::comms_hal::{Packet, NetworkAddress};
use rocket::{
    serde::json::Json,
    State,
};

use crate::{observer::ObserverHandler, commands::CommandResponse};

use super::send_command;

#[post("/test-fire-igniter")]
pub fn test_fire_igniter(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::FireIgniter,
    )
}