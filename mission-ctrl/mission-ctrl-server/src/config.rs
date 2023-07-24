use hal::{
    comms_hal::{Packet, NetworkAddress},
    ecu_hal::{EcuSensor, IgniterConfig},
    SensorConfig,
};
use rocket::serde::Deserialize;
use std::{sync::Arc, time::Duration};
use strum::IntoEnumIterator;

use crate::{observer::{ObserverHandler, ObserverEvent}, process_is_running};

struct ConfigHandler {
    observer_handler: Arc<ObserverHandler>
}

impl ConfigHandler {
    fn new(observer_handler: Arc<ObserverHandler>) -> Self {
        Self {
            observer_handler,
        }
    }

    fn run(&mut self) {
        self.observer_handler.register_observer_thread();
        self.send_ecu_config(NetworkAddress::EngineController(0));

        while process_is_running() {
            if let Some(address) = self.get_device_booted() {
                match address {
                    NetworkAddress::EngineController(_) => {
                        self.send_ecu_config(address);
                    },
                    _ => {}
                }
            }
        }
    }

    fn send_ecu_config(&self, address: NetworkAddress) {
        for packet in read_hardware_config() {
            self.observer_handler.notify(ObserverEvent::SendPacket { address, packet });
        }
    }

    fn get_device_booted(&self) -> Option<NetworkAddress> {
        if let Some((_, event)) = self.observer_handler.wait_event(Duration::from_millis(100)) {
            if let ObserverEvent::PacketReceived { address, packet } = event {
                if let Packet::DeviceBooted = packet {
                    return Some(address);
                }
            }
        }

        None
    }
}

pub fn config_thread(observer_handler: Arc<ObserverHandler>) {
    ConfigHandler::new(observer_handler).run();
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct HardwareConfigSensorMap {
    index: usize,
    config: SensorConfig,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct HardwareConfig {
    sensor_mappings: Vec<HardwareConfigSensorMap>,
    igniter_config: IgniterConfig,
}

fn read_hardware_config() -> Vec<Packet> {
    let hardware_config = std::fs::read_to_string("hardware.json")
        .expect("Couldn't load hardware config file!");
    let config: HardwareConfig = rocket::serde::json::from_str(&hardware_config).unwrap();
    let mut config_packets = Vec::new();

    for sensor_config in config.sensor_mappings.iter() {
        config_packets.push(Packet::ConfigureSensor {
            sensor: EcuSensor::iter().nth(sensor_config.index).unwrap(),
            config: sensor_config.config,
        });
    }

    config_packets.push(Packet::ConfigureIgniter(config.igniter_config));

    config_packets
}