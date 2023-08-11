#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use hal::comms_hal::{NetworkAddress, Packet, self};

pub const NETWORK_MAP_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub enum CommsError {
    SerializationError(comms_hal::SerializationError),
    UnknownNetworkAddress,
}

pub struct CommsManager {
    network_map: [Option<(NetworkAddress, [u8; 4])>; NETWORK_MAP_SIZE],
}

impl CommsManager {
    pub fn new() -> Self {
        Self {
            network_map: [None; NETWORK_MAP_SIZE],
        }
    }

    pub fn process_packet(
        &self,
        packet: Packet,
        destination: NetworkAddress,
        buffer: &mut [u8],
    ) -> Result<(usize, [u8; 4]), CommsError> {
        let addr_size = destination.serialize(&mut buffer[1..]);
        if let Err(e) = addr_size {
            return Err(CommsError::SerializationError(e));
        }
        let addr_size = addr_size.unwrap();

        if addr_size > 255 {
            panic!("Address size is too large!");
        }

        buffer[0] = addr_size as u8;

        let size = packet.serialize(&mut buffer[(1 + addr_size)..]);
        if let Err(e) = size {
            return Err(CommsError::SerializationError(e));
        }
        let size = size.unwrap();

        let ip = self.network_address_to_ip(destination);
        if let Some(ip) = ip {
            Ok((addr_size + size, ip))
        } else {
            Err(CommsError::UnknownNetworkAddress)
        }
    }

    pub fn extract_packet(
        &mut self,
        buffer: &mut [u8],
        source_address: [u8; 4],
    ) -> Result<(Packet, NetworkAddress), CommsError> {
        let addr_size = buffer[0] as usize;

        let addr = NetworkAddress::deserialize(&mut buffer[1..(1 + addr_size)]);
        if let Err(e) = addr {
            return Err(CommsError::SerializationError(e));
        }
        let addr = addr.unwrap();

        let packet = Packet::deserialize(&mut buffer[(1 + addr_size)..]);
        if let Err(e) = packet {
            return Err(CommsError::SerializationError(e));
        }
        let packet = packet.unwrap();

        self.map_network_address(&addr, source_address);

        Ok((packet, addr))
    }

    pub fn network_address_to_ip(&self, address: NetworkAddress) -> Option<[u8; 4]> {
        for mapping in &self.network_map {
            if mapping.is_none() {
                break;
            }

            let mapping = mapping.as_ref().unwrap();
            if mapping.0 == address {
                return Some(mapping.1);
            }
        }

        None
    }

    pub fn ip_to_network_address(&self, ip: [u8; 4]) -> Option<NetworkAddress> {
        for mapping in &self.network_map {
            if mapping.is_none() {
                break;
            }

            let mapping = mapping.as_ref().unwrap();
            if mapping.1 == ip {
                return Some(mapping.0);
            }
        }

        None
    }

    fn map_network_address(&mut self, address: &NetworkAddress, ip: [u8; 4]) {
        let mut mapped = false;
        for mapping in &mut self.network_map {
            if mapping.is_none() {
                *mapping = Some((*address, ip));
                mapped = true;
                break;
            }

            let mapping = mapping.as_mut().unwrap();
            if mapping.0 == *address {
                *mapping = (*address, ip);
                mapped = true;
                break;
            }
        }

        if !mapped {
            panic!("Network map is full!");
        }
    }
}

#[cfg(test)]
pub mod tests {
    use hal::comms_hal::{NetworkAddress, PACKET_BUFFER_SIZE};
    use hal::comms_hal::tests_data::PACKET_TEST_DEFAULTS;

    #[test]
    fn test_serialization_deserialization() {
        let mut buffer = [0; PACKET_BUFFER_SIZE];
        let mut comms_manager = super::CommsManager::new();
        let dummy_address = [123, 0, 255, 42];

        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            comms_manager.map_network_address(address, dummy_address);
            for packet in &PACKET_TEST_DEFAULTS {
                println!("Testing packet: {:?} to address: {:?}", packet, address);
                let (size, _) = comms_manager.process_packet(packet.clone(), address.clone(), &mut buffer).unwrap();
                let (deserialized_packet, deserialized_address) = comms_manager.extract_packet(&mut buffer[0..size], dummy_address).unwrap();

                assert_eq!(deserialized_packet, *packet);
                assert_eq!(deserialized_address, *address);
            }
        }
    }

    const NETWORK_ADDRESS_TEST_DEFAULTS: [NetworkAddress; 8] = [
        NetworkAddress::FlightController,
        NetworkAddress::EngineController(0),
        NetworkAddress::EngineController(42),
        NetworkAddress::EngineController(201),
        NetworkAddress::GroundCamera(1),
        NetworkAddress::GroundCamera(70),
        NetworkAddress::GroundCamera(255),
        NetworkAddress::Broadcast,
    ];
}