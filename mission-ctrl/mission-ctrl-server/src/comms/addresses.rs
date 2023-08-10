use std::{sync::RwLock, collections::HashMap, net::{Ipv4Addr, IpAddr}};

use hal::comms_hal::NetworkAddress;

// pub fn network_address_to_ip(address: NetworkAddress) -> String {
//     match address {
//         NetworkAddress::EngineController(0) => String::from("169.254.0.6"),
//         NetworkAddress::FlightController => String::from("169.254.0.7"),
//         NetworkAddress::GroundCamera(0) => String::from("192.168.1.88"),
//         _ => String::from(""),
//     }
// }

// pub fn ip_to_network_address(ip: String) -> NetworkAddress {
//     match ip.as_str() {
//         "169.254.0.6" => NetworkAddress::EngineController(0),
//         "169.254.0.7" => NetworkAddress::FlightController,
//         "192.168.1.88" => NetworkAddress::GroundCamera(0),
//         _ => NetworkAddress::Broadcast,
//     }
// }

pub struct AddressManager {
    address_map: RwLock<HashMap<NetworkAddress, Ipv4Addr>>,
}

impl AddressManager {
    pub fn new(_defaults_file: String) -> Self {
        let mut default_map = HashMap::new();
        default_map.insert(NetworkAddress::FlightController, Ipv4Addr::new(169, 254, 0, 7));
        default_map.insert(NetworkAddress::EngineController(0), Ipv4Addr::new(169, 254, 0, 6));

        Self {
            address_map: RwLock::new(default_map),
        }
    }

    pub fn network_address_to_ip(&self, address: NetworkAddress) -> Option<Ipv4Addr> {
        let address_map = self.address_map.read().unwrap();
        address_map.get(&address).copied()
    }

    pub fn ip_to_network_address(&self, ip: IpAddr) -> Option<NetworkAddress> {
        match ip {
            IpAddr::V4(ip) => self.ipv4_to_network_address(ip),
            IpAddr::V6(_) => None,
        }
    }

    pub fn ipv4_to_network_address(&self, ip: Ipv4Addr) -> Option<NetworkAddress> {
        let address_map = self.address_map.read().unwrap();

        address_map.iter().find(|(_, &v)| v == ip).map(|(&k, _)| k)
    }

    pub fn map_ip_address(&self, address: NetworkAddress, ip: Ipv4Addr) {
        let mut address_map = self.address_map.write().unwrap();
        address_map.insert(address, ip);
    }
}