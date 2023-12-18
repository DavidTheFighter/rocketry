use std::any::Any;
use std::net::UdpSocket;

use shared::fcu_hal::{OutputChannel, PwmChannel, FcuDriver, FcuTelemetryFrame, FcuDevStatsFrame, FcuHardwareData};
use shared::comms_hal::{Packet, NetworkAddress};
use strum::EnumCount;

const BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub struct FcuDriverSim {
    outputs: [bool; OutputChannel::COUNT],
    pwm: [f32; PwmChannel::COUNT],
    continuities: [bool; OutputChannel::COUNT],
    socket: Option<UdpSocket>,
    pub packet_log_queue: Vec<Packet>,
    pub current_sim_timestamp: f32,
    pub last_sim_timestamp_update_timestamp: f64,
    pub last_telem_packet: Option<FcuTelemetryFrame>,
    pub last_dev_stats_packet: Option<FcuDevStatsFrame>,
}

impl FcuDriver for FcuDriverSim {
    fn timestamp(&self) -> f32 {
        let elapsed = get_timestamp() - self.last_sim_timestamp_update_timestamp;

        self.current_sim_timestamp + (elapsed as f32)
    }

    fn set_output_channel(&mut self, channel: OutputChannel, state: bool) {
        self.outputs[channel.index()] = state;
    }

    fn set_pwm_channel(&mut self, channel: PwmChannel, duty_cycle: f32) {
        self.pwm[channel as usize] = duty_cycle;
    }

    fn get_output_channel(&self, channel: OutputChannel) -> bool {
        self.outputs[channel.index()]
    }

    fn get_output_channel_continuity(&self, channel: OutputChannel) -> bool {
        self.continuities[channel.index()]
    }

    fn get_pwm_channel(&self, channel: PwmChannel) -> f32 {
        self.pwm[channel as usize]
    }

    // fn send_packet(&mut self, packet: Packet, _destination: NetworkAddress) {
    //     let mut buffer = [0_u8; BUFFER_SIZE];

    //     if let Packet::FcuTelemetry(frame) = &packet {
    //         self.last_telem_packet = Some(frame.clone());
    //     }

    //     if let Packet::FcuDevStatsFrame(frame) = &packet {
    //         self.last_dev_stats_packet = Some(frame.clone());
    //     }

    //     self.packet_log_queue.push(packet.clone());

    //     if let Some(socket) = self.socket.as_mut() {
    //         // match packet.serialize(&mut buffer) {
    //         //     Ok(size) => {
    //         //         let address = "127.0.0.1:25565";

    //         //         if let Err(err) = socket.send_to(&buffer[0..size], address) {
    //         //             println!("FcuDriverSim: Failed to send packet: {err}");
    //         //         }
    //         //     }
    //         //     Err(err) => {
    //         //         println!("FcuDriverSim: Failed to serialize packet: {:?}", err);
    //         //     }
    //         // }
    //     }
    // }

    fn hardware_data(&self) -> FcuHardwareData {
        FcuHardwareData::default()
    }

    fn erase_flash_chip(&mut self) {
        // Nothing
    }

    fn enable_logging_to_flash(&mut self) {
        // Nothing
    }

    fn disable_logging_to_flash(&mut self) {
        // Nothing
    }

    fn retrieve_log_flash_page(&mut self, _addr: u32) {
        // Nothing
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl FcuDriverSim {
    pub fn new() -> Self {
        Self {
            outputs: [false; OutputChannel::COUNT],
            pwm: [0.0; PwmChannel::COUNT],
            continuities: [false; OutputChannel::COUNT],
            socket: None,
            packet_log_queue: Vec::new(),
            current_sim_timestamp: 0.0,
            last_sim_timestamp_update_timestamp: get_timestamp(),
            last_telem_packet: None,
            last_dev_stats_packet: None,
        }
    }

    pub fn update_timestamp(&mut self, sim_time: f32) {
        self.current_sim_timestamp = sim_time;
        self.last_sim_timestamp_update_timestamp = get_timestamp();
    }

    pub fn set_output_channel_continuity(&mut self, channel: OutputChannel, state: bool) {
        self.continuities[channel.index()] = state;
    }
}

fn get_timestamp() -> f64 {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    duration.as_secs_f64()
}