use serde::{Deserialize, Serialize};

use crate::big_brother::BigBrotherError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetworkMapEntry<T> {
    pub network_address: T,
    pub ip: [u8; 4],
    pub port: u16,
    pub interface_index: u8,
    pub to_counter: u16,
    pub from_counter: u16,
    pub from_session_id: u32,
}

pub struct NetworkMap<T, const NETWORK_MAP_SIZE: usize> {
    network_map: [Option<NetworkMapEntry<T>>; NETWORK_MAP_SIZE],
}

impl<T, const NETWORK_MAP_SIZE: usize> NetworkMap<T, NETWORK_MAP_SIZE>
where
    T: Copy + PartialEq + Eq,
{
    pub fn new() -> Self {
        Self {
            network_map: [None; NETWORK_MAP_SIZE],
        }
    }

    pub fn map_network_address(
        &mut self,
        from_address: T,
        ip: [u8; 4],
        port: u16,
        interface_index: u8,
    ) -> Result<&mut NetworkMapEntry<T>, BigBrotherError> {
        for mapping in self.network_map.iter_mut() {
            match mapping {
                Some(mapping) => {
                    if mapping.network_address == from_address {
                        *mapping = NetworkMapEntry {
                            network_address: from_address,
                            ip,
                            port,
                            interface_index,
                            to_counter: mapping.to_counter,
                            from_counter: mapping.from_counter,
                            from_session_id: mapping.from_session_id,
                        };

                        return Ok(mapping);
                    }
                },
                None => {
                    *mapping = Some(NetworkMapEntry {
                        network_address: from_address,
                        ip,
                        port,
                        interface_index,
                        to_counter: 0,
                        from_counter: 0,
                        from_session_id: 0,
                    });

                    return Ok(mapping.as_mut().unwrap());
                }
            }
        }

        Err(BigBrotherError::NetworkMapFull)
    }

    pub fn get_address_mapping(
        &mut self,
        address: T,
    ) -> Result<&mut NetworkMapEntry<T>, BigBrotherError> {
        for mapping in &mut self.network_map {
            match mapping {
                Some(mapping) => {
                    if mapping.network_address == address {
                        return Ok(mapping);
                    }
                }
                None => break,
            }
        }

        Err(BigBrotherError::UnknownNetworkAddress)
    }

    pub fn update_session_id(
        &mut self,
        address: T,
        session_id: u32,
    ) -> Result<(), BigBrotherError> {
        let mapping = self.get_address_mapping(address)?;

        if session_id != mapping.from_session_id {
            mapping.from_counter = 0;
            mapping.from_session_id = session_id;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::big_brother::UDP_PORT;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TestNetworkAddress {
        FlightController,
        EngineController(u8),
        Camera(u8),
        Broadcast,
    }

    pub const NETWORK_ADDRESS_TEST_DEFAULTS: [TestNetworkAddress; 8] = [
        TestNetworkAddress::FlightController,
        TestNetworkAddress::EngineController(0),
        TestNetworkAddress::EngineController(42),
        TestNetworkAddress::EngineController(201),
        TestNetworkAddress::Camera(1),
        TestNetworkAddress::Camera(70),
        TestNetworkAddress::Camera(255),
        TestNetworkAddress::Broadcast,
    ];

    #[test]
    fn test_mapping_iter() {
        let mut network_map = NetworkMap::<TestNetworkAddress, 32>::new();

        let mut i = 0;
        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            network_map
                .map_network_address(*address, [123 + i, 0 + i, 200 + i, 42 + i], UDP_PORT, i % 2)
                .unwrap();
            i += 1;
        }

        i = 0;
        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            let mapping = network_map.get_address_mapping(*address).unwrap();
            println!("Testing address: {:?} with mapping: {:?}", address, mapping);

            assert_eq!(mapping.network_address, *address);
            assert_eq!(mapping.ip, [123 + i, 0 + i, 200 + i, 42 + i]);
            assert_eq!(mapping.interface_index, i % 2);
            i += 1;
        }
    }
}
