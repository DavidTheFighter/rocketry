use std::net::UdpSocket;

use hal::fcu_hal::{OutputChannel, PwmChannel, FcuDriver};
use hal::comms_hal::{Packet, NetworkAddress};
use strum::EnumCount;

const BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub struct FcuDriverSim {
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
    socket: Option<UdpSocket>,
}

impl FcuDriver for FcuDriverSim {
    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        self.outputs[channel as usize] = state;
    }

    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32) {
        self.pwm[channel as usize] = duty_cycle;
    }

    fn get_output_channel(&self, channel: OutputChannel) -> bool {
        self.outputs[channel as usize]
    }

    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool {
        self.continuities[channel as usize]
    }

    fn get_pwm_channel(&self, channel: PwmChannel) -> f32 {
        self.pwm[channel as usize]
    }

    fn send_packet(&mut self, packet: Packet, _destination: NetworkAddress) {
        let mut buffer = [0_u8; BUFFER_SIZE];
        let socket = self.socket.as_mut().expect("FcuDriverSim: Socket not initialized");

        match packet.serialize(&mut buffer) {
            Ok(size) => {
                let address = "127.0.0.1:25565";

                if let Err(err) = socket.send_to(&buffer[0..size], address) {
                    println!("FcuDriverSim: Failed to send packet: {err}");
                }
            }
            Err(err) => {
                println!("FcuDriverSim: Failed to serialize packet: {:?}", err);
            }
        }
    }
}

impl FcuDriverSim {
    pub const fn new() -> Self {
        Self {
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
            socket: None,
        }
    }

    pub fn init(&mut self) {
        self.socket = Some(UdpSocket::bind("0.0.0.0:25564").unwrap());
    }
}