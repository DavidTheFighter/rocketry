use std::sync::Arc;

use rocket::{serde::json::Json, State};
use shared::comms_hal::{NetworkAddress, Packet};

use crate::{commands::CommandResponse, observer::ObserverHandler};

use super::send_command;

#[post("/fuel-press")]
pub fn fuel_pressurize(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::DoNothing,
    )
}

#[post("/fuel-idle")]
pub fn fuel_idle(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::DoNothing,
    )
}

#[post("/fuel-depress")]
pub fn fuel_depressurize(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::DoNothing,
    )
}
