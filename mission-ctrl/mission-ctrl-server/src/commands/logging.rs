use std::sync::Arc;

use rocket::{serde::json::Json, State};
use shared::comms_hal::{NetworkAddress, Packet};

use crate::{commands::CommandResponse, observer::ObserverHandler};

use super::{format_response, send_command};

#[post("/erase-flash")]
pub fn erase_flash(_observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    return format_response(format!("You know this command doesn't work"), false);
}

#[post("/set-logging", data = "<args>")]
pub fn set_logging(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() != 2 {
        return format_response(format!("{} <true|false>", args[0]), false);
    }

    let state = match args[1].as_str() {
        "true" => true,
        "false" => false,
        _ => {
            return format_response(
                format!("'{}' is not a valid state!", args[1].as_str()),
                false,
            );
        }
    };

    send_command(
        observer_handler,
        NetworkAddress::FlightController,
        Packet::EnableDataLogging(state),
    )
}

#[post("/retrieve-logs")]
pub fn retrieve_logs(_observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    return format_response(format!("You know this command doesn't work"), false);

    // let mut data = Vec::new();
    // let mut addr = 0;

    // observer_handler.register_observer_thread();

    // loop {
    //     let response = send_command(
    //         observer_handler,
    //         NetworkAddress::FlightController,
    //         Packet::RetrieveDataLogPage(addr),
    //     );

    //     if !response.success {
    //         return response;
    //     }

    //     println!("Retrieving data log page at address {}", addr);

    //     let timeout = std::time::Duration::from_millis(1000);
    //     let start_time = std::time::Instant::now();
    //     let mut data_log_buffer: Option<DataLogBuffer> = None;
    //     while std::time::Instant::now().duration_since(start_time).as_secs_f32() < timeout.as_secs_f32() {
    //         if let Some((_id, event)) = observer_handler.wait_event(timeout) {
    //             if let ObserverEvent::PacketReceived { address: _, packet } = event {
    //                 if let Packet::FcuDataLogPage(data_log_page) = packet {
    //                     data_log_buffer = Some(data_log_page);
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     match data_log_buffer {
    //         Some(data_log_buffer) => {
    //             // println!("{:?}", data_log_buffer.buffer);

    //             if data_log_buffer.buffer.iter().all(|&b| b == 0xFF) {
    //                 return format_response(
    //                     format!("Successfully retrieved {} KiB of log data", data.len() / 1024),
    //                     true,
    //                 );
    //             }

    //             data.extend_from_slice(&data_log_buffer.buffer);
    //         }
    //         None => {
    //             return format_response(
    //                 format!("Failed to retrieve data log page at address {}", addr),
    //                 false,
    //             );
    //         }
    //     }

    //     addr += 256;

    //     if addr >= 256 * 256 {
    //         return format_response(
    //             format!("Read entire flash chip"),
    //             false,
    //         );
    //     }
    // }
}
