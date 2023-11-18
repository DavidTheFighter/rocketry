#![no_std]

use postcard::{ser_flavors::{Cobs, Slice}, serialize_with_flavor, from_bytes_cobs};
use serde::{Serialize, Deserialize};
use smoltcp::{
    iface::{self, Interface},
    phy::Device,
    storage::PacketMetadata,
    wire, time::Instant,
    socket::udp::{Socket as UdpSocket, PacketBuffer, UdpMetadata, SendError, RecvError},
};
use strum_macros::{FromRepr, EnumIter, EnumDiscriminants};

pub const BOOTLOADER_PORT: u16 = 4080;
pub const PROGRAM_CHUNK_LENGTH: usize = 256;
pub const PROGRAM_PACKET_LENGTH: usize = PROGRAM_CHUNK_LENGTH + 6;

pub const DEVICE_MAC_ADDR: [u8; 6] = [0x00, 0x80, 0xE1, 0x00, 0x00, 0x01];
pub const DEVICE_IP_ADDR: wire::Ipv4Address = wire::Ipv4Address::new(169, 254, 0, 7);
pub const DEVICE_CIDR_LENGTH: u8 = 16;

#[derive(Clone, Serialize, Deserialize, FromRepr, EnumDiscriminants)]
#[strum_discriminants(name(BootloaderNetworkCommandIndex))]
#[strum_discriminants(derive(EnumIter))]
pub enum BootloaderNetworkCommand {
    PingBootloader,
    Response {
        command: u8,
        success: bool,
    },
    EraseFlash {
        sector: u16,
    },
    ProgramFlash {
        flash_offset: u32,
        buffer_offset: u16,
        buffer_length: u16,
    },
    VerifyFlash {
        checksum: u128,
    },
}

pub enum BootloaderAction<'a> {
    None,
    Ping,
    EraseFlash {
        sector: u16,
    },
    ProgramFlash {
        offset: u32,
        data: &'a [u8],
    },
    VerifyFlash {
        start_offset: u32,
        end_offset: u32,
        checksum: u128,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BootloaderError {
    SmoltcpSendUnaddressable,
    SmoltcpSendBufferFull,
    SmoltcpRecvExhausted,
    SerializationError,
}

pub struct ProgramCommandData<'a> {
    pub address: u32,
    pub length: u16,
    pub data: &'a [u8],
}

pub struct SocketBuffer {
    pub rx_storage: [u8; 512],
    pub rx_metadata_storage: [PacketMetadata<UdpMetadata>; 8],
    pub tx_storage: [u8; 512],
    pub tx_metadata_storage: [PacketMetadata<UdpMetadata>; 8],
}

impl SocketBuffer {
    pub const fn new() -> Self {
        Self {
            rx_storage: [0_u8; 512],
            rx_metadata_storage: [PacketMetadata::EMPTY; 8],
            tx_storage: [0_u8; 512],
            tx_metadata_storage: [PacketMetadata::EMPTY; 8],
        }
    }

    pub fn into_udp_socket_buffers(&mut self) -> (PacketBuffer, PacketBuffer) {
        (
            PacketBuffer::new(&mut self.rx_metadata_storage[..], &mut self.rx_storage[..]),
            PacketBuffer::new(&mut self.tx_metadata_storage[..], &mut self.tx_storage[..]),
        )
    }
}

pub struct Bootloader<'a, D>
where
    D: Device + ?Sized,
{
    interface: Interface,
    device: &'a mut D,
    sockets_set: iface::SocketSet<'a>,
    udp_socket_handle: iface::SocketHandle,
}

