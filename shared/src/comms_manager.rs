use crate::comms_hal::{self, NetworkAddress, Packet};

// Serialization format:
// u8: to_address_size
// u8: from_address_size
// [u8; to_address_size]: to_address
// [u8; from_address_size]: from_address
// [u8; rest]: packet

#[derive(Debug, Clone)]
pub enum CommsError {
    SerializationError(comms_hal::SerializationError),
    UnknownNetworkAddress,
}

pub struct CommsManager<const NETWORK_MAP_SIZE: usize> {
    network_map: [Option<(NetworkAddress, [u8; 4])>; NETWORK_MAP_SIZE],
    broadcast_addr: [u8; 4],
    host_addr: NetworkAddress,
}

impl<const NETWORK_MAP_SIZE: usize> CommsManager<NETWORK_MAP_SIZE> {
    pub fn new(host_addr: NetworkAddress) -> Self {
        Self {
            network_map: [None; NETWORK_MAP_SIZE],
            broadcast_addr: [255, 255, 255, 255],
            host_addr,
        }
    }

    pub fn process_packet(
        &self,
        packet: &Packet,
        destination: NetworkAddress,
        buffer: &mut [u8],
    ) -> Result<(usize, [u8; 4]), CommsError> {
        let to_addr_size = destination.serialize(&mut buffer[2..]);
        if let Err(e) = to_addr_size {
            return Err(CommsError::SerializationError(e));
        }
        let to_addr_size = to_addr_size.unwrap();

        let from_addr_size = self.host_addr.serialize(&mut buffer[(2 + to_addr_size)..]);
        if let Err(e) = from_addr_size {
            return Err(CommsError::SerializationError(e));
        }
        let from_addr_size = from_addr_size.unwrap();

        buffer[0] = to_addr_size as u8;
        buffer[1] = from_addr_size as u8;

        let size = packet.serialize(&mut buffer[(2 + to_addr_size + from_addr_size)..]);
        if let Err(e) = size {
            return Err(CommsError::SerializationError(e));
        }
        let size = size.unwrap();

        let ip = self.network_address_to_ip(destination);
        if let Some(ip) = ip {
            Ok((2 + to_addr_size + from_addr_size + size, ip))
        } else {
            Err(CommsError::UnknownNetworkAddress)
        }
    }

    pub fn extract_packet(
        &mut self,
        buffer: &mut [u8],
        source_address: [u8; 4],
    ) -> Result<(Packet, NetworkAddress), CommsError> {
        let to_addr_size = buffer[0] as usize;
        let from_addr_size = buffer[1] as usize;

        let to_addr_pos = 2;
        let from_addr_pos = to_addr_pos + to_addr_size;
        let packet_pos = from_addr_pos + from_addr_size;

        // let to_addr = NetworkAddress::deserialize(&mut buffer[to_addr_pos..from_addr_pos]);
        // if let Err(e) = to_addr {
        //     return Err(CommsError::SerializationError(e));
        // }
        // let to_addr = to_addr.unwrap();

        let from_addr = NetworkAddress::deserialize(&mut buffer[from_addr_pos..packet_pos]);
        if let Err(e) = from_addr {
            return Err(CommsError::SerializationError(e));
        }
        let from_addr = from_addr.unwrap();

        let packet = Packet::deserialize(&mut buffer[packet_pos..]);
        if let Err(e) = packet {
            return Err(CommsError::SerializationError(e));
        }
        let packet = packet.unwrap();

        self.map_network_address(&from_addr, source_address);

        Ok((packet, from_addr))
    }

    pub fn network_address_to_ip(&self, address: NetworkAddress) -> Option<[u8; 4]> {
        if let NetworkAddress::Broadcast = address {
            return Some(self.broadcast_addr);
        }

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
        if ip == self.broadcast_addr {
            return Some(NetworkAddress::Broadcast);
        }

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

    pub fn set_broadcast_address(&mut self, ip: [u8; 4]) {
        self.broadcast_addr = ip;
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

pub struct CommsManagerIterator<'a, const NETWORK_MAP_SIZE: usize> {
    comms_manager: &'a CommsManager<NETWORK_MAP_SIZE>,
    index: usize,
}

impl<'a, const NETWORK_MAP_SIZE: usize> IntoIterator for &'a CommsManager<NETWORK_MAP_SIZE> {
    type Item = (NetworkAddress, [u8; 4]);
    type IntoIter = CommsManagerIterator<'a, NETWORK_MAP_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        CommsManagerIterator {
            comms_manager: self,
            index: 0,
        }
    }
}

impl<'a, const NETWORK_MAP_SIZE: usize> Iterator for CommsManagerIterator<'a, NETWORK_MAP_SIZE> {
    type Item = (NetworkAddress, [u8; 4]);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < NETWORK_MAP_SIZE {
            let mapping = self.comms_manager.network_map[self.index];
            self.index += 1;

            if let Some(mapping) = mapping {
                return Some(mapping);
            }
        }

        None
    }
}

impl CommsError {
    pub fn error_str(&self) -> &str {
        match self {
            CommsError::SerializationError(_) => "Serialization error",
            CommsError::UnknownNetworkAddress => "Unknown network address",
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::comms_hal::{tests_data::PACKET_TEST_DEFAULTS, NetworkAddress, PACKET_BUFFER_SIZE};

    #[test]
    fn test_serialization_deserialization() {
        let host_addr = NetworkAddress::EngineController(19);
        let mut buffer = [0; PACKET_BUFFER_SIZE];
        let mut comms_manager = super::CommsManager::<32>::new(host_addr);
        let dummy_address = [123, 0, 255, 42];

        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            comms_manager.map_network_address(address, dummy_address);
            for packet in &PACKET_TEST_DEFAULTS {
                println!("Testing packet: {:?} to address: {:?}", packet, address);
                let (size, _) = comms_manager
                    .process_packet(packet, address.clone(), &mut buffer)
                    .unwrap();
                let (deserialized_packet, deserialized_address) = comms_manager
                    .extract_packet(&mut buffer[0..size], dummy_address)
                    .unwrap();

                assert_eq!(deserialized_packet, *packet);
                assert_eq!(deserialized_address, host_addr);
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

    #[test]
    fn test_mapping_iter() {
        let mut comms_manager = super::CommsManager::<32>::new(NetworkAddress::MissionControl);

        let mut i = 0;
        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            comms_manager.network_map[(i as usize) * 2] =
                Some((*address, [123 + i, 0 + i, 200 + i, 42 + i]));
            i += 1;
        }

        let mut count = 0;
        for (address, ip) in &comms_manager {
            assert_eq!(address, NETWORK_ADDRESS_TEST_DEFAULTS[count as usize]);
            assert_eq!(ip, [123 + count, 0 + count, 200 + count, 42 + count]);
            count += 1;
        }

        assert_eq!(count as usize, NETWORK_ADDRESS_TEST_DEFAULTS.len());
    }

    #[test]
    fn test_network_addr_size() {
        let mut buffer = [0_u8; 1024];

        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            let size = address.serialize(&mut buffer[1..]).unwrap();
            assert!(size <= 255);
        }
    }
}
