use std::sync::{mpsc::{Receiver, Sender}, Mutex, Arc};

use hal::comms_hal::Packet;
use rocket::{serde::{Serialize, json::Json}, State};


#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CommandResponse {
    text_response: String,
    success: bool,
}

fn send_command(packet_sender: &Arc<Mutex<Sender<Packet>>>, packet: Packet) -> Json<CommandResponse> {
    match packet_sender.lock().unwrap().send(packet.clone()) {
        Ok(_) => Json(CommandResponse { 
            text_response: String::from(format!("Sent '${:?}' command to ECU", packet)), 
            success: true,
        }),
        Err(err) => Json(CommandResponse { 
            text_response: String::from(format!("Failed to send '${:?}' command, got {:?}", packet, err)), 
            success: false,
        })
    }
}

#[post("/testvalve")]
pub fn testvalve() -> Json<CommandResponse> {
    Json(CommandResponse { 
        text_response: String::from("Test valve IG Fuel Main"), 
        success: true,
    })
}

#[post("/fire")]
pub fn fire(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {    
    send_command(packet_sender, Packet::FireIgniter)
}

#[post("/press")]
pub fn pressurize(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {    
    send_command(packet_sender, Packet::PressurizeFuelTank)
}

#[post("/depress")]
pub fn depressurize(packet_sender: &State<Arc<Mutex<Sender<Packet>>>>) -> Json<CommandResponse> {
    send_command(packet_sender, Packet::DepressurizeFuelTank)
}