impl<'a, D> Bootloader<'a, D>
where
    D: Device + ?Sized,
{
    pub fn new(
        config: iface::Config,
        device: &'a mut D,
        sockets: &'a mut [iface::SocketStorage<'a>; 1],
        socket_buffer: &'a mut SocketBuffer,
        timestamp: Instant,
    ) -> Self {
        let (rx_buffer, tx_buffer) = socket_buffer.into_udp_socket_buffers();

        let mut interface = Interface::new(
            config,
            device,
            timestamp,
        );

        interface.update_ip_addrs(|addr| {
            addr.push(wire::IpCidr::Ipv4(wire::Ipv4Cidr::new(DEVICE_IP_ADDR, DEVICE_CIDR_LENGTH))).ok();
        });

        let mut sockets_set = iface::SocketSet::new(&mut sockets[..]);

        let mut udp_socket = UdpSocket::new(rx_buffer, tx_buffer);
        udp_socket.bind(BOOTLOADER_PORT).expect("failed to bind UDP socket");

        let udp_socket_handle = sockets_set.add(udp_socket);

        Self {
            interface,
            device,
            sockets_set,
            udp_socket_handle,
        }
    }

    pub fn poll<'b>(
        &mut self,
        timestamp: Instant,
        working_buffer: &'b mut [u8; 512],
    ) -> Result<BootloaderAction<'b>, BootloaderError> {
        self.interface.poll(
            timestamp,
            self.device,
            &mut self.sockets_set,
        );

        if let Some((source, command)) = self.receive(working_buffer)? {
            match command {
                BootloaderNetworkCommand::PingBootloader => {
                    self.send(source, BootloaderNetworkCommand::Response {
                        command: BootloaderNetworkCommandIndex::PingBootloader as u8,
                        success: true,
                    }, working_buffer)?;

                    return Ok(BootloaderAction::Ping);
                },
                BootloaderNetworkCommand::EraseFlash { sector } => {
                    return Ok(BootloaderAction::EraseFlash { sector });
                },
                BootloaderNetworkCommand::ProgramFlash { flash_offset, buffer_offset, buffer_length } => {
                    let slice_start = buffer_offset as usize;
                    let slice_end = (buffer_offset + buffer_length) as usize;

                    return Ok(BootloaderAction::ProgramFlash {
                        offset: flash_offset,
                        data: &working_buffer[slice_start..slice_end],
                    });
                },
                BootloaderNetworkCommand::VerifyFlash { checksum } => {
                    return Ok(BootloaderAction::VerifyFlash {
                        start_offset: 0,
                        end_offset: 0,
                        checksum: checksum,
                    });
                },
                BootloaderNetworkCommand::Response { command: _, success: _ } => {
                    return Ok(BootloaderAction::None);
                },
            }
        }

        Ok(BootloaderAction::None)
    }

    fn send(
        &mut self,
        dest: wire::IpEndpoint,
        command: BootloaderNetworkCommand,
        buffer: &mut [u8],
    ) -> Result<(), BootloaderError> {
        let udp_socket = self.sockets_set.get_mut::<UdpSocket>(self.udp_socket_handle);

        if let Some(size) = command.serialize(buffer) {
            udp_socket.send_slice(&buffer[..size], dest)
                .map_err(|send_err| BootloaderError::from(send_err))
        } else {
            Err(BootloaderError::SerializationError)
        }
    }

    fn receive(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<Option<(wire::IpEndpoint, BootloaderNetworkCommand)>, BootloaderError> {
        let udp_socket = self.sockets_set.get_mut::<UdpSocket>(self.udp_socket_handle);

        if !udp_socket.can_recv() {
            return Ok(None);
        }

        match udp_socket.recv_slice(buffer) {
            Ok((_size, source)) => {
                if let Some(command) = BootloaderNetworkCommand::deserialize(buffer) {
                    Ok(Some((source.endpoint, command)))
                } else {
                    Ok(None)
                }
            },
            Err(recv_err) => {
                Err(BootloaderError::from(recv_err))
            },
        }
    }
}

impl BootloaderNetworkCommand {
    pub fn serialize(&self, buffer: &mut [u8]) -> Option<usize> {
        let result = match Cobs::try_new(Slice::new(&mut buffer[1..])) {
            Ok(flavor) => {
                let serialized =
                    serialize_with_flavor::<BootloaderNetworkCommand, Cobs<Slice>, &mut [u8]>(self, flavor);

                match serialized {
                    Ok(output_buffer) => {
                        Some(output_buffer.len())
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        if let Some(size) = result {
            buffer[0] = size as u8;
        }

        result
    }

    pub fn deserialize(buffer: &mut [u8]) -> Option<BootloaderNetworkCommand> {
        let size = buffer[0] as usize;

        match from_bytes_cobs(&mut buffer[1..(size + 1)]) {
            Ok(command) => Some(command),
            Err(_) => None,
        }
    }

    pub fn retrieve_program_data(packet_buffer: &mut [u8], output_buffer: &mut [u8; PROGRAM_CHUNK_LENGTH]) {
        let size = packet_buffer[0] as usize;

        for (i, o) in packet_buffer.iter().skip(size).zip(output_buffer.iter_mut()) {
            *o = *i;
        }
    }
}

impl From<SendError> for BootloaderError {
    fn from(send_error: SendError) -> Self {
        match send_error {
            SendError::Unaddressable => BootloaderError::SmoltcpSendUnaddressable,
            SendError::BufferFull => BootloaderError::SmoltcpSendBufferFull,
        }
    }
}

impl From<RecvError> for BootloaderError {
    fn from(recv_error: RecvError) -> Self {
        match recv_error {
            RecvError::Exhausted => BootloaderError::SmoltcpRecvExhausted,
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::BootloaderNetworkCommandIndex;

    #[test]
    fn test_discriminant_values() {
        for (i, command_index) in BootloaderNetworkCommandIndex::iter().enumerate() {
            assert_eq!(u8::try_from(i).unwrap(), command_index as u8);
        }
    }
}
