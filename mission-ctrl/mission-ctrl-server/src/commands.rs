pub mod components;
pub mod logging;
pub mod sequence;
pub mod streamish;
pub mod tanks;

use std::{sync::Arc, time::Duration};

use rocket::{
    serde::{json::Json, Serialize},
    State,
};
use shared::comms_hal::{NetworkAddress, Packet};

use crate::observer::{ObserverEvent, ObserverHandler};
use components::{set_solenoid_valve, test_solenoid_valve, test_spark};
use logging::{erase_flash, retrieve_logs, set_logging};
use sequence::test_fire_igniter;
use streamish::{start_stream, stop_stream};
use tanks::{fuel_depressurize, fuel_idle, fuel_pressurize};

pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        // Components
        set_solenoid_valve,
        test_solenoid_valve,
        test_spark,
        // Sequence
        test_fire_igniter,
        // Tanks
        fuel_pressurize,
        fuel_idle,
        fuel_depressurize,
        // Logging
        erase_flash,
        set_logging,
        retrieve_logs,
        // Cameras
        start_stream,
        stop_stream,
    ]
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CommandResponse {
    text_response: String,
    success: bool,
}

fn send_command(
    observer_handler: &State<Arc<ObserverHandler>>,
    address: NetworkAddress,
    packet: Packet,
) -> Json<CommandResponse> {
    observer_handler.register_observer_thread();
    let event_id = observer_handler.notify(ObserverEvent::SendPacket {
        address,
        packet: packet.clone(),
    });
    let timeout = Duration::from_millis(1000);
    let response = observer_handler.get_response(event_id, timeout);

    match response {
        Some(result) => match result {
            Ok(_) => Json(CommandResponse {
                text_response: String::from(format!(
                    "Sent '${:?}' command to {:?}",
                    packet, address
                )),
                success: true,
            }),
            Err(err) => Json(CommandResponse {
                text_response: String::from(format!(
                    "Failed to send '${:?}' command, got {:?}",
                    packet, err
                )),
                success: false,
            }),
        },
        None => Json(CommandResponse {
            text_response: String::from(format!(
                "Failed to send '${:?}' command, got timeout",
                packet
            )),
            success: false,
        }),
    }
}

fn format_response(text_response: String, success: bool) -> Json<CommandResponse> {
    Json(CommandResponse {
        text_response,
        success,
    })
}

impl CommandResponse {
    pub fn new(text_response: String, success: bool) -> Self {
        Self {
            text_response,
            success,
        }
    }
}
