use std::sync::Arc;

use shared::{
    comms_hal::{Packet, NetworkAddress}, ecu_hal::FuelTankState,
};
use rocket::{
    serde::json::Json,
    State,
};

use crate::{observer::ObserverHandler, commands::CommandResponse};

use super::send_command;

#[post("/fuel-press")]
pub fn fuel_pressurize(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::TransitionFuelTankState(FuelTankState::Pressurized),
    )
}

#[post("/fuel-idle")]
pub fn fuel_idle(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::TransitionFuelTankState(FuelTankState::Idle),
    )
}

#[post("/fuel-depress")]
pub fn fuel_depressurize(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::TransitionFuelTankState(FuelTankState::Depressurized),
    )
}
