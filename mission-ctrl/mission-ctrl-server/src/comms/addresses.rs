use hal::comms_hal::NetworkAddress;

pub fn network_address_to_ip(address: NetworkAddress) -> String {
    match address {
        NetworkAddress::EngineController(0) => String::from("169.254.0.6"),
        NetworkAddress::FlightController => String::from("169.254.0.7"),
        _ => String::from(""),
    }
}

pub fn ip_to_network_address(ip: String) -> NetworkAddress {
    match ip.as_str() {
        "169.254.0.6" => NetworkAddress::EngineController(0),
        "169.254.0.7" => NetworkAddress::FlightController,
        _ => NetworkAddress::Broadcast,
    }
}