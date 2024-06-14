use serde::{Deserialize, Serialize};

use crate::{big_brother::BigBrotherError, dedupe};

pub const MAX_UPSTREAM_LOCAL_PORTS: usize = 4;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetworkMapEntry<T> {
    pub network_address: T,
    pub ip: [u8; 4],
    pub port: u16,
    pub interface_index: u8,
    pub to_counter: dedupe::CounterType,
    pub from_counter: dedupe::CounterType,
    pub broadcast_counter: dedupe::CounterType,
    pub from_session_id: u32,
}

pub struct NetworkMap<T, const NETWORK_MAP_SIZE: usize> {
    network_map: [Option<NetworkMapEntry<T>>; NETWORK_MAP_SIZE],
    host_addr: T,
    host_ip: Option<[u8; 4]>,
    upstream_local_ports: [u16; MAX_UPSTREAM_LOCAL_PORTS],
    num_upstream_local_ports: usize,
}

impl<T, const NETWORK_MAP_SIZE: usize> NetworkMap<T, NETWORK_MAP_SIZE>
where
    T: Copy + PartialEq + Eq + core::fmt::Debug,
{
    pub fn new(host_addr: T) -> Self {
        Self {
            network_map: [None; NETWORK_MAP_SIZE],
            host_addr,
            host_ip: None,
            upstream_local_ports: [0; MAX_UPSTREAM_LOCAL_PORTS],
            num_upstream_local_ports: 0,
        }
    }

    pub fn map_network_address(
        &mut self,
        from_address: T,
        ip: [u8; 4],
        port: u16,
        interface_index: u8,
        update: bool,
    ) -> Result<&mut NetworkMapEntry<T>, BigBrotherError> {
        for (i, mapping) in self.network_map.iter_mut().enumerate() {
            match mapping {
                Some(mapping) => {
                    if mapping.network_address == from_address {
                        if update {
                            if mapping.ip != ip
                                || mapping.port != port
                                || mapping.interface_index != interface_index
                            {
                                // println!("{:?}: Remapped {:?} to {:?}:{} (i{} -> i{})", self.host_addr, from_address, ip, port, mapping.interface_index, interface_index);

                                if from_address == self.host_addr {
                                    self.host_ip = Some(ip);
                                }
                            }

                            // print!("bc {} -> ", mapping.broadcast_counter);

                            *mapping = NetworkMapEntry {
                                network_address: from_address,
                                ip,
                                port,
                                interface_index,
                                to_counter: mapping.to_counter,
                                from_counter: mapping.from_counter,
                                broadcast_counter: mapping.broadcast_counter,
                                from_session_id: mapping.from_session_id,
                            };

                            // print!("{} (i{})", mapping.broadcast_counter, interface_index);
                        }

                        // println!(" (existing {:?} mapping at {} - {})", from_address, i, mapping.broadcast_counter);

                        return Ok(mapping);
                    }
                }
                None => {
                    *mapping = Some(NetworkMapEntry {
                        network_address: from_address,
                        ip,
                        port,
                        interface_index,
                        to_counter: 0,
                        from_counter: 0,
                        broadcast_counter: 0,
                        from_session_id: 0,
                    });

                    if from_address == self.host_addr {
                        self.host_ip = Some(ip);
                    } else if let Some(host_ip) = self.host_ip {
                        if host_ip == ip {
                            self.upstream_local_ports[self.num_upstream_local_ports] = port;
                            self.num_upstream_local_ports += 1;

                            // println!("{:?}: Upstream chain detected for {:?} ({:?}:{})", self.host_addr, from_address, ip, port);
                        }
                    }

                    // println!("{:?}: Mapped {:?} to {:?}:{} @i{} (index {})", self.host_addr, from_address, ip, port, interface_index, i);
                    // defmt::info!("Mapped new address from {}:{}", ip, port);

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
        broadcast_counter: Option<dedupe::CounterType>,
    ) -> Result<(), BigBrotherError> {
        let mapping = self.get_address_mapping(address)?;

        if session_id != mapping.from_session_id {
            mapping.from_counter = 0;
            mapping.broadcast_counter = broadcast_counter.unwrap_or(0);
            mapping.from_session_id = session_id;
        }

        Ok(())
    }

    pub fn get_upstream_local_ports(&self) -> &[u16] {
        &self.upstream_local_ports[..self.num_upstream_local_ports]
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
        let mut network_map =
            NetworkMap::<TestNetworkAddress, 32>::new(TestNetworkAddress::FlightController);

        let mut i = 0;
        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            network_map
                .map_network_address(
                    *address,
                    [123 + i, 0 + i, 200 + i, 42 + i],
                    UDP_PORT,
                    i % 2,
                    true,
                )
                .unwrap();
            i += 1;
        }

        i = 0;
        for address in &NETWORK_ADDRESS_TEST_DEFAULTS {
            let mapping = network_map.get_address_mapping(*address).unwrap();
            // println!("Testing address: {:?} with mapping: {:?}", address, mapping);

            assert_eq!(mapping.network_address, *address);
            assert_eq!(mapping.ip, [123 + i, 0 + i, 200 + i, 42 + i]);
            assert_eq!(mapping.interface_index, i % 2);
            i += 1;
        }
    }
}
