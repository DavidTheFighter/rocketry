use hal::comms_hal::NetworkAddress;

use crate::timestamp;


pub struct CameraConnection {
    pub address: NetworkAddress,
    pub last_ping: f64,
}

impl CameraConnection {
    pub fn new(address: NetworkAddress) -> Self {
        Self {
            address,
            last_ping: timestamp(),
        }
    }

    pub fn ping(&mut self) {
        self.last_ping = timestamp();
    }

    pub fn drop_connection(&mut self) {
        
    }
}