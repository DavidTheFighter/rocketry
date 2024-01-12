use std::{sync::{mpsc, Mutex, Arc}, collections::{HashMap, VecDeque}};

use crate::big_brother::{BigBrotherEndpoint, UDP_PORT};

use super::mock_interface::MockPayload;

#[derive(Debug, Clone)]
pub struct MockPhysicalNet {
    interface_map: HashMap<[u8; 4], mpsc::Sender<MockPayload>>,
    subnet_ip: [u8; 4],
    subnet_mask: [bool; 4],
    broadcast_ip: [u8; 4],
    packet_log: Option<Vec<MockPayload>>,
}

impl MockPhysicalNet {
    pub fn new(subnet_ip: [u8; 4], subnet_mask: [bool; 4], broadcast_ip: [u8; 4]) -> Self {
        // println!("New physical net on {:?} w/ mask {:?} w/ broadcast {:?}", subnet_ip, subnet_mask, broadcast_ip);
        Self {
            interface_map: HashMap::new(),
            subnet_ip,
            subnet_mask,
            broadcast_ip,
            packet_log: None,
        }
    }

    pub fn send_udp(&mut self, payload: MockPayload) {
        // print!("{:?}:{} -> {} bytes to ", payload.remote.ip, payload.remote.port, payload.data.len());

        if let Some(log) = &mut self.packet_log {
            log.push(payload.clone());
        }

        if payload.host.ip == self.broadcast_ip {
            // println!("port, {} broadcasted to {} interfaces", payload.host.port, self.interface_map.len());

            for (_ip, tx) in self.interface_map.iter_mut() {
                tx.send(payload.clone()).expect("Failed to broadcast UDP payload over TX");
            }
        } else {
            if let Some(tx) = self.interface_map.get(&payload.host.ip) {
                // println!("{:?}:{}", payload.host.ip, payload.host.port);

                tx.send(payload).expect("Failed to send UDP payload over TX");
            } else if payload.host.ip == [127, 0, 0, 1] {
                // println!("localhost:{} / {:?}:{}", payload.host.port, payload.remote.ip, payload.host.port);

                if let Some(tx) = self.interface_map.get(&payload.remote.ip) {
                    tx.send(payload).expect("Failed to send UDP payload over TX");
                } else {
                    eprintln!("Destination for UDP payload does not exist! {:?}", payload);
                }
            } else {
                eprintln!("Destination for UDP payload does not exist! {:?}", payload);
            }
        }
    }

    pub fn register_physical_interface(
        &mut self,
        tx: mpsc::Sender<MockPayload>,
    ) -> [u8; 4] {
        let mut attempts = 0;
        let mut ip;

        loop {
            if attempts > 65536 {
                panic!("Unable to allocate IP for physical interface!");
            }

            ip = self.generate_random_ip();
            if !self.interface_map.contains_key(&ip) {
                break;
            }

            attempts += 1;
        }

        self.interface_map.insert(ip, tx);

        ip
    }

    pub fn broadcast_ip(&self) -> [u8; 4] {
        self.broadcast_ip
    }

    pub fn enable_payload_logging(&mut self) {
        if self.packet_log.is_none() {
            self.packet_log = Some(Vec::new());
        }
    }

    pub fn take_payload_log(&mut self) -> Vec<MockPayload> {
        if self.packet_log.is_none() {
            Vec::new()
        } else {
            let log = self.packet_log.take().unwrap();
            self.packet_log = Some(Vec::new());

            log
        }
    }

    fn generate_random_ip(&self) -> [u8; 4] {
        let mut ip = self.subnet_ip;

        for i in 0..4 {
            if !self.subnet_mask[i] {
                ip[i] = rand_u8().min(254);
            }
        }

        ip
    }
}

fn rand_u8() -> u8 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    (nanos % 255) as u8
}

pub struct MockPhysicalInterface {
    pub host_ip: [u8; 4],
    pub num_virtual_interfaces: usize,
    network: Arc<Mutex<MockPhysicalNet>>,
    interface_rx: mpsc::Receiver<MockPayload>,
    virtual_rx_queue: HashMap<u16, VecDeque<MockPayload>>,
}

impl MockPhysicalInterface {
    pub fn new(network: Arc<Mutex<MockPhysicalNet>>) -> Self {
        let (tx, rx) = mpsc::channel();

        let host_ip = network
            .lock()
            .expect("Failed to unlock MockPhysicalNet for physical interface init")
            .register_physical_interface(tx);

        // println!("New phy iface @ {:?}", host_ip);

        Self {
            host_ip,
            num_virtual_interfaces: 0,
            network,
            interface_rx: rx,
            virtual_rx_queue: HashMap::new(),
        }
    }

    pub fn send_udp(&mut self, payload: MockPayload) {
        self.network
            .lock()
            .expect("Failed to lock physical network for send UDP")
            .send_udp(payload);
    }

    pub fn recv_udp(&mut self, port: u16) -> Option<MockPayload> {
        // print!("{:?}:{} <- ", self.host_ip, port);

        if let Some(payload) = self.virtual_rx_queue.get_mut(&port).unwrap().pop_front() {
            // println!("{} bytes from {:?}:{} via queue", payload.data.len(), payload.remote.ip, payload.remote.port);
            return Some(payload);
        }

        loop {
            if let Ok(payload) = self.interface_rx.try_recv() {
                if payload.host.port == port {
                    // println!("{} bytes from {:?}:{} via mpsc", payload.data.len(), payload.remote.ip, payload.remote.port);
                    return Some(payload);
                } else {
                    // print!(". ");
                    self.virtual_rx_queue.get_mut(&payload.host.port).unwrap().push_back(payload);
                }
            } else {
                // println!(" 0 bytes (buffers empty)");
                return None;
            }
        }
    }

    pub fn register_virtual_interface(&mut self) -> BigBrotherEndpoint {
        let host = BigBrotherEndpoint {
            ip: self.host_ip,
            port: UDP_PORT + self.num_virtual_interfaces as u16,
        };

        self.num_virtual_interfaces += 1;
        self.virtual_rx_queue.insert(host.port, VecDeque::new());

        // println!("New virtual iface @ {:?}:{}", host.ip, host.port);

        host
    }

    pub fn broadcast_ip(&self) -> [u8; 4] {
        self.network
            .lock()
            .expect("Failed to lock physical network to get broadcast ip")
            .broadcast_ip()
    }
}
