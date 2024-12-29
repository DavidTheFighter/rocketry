pub mod components;
pub mod logging;
pub mod sequence;
pub mod streamish;
pub mod tanks;

use std::{sync::Arc, time::Duration};

use rocket::{
    futures::{stream::FusedStream, SinkExt, StreamExt},
    serde::{
        json::{serde_json, Json},
        Deserialize, Serialize,
    },
    State,
};
use shared::{
    comms_hal::{NetworkAddress, Packet, PacketWithAddress},
    ecu_hal::EcuResponse,
};

use crate::{
    observer::{ObserverEvent, ObserverHandler},
    process_is_running,
};
use components::{set_fcu_output, set_solenoid_valve, test_solenoid_valve, test_spark};
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
        set_fcu_output,
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

#[post("/packet", data = "<args>")]
pub fn send_packet(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<PacketWithAddress>,
) -> Json<CommandResponse> {
    observer_handler.register_observer_thread();
    observer_handler.notify(ObserverEvent::SendPacket {
        address: args.address,
        packet: args.packet.clone(),
    });

    format_response(
        format!("Sent packet {:?} to {:?}", args.packet, args.address),
        true,
    )
}

#[get("/packet-proxy")]
pub fn packet_proxy(
    ws: ws::WebSocket,
    observer_handler: &State<Arc<ObserverHandler>>,
) -> ws::Channel<'static> {
    let observer_handler = observer_handler.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            while process_is_running() && !stream.is_terminated() {
                tokio::select! {
                    message = stream.next() => {
                        if let Some(Ok(ws::Message::Text(text))) = message {
                            let packet: PacketWithAddress = serde_json::from_str(&text).unwrap();
                            observer_handler.register_observer_thread();
                            observer_handler.notify(ObserverEvent::SendPacket {
                                address: packet.address,
                                packet: packet.packet,
                            });
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        if observer_handler.register_observer_thread() {
                            // Only send config packets, sending ALL packets is so slow
                            // we build a backlog of packets
                            observer_handler.register_subscription_filter("pack-proxy-rx", |event| {
                                if let ObserverEvent::PacketReceived { address: _, ip: _, packet } = event {
                                    matches!(packet, Packet::EcuResponse(EcuResponse::Config(_)))
                                } else {
                                    false
                                }
                            });
                        }

                        while let Some((_id, event)) = observer_handler.get_event() {
                            if let ObserverEvent::PacketReceived { address, ip: _, packet } = event {
                                let packet_with_address = PacketWithAddress { address, packet };
                                let json_str = serde_json::to_string(&packet_with_address).unwrap();
                                stream.send(ws::Message::text(json_str)).await?;
                            }
                        }
                    },
                }
            }

            Ok(())
        })
    })
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
