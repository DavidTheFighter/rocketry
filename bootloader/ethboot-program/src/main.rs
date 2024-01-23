use std::{
    fs::File,
    io::{self, Read, Write},
    net::UdpSocket,
};

use big_brother::{interface::std_interface::StdInterface, BigBrother};
use ethboot_shared::{BootloaderNetworkCommand, BOOTLOADER_PORT, PROGRAM_CHUNK_LENGTH};
use serde::{Deserialize, Deserializer};
use shared::comms_hal::{NetworkAddress, Packet};

const WORKING_BUFFER_SIZE: usize = PROGRAM_CHUNK_LENGTH * 2;

#[derive(Debug, Deserialize)]
struct FlashSector {
    #[serde(deserialize_with = "from_hex")]
    start_offset: u32,
    #[serde(deserialize_with = "from_hex")]
    end_offset: u32,
}

#[derive(Debug, Deserialize)]
struct BootloaderDefinition {
    #[serde(deserialize_with = "from_hex")]
    app_offset: u32,
    sectors: Vec<FlashSector>,
}

fn main() {
    let cmd_args = std::env::args().collect::<Vec<_>>();
    println!("Command line args: {:?}", cmd_args);

    if cmd_args.len() != 4 {
        println!(
            "Usage: {} <bootloader_definition_file> <binary_file> <ip_addr>",
            cmd_args[0]
        );
        return;
    }

    let bootloader_definition_file = &cmd_args[1];
    let binary_file = &cmd_args[2];
    let mcu_blt_address = format!("{}:{}", cmd_args[3], BOOTLOADER_PORT);
    // let mcu_app_address = format!("{}:{}", cmd_args[3], UDP_RECV_PORT);

    let mut udp_socket =
        UdpSocket::bind(format!("0.0.0.0:{}", BOOTLOADER_PORT)).expect("Failed to bind UDP socket");
    udp_socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");
    let mut buffer = [0u8; WORKING_BUFFER_SIZE];

    let mut bb_interface =
        StdInterface::new([169, 254, 255, 255]).expect("Failed to create bb interface");
    let mut comms: BigBrother<'_, 64, Packet, NetworkAddress> = BigBrother::new(
        NetworkAddress::EthbootProgrammer,
        rand::random(),
        NetworkAddress::Broadcast,
        [Some(&mut bb_interface), None],
    );

    let mut timestamp = 0;

    // Wait until we get a ping back
    println!("Waiting for ping response");
    loop {
        comms.poll_1ms(timestamp);

        if timestamp % 100 == 0 {
            let command = BootloaderNetworkCommand::PingBootloader;
            if let Some(size) = command.serialize(&mut buffer) {
                udp_socket
                    .send_to(&buffer[..size], mcu_blt_address.clone())
                    .unwrap();
            } else {
                panic!("Failed to serialize command");
            }

            let reset_packet = Packet::ResetMcu {
                magic_number: shared::RESET_MAGIC_NUMBER,
            };
            if let Err(e) = comms.send_packet(&reset_packet, NetworkAddress::FlightController) {
                println!("Failed to send reset packet: {:?}", e);
            }
        }

        if let Err(e) = comms.recv_packet() {
            println!("BB recv error: {:?}", e);
        }

        if let Some(command) = check_for_response(&mut udp_socket, &mut buffer) {
            if let BootloaderNetworkCommand::Response {
                command: _,
                success,
            } = command
            {
                if success {
                    break;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
        timestamp += 1;
    }

    let bootloader_definition = load_bootloader_definition(bootloader_definition_file);
    let binary = load_image_binary(binary_file);

    // Erase sectors that need to be erased
    let mut start_sector = None;
    let mut end_sector = 0_u16;

    for (index, sector) in bootloader_definition.sectors.iter().enumerate() {
        if start_sector.is_none() && bootloader_definition.app_offset <= sector.end_offset {
            start_sector = Some(index as u16);
        }

        if sector.end_offset >= bootloader_definition.app_offset + binary.len() as u32 {
            end_sector = index as u16;
            break;
        }
    }

    let start_sector = start_sector.expect("Failed to find start sector");

    println!(
        "Flash goes from 0x{:X} to 0x{:X}, erasing sectors {} to {} (0x{:X} to 0x{:X})",
        bootloader_definition.app_offset,
        bootloader_definition.app_offset + binary.len() as u32,
        start_sector,
        end_sector,
        bootloader_definition.sectors[start_sector as usize].start_offset,
        bootloader_definition.sectors[end_sector as usize].end_offset,
    );

    for sector in start_sector..=end_sector {
        print!("Erasing sector {}...", sector);
        io::stdout().flush().expect("Failed to flush stdout");

        let command = BootloaderNetworkCommand::EraseFlash {
            sector: sector as u16,
        };
        if let Some(size) = command.serialize(&mut buffer) {
            udp_socket
                .send_to(&buffer[..size], mcu_blt_address.clone())
                .unwrap();
        }

        if let Some(command) = wait_for_response(&mut udp_socket, &mut buffer) {
            if let BootloaderNetworkCommand::Response {
                command: _,
                success,
            } = command
            {
                if success {
                    println!("Done!");
                } else {
                    println!("Failed");
                    return;
                }
            }
        }
    }

    // Program flash

    let mut offset = 0;

    print!("Programming flash ");
    io::stdout().flush().expect("Failed to flush stdout");

    for chunk_bytes in binary.chunks(PROGRAM_CHUNK_LENGTH) {
        let bytes_read = chunk_bytes.len();

        if bytes_read == 0 {
            break;
        }

        // println!("Programming flash at offset 0x{:X} with {} bytes", offset, bytes_read);

        let mut command = BootloaderNetworkCommand::ProgramFlash {
            flash_offset: (bootloader_definition.app_offset + offset) as u32,
            buffer_offset: 0,
            buffer_length: PROGRAM_CHUNK_LENGTH as u16,
        };

        let calc_buffer_offset = calc_bootloader_command_size(&command, &mut buffer) + 1;

        if let BootloaderNetworkCommand::ProgramFlash {
            flash_offset: _,
            buffer_offset,
            buffer_length: _,
        } = &mut command
        {
            *buffer_offset = calc_buffer_offset as u16;
        } else {
            panic!("Command is not ProgramFlash. WTF");
        }

        if let Some(size) = command.serialize(&mut buffer) {
            for (index, byte) in chunk_bytes.iter().enumerate() {
                buffer[calc_buffer_offset + index] = *byte;
            }

            // Fill in the rest of the buffer with 0xFF
            for index in bytes_read..PROGRAM_CHUNK_LENGTH {
                buffer[calc_buffer_offset + index] = 0xFF;
            }

            udp_socket
                .send_to(
                    &buffer[..(size + 1 + PROGRAM_CHUNK_LENGTH)],
                    mcu_blt_address.clone(),
                )
                .unwrap();
        }

        if let Some(command) = wait_for_response(&mut udp_socket, &mut buffer) {
            if let BootloaderNetworkCommand::Response {
                command: _,
                success,
            } = command
            {
                if !success {
                    println!("Failed!");
                    return;
                }
            }
        }

        if offset % ((PROGRAM_CHUNK_LENGTH as u32) * 20) == 0 {
            print!(".");
            io::stdout().flush().expect("Failed to flush stdout");
        }

        offset += bytes_read as u32;
    }

    println!("Done!");

    println!("Verifying flash");

    let mut checksum = 0;
    for byte in binary.iter() {
        checksum += *byte as u128;
    }

    println!("Checksum is {}", checksum);

    let command = BootloaderNetworkCommand::VerifyFlash {
        start_offset: bootloader_definition.app_offset as u32,
        end_offset: bootloader_definition.app_offset + binary.len() as u32,
        checksum,
    };

    if let Some(size) = command.serialize(&mut buffer) {
        udp_socket
            .send_to(&buffer[..size], mcu_blt_address.clone())
            .unwrap();
    } else {
        panic!("Failed to serialize command");
    }

    if let Some(command) = wait_for_response(&mut udp_socket, &mut buffer) {
        if let BootloaderNetworkCommand::Response {
            command: _,
            success,
        } = command
        {
            if !success {
                println!("Failed!");
                return;
            }
        }
    }

    println!("Booting into application");

    let command = BootloaderNetworkCommand::BootIntoApplication;

    if let Some(size) = command.serialize(&mut buffer) {
        udp_socket
            .send_to(&buffer[..size], mcu_blt_address.clone())
            .unwrap();
    } else {
        panic!("Failed to serialize command");
    }
}

fn load_bootloader_definition(bootloader_definition_file: &str) -> BootloaderDefinition {
    // Read file into a string
    let mut file =
        File::open(bootloader_definition_file).expect("Unable to open flash definition file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read flash definition file");

    // Parse the string of data into serde_json::Value.
    let bootloader_definition: BootloaderDefinition =
        serde_json::from_str(&contents).expect("Unable to parse flash definition file");

    bootloader_definition
}

fn load_image_binary(image_binary_file: &str) -> Vec<u8> {
    let mut file = File::open(image_binary_file).expect("Unable to open image binary file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Unable to read image binary file");

    contents
}

fn check_for_response(
    udp_socket: &mut UdpSocket,
    buffer: &mut [u8],
) -> Option<BootloaderNetworkCommand> {
    if let Ok(size) = udp_socket.recv(buffer) {
        let ret = BootloaderNetworkCommand::deserialize(buffer);

        if ret.is_none() {
            println!("Got some garbage: {:?}", &buffer[..size]);
        }

        return ret;
    }

    None
}

fn wait_for_response(
    udp_socket: &mut UdpSocket,
    buffer: &mut [u8; WORKING_BUFFER_SIZE],
) -> Option<BootloaderNetworkCommand> {
    loop {
        if let Some(command) = check_for_response(udp_socket, buffer) {
            return Some(command);
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn calc_bootloader_command_size(command: &BootloaderNetworkCommand, buffer: &mut [u8]) -> usize {
    if let Some(size) = command.serialize(buffer) {
        size
    } else {
        0
    }
}

fn from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    // do better hex decoding than this
    u32::from_str_radix(&s[2..], 16).map_err(serde::de::Error::custom)
}
