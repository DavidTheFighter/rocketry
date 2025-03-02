use std::{sync::Arc, thread, time::Duration};

use rocket::{serde::json::Json, State};
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuBinaryOutput, EcuCommand},
    fcu_hal::{self, VehicleCommand},
};

use crate::{
    commands::CommandResponse,
    observer::{ObserverEvent, ObserverHandler},
};

use super::{format_response, send_command};

fn match_valve(valve: &str) -> Option<EcuBinaryOutput> {
    match valve {
        "ig_fuel" => Some(EcuBinaryOutput::IgniterFuelValve),
        "ig_gox" => Some(EcuBinaryOutput::IgniterOxidizerValve),
        "press" => Some(EcuBinaryOutput::FuelPressValve),
        "vent" => Some(EcuBinaryOutput::FuelVentValve),
        _ => None,
    }
}

fn match_state(state: &str) -> Option<bool> {
    match state {
        "1" | "on" | "open" | "true" => Some(true),
        "0" | "off" | "closed" | "false" => Some(false),
        _ => None,
    }
}

fn valve_name_list() -> String {
    String::from("Valves: [ig_fuel, ig_gox, press, vent]")
}

fn valve_state_list() -> String {
    String::from("States: [0, closed, off, 1, open, on]")
}

#[post("/sv-valve", data = "<args>")]
pub fn set_solenoid_valve(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() != 3 {
        return format_response(
            format!(
                "{} <name> <state>\n{}\n{}",
                args[0],
                valve_name_list(),
                valve_state_list()
            ),
            false,
        );
    }

    let valve = match match_valve(args[1].as_str()) {
        Some(valve) => valve,
        None => {
            return format_response(
                format!("'{}' is not a valid valve name!", args[1].as_str()),
                false,
            );
        }
    };

    let state = match match_state(args[2].as_str()) {
        Some(state) => state,
        None => {
            return format_response(
                format!("'{}' is not a valid valve state!", args[2].as_str()),
                false,
            );
        }
    };

    send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::EcuCommand(EcuCommand::SetBinaryValve { valve, state }),
    )
}

#[post("/test-sv-valve", data = "<args>")]
pub fn test_solenoid_valve(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() != 2 {
        return format_response(format!("{} <name>\n{}", args[0], valve_name_list()), false);
    }

    let valve = match match_valve(args[1].as_str()) {
        Some(valve) => valve,
        None => {
            return format_response(
                format!("'{}' is not a valid valve name!", args[1].as_str()),
                false,
            );
        }
    };

    let return_value = send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::EcuCommand(EcuCommand::SetBinaryValve { valve, state: true }),
    );

    if return_value.success {
        let observer_handler_clone = observer_handler.inner().clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            observer_handler_clone.notify(ObserverEvent::SendPacket {
                address: NetworkAddress::EngineController(0),
                packet: Packet::EcuCommand(EcuCommand::SetBinaryValve {
                    valve,
                    state: false,
                }),
            });
        });
    }

    return_value
}

#[post("/test-spark")]
pub fn test_spark(observer_handler: &State<Arc<ObserverHandler>>) -> Json<CommandResponse> {
    let return_value = send_command(
        observer_handler,
        NetworkAddress::EngineController(0),
        Packet::EcuCommand(EcuCommand::SetSparking(true)),
    );

    if return_value.success {
        let observer_handler_clone = observer_handler.inner().clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            observer_handler_clone.notify(ObserverEvent::SendPacket {
                address: NetworkAddress::EngineController(0),
                packet: Packet::EcuCommand(EcuCommand::SetSparking(false)),
            });
        });
    }

    return_value
}

#[post("/fcu-output", data = "<args>")]
pub fn set_fcu_output(
    observer_handler: &State<Arc<ObserverHandler>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() != 3 {
        return format_response(format!("{} <name> <state>\n", args[0]), false);
    }

    let channel = match args[1].as_str() {
        "igniter" => fcu_hal::OutputChannel::SolidMotorIgniter,
        _ => {
            return format_response(
                format!("'{}' is not a valid output name!", args[1].as_str()),
                false,
            );
        }
    };

    let state = match match_state(args[2].as_str()) {
        Some(state) => state,
        None => {
            return format_response(
                format!("'{}' is not a valid output state!", args[2].as_str()),
                false,
            );
        }
    };

    send_command(
        observer_handler,
        NetworkAddress::FlightController,
        Packet::VehicleCommand(VehicleCommand::SetOutputChannel { channel, state }),
    )
}
