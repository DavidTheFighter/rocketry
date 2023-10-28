use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    time::Duration,
};

use shared::{
    comms_hal::Packet,
    ecu_hal::{ECUSolenoidValve, FuelTankState},
};
use rocket::{
    serde::{json::Json, Serialize},
    State,
};

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CommandResponse {
    text_response: String,
    success: bool,
}

fn send_command(
    packet_sender: &Arc<Mutex<Sender<Packet>>>,
    packet: Packet,
) -> Json<CommandResponse> {
    match packet_sender.lock().unwrap().send(packet.clone()) {
        Ok(_) => Json(CommandResponse {
            text_response: String::from(format!("Sent '${:?}' command to ECU", packet)),
            success: true,
        }),
        Err(err) => Json(CommandResponse {
            text_response: String::from(format!(
                "Failed to send '${:?}' command, got {:?}",
                packet, err
            )),
            success: false,
        }),
    }
}

fn match_valve(valve: &str) -> Option<ECUSolenoidValve> {
    match valve {
        "ig_fuel" => Some(ECUSolenoidValve::IgniterFuelMain),
        "ig_gox" => Some(ECUSolenoidValve::IgniterGOxMain),
        "press" => Some(ECUSolenoidValve::FuelPress),
        "vent" => Some(ECUSolenoidValve::FuelVent),
        _ => None,
    }
}

fn valve_name_list() -> String {
    String::from("Valves: [ig_fuel, ig_gox, press, vent]")
}

#[post("/valve", data = "<args>")]
pub fn valve(
    packet_sender: &State<Arc<Mutex<Sender<Packet>>>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() == 1 {
        return Json(CommandResponse {
            text_response: format!(
                "{} - States: [0, off, closed, 1, on, open]",
                valve_name_list()
            ),
            success: true,
        });
    }

    if args.len() == 2 {
        return Json(CommandResponse {
            text_response: String::from("Not enough args: valve <name> <state>"),
            success: true,
        });
    }

    let valve = match match_valve(args[1].as_str()) {
        Some(valve) => valve,
        None => {
            return Json(CommandResponse {
                text_response: format!("Failed, '{}' is not a valid valve name!", args[1].as_str()),
                success: false,
            })
        }
    };

    let state = match args[2].as_str() {
        "1" | "on" | "open" => true,
        "0" | "off" | "closed" => false,
        _ => {
            return Json(CommandResponse {
                text_response: format!("Failed, '{}' is not a valid state name!", args[2].as_str()),
                success: false,
            })
        }
    };

    send_command(packet_sender, Packet::SetSolenoidValve { valve, state })
}

#[post("/testvalve", data = "<args>")]
pub fn testvalve(
    packet_sender: &State<Arc<Mutex<Sender<Packet>>>>,
    args: Json<Vec<String>>,
) -> Json<CommandResponse> {
    if args.len() == 1 {
        return Json(CommandResponse {
            text_response: valve_name_list(),
            success: true,
        });
    }

    let valve = match match_valve(args[1].as_str()) {
        Some(valve) => valve,
        None => {
            return Json(CommandResponse {
                text_response: format!("Failed, '{}' is not a valid valve name!", args[1].as_str()),
                success: false,
            })
        }
    };

    let return_val = send_command(
        packet_sender,
        Packet::SetSolenoidValve { valve, state: true },
    );

    let packet_sender = packet_sender.lock().unwrap().clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        packet_sender
            .send(Packet::SetSolenoidValve {
                valve,
                state: false,
            })
            .unwrap();
    });

    return_val
}

#[post("/testspark")]
pub fn testspark(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    let return_val = send_command(packet_sender, Packet::SetSparking(true));

    let packet_sender = packet_sender.lock().unwrap().clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1000));
        packet_sender.send(Packet::SetSparking(false)).unwrap();
    });

    return_val
}

#[post("/fire")]
pub fn fire(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    send_command(packet_sender, Packet::FireIgniter)
}

#[post("/press")]
pub fn pressurize(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    send_command(
        packet_sender,
        Packet::TransitionFuelTankState(FuelTankState::Pressurized),
    )
}

#[post("/depress")]
pub fn depressurize(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    send_command(
        packet_sender,
        Packet::TransitionFuelTankState(FuelTankState::Depressurized),
    )
}

#[post("/tankidle")]
pub fn tankidle(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    send_command(
        packet_sender,
        Packet::TransitionFuelTankState(FuelTankState::Idle),
    )
}
