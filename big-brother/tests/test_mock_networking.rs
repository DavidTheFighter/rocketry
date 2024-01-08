use std::sync::{Arc, Mutex};

use big_brother::{interface::{mock_topology::{MockPhysicalNet, MockPhysicalInterface}, mock_interface::MockInterface, BigBrotherInterface}, big_brother::{BigBrotherEndpoint, WORKING_BUFFER_SIZE, UDP_PORT}};

fn rand_u32() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

/// Basic test to ensure I can put 2 interfaces on a net and they can talk
#[test]
fn send_recv() {
    let network = fixture_network();
    let mut iface0 = fixture_iface_singleton(network.clone());
    let mut iface1 = fixture_iface_singleton(network.clone());

    assert_eq!(iface0.host_port, UDP_PORT);
    assert_eq!(iface1.host_port, UDP_PORT);

    let mut dummy_data0 = [1, 2, 3, 4, 255, 254, 253];
    let mut dummy_data1 = [255, 0, 254, 1, 128, 16, 175];
    let mut recv_buffer = [0, 0, 0, 0, 0, 0, 0];

    for i in 0..128 {
        println!("-- Attempt {} --", i);

        assert!(iface0.recv_udp(&mut recv_buffer).unwrap().is_none());
        assert!(iface1.recv_udp(&mut recv_buffer).unwrap().is_none());

        iface0.send_udp(BigBrotherEndpoint {
            ip: iface1.host_ip,
            port: iface1.host_port,
        }, &mut dummy_data0).unwrap();

        let (size, remote) = iface1.recv_udp(&mut recv_buffer).unwrap().unwrap();
        assert_eq!(size, dummy_data0.len());
        assert_eq!(recv_buffer, dummy_data0);
        assert_eq!(remote.ip, iface0.host_ip);
        assert_eq!(remote.port, iface0.host_port);
        assert!(iface0.recv_udp(&mut recv_buffer).unwrap().is_none());
        assert!(iface1.recv_udp(&mut recv_buffer).unwrap().is_none());

        iface1.send_udp(BigBrotherEndpoint {
            ip: iface0.host_ip,
            port: iface0.host_port,
        }, &mut dummy_data1).unwrap();

        let (size, remote) = iface0.recv_udp(&mut recv_buffer).unwrap().unwrap();
        assert_eq!(size, dummy_data1.len());
        assert_eq!(recv_buffer, dummy_data1);
        assert_eq!(remote.ip, iface1.host_ip);
        assert_eq!(remote.port, iface1.host_port);
        assert!(iface0.recv_udp(&mut recv_buffer).unwrap().is_none());
        assert!(iface1.recv_udp(&mut recv_buffer).unwrap().is_none());

        for byte in dummy_data0.iter_mut() {
            *byte = (rand_u32() % 255) as u8;
        }

        for byte in dummy_data1.iter_mut() {
            *byte = (rand_u32() % 255) as u8;
        }
    }
}

/// Ensures that broadcasts properly go to all interfaces (including loopback)
#[test]
fn broadcast() {
    let network = fixture_network();
    let iface0 = fixture_iface_singleton(network.clone());
    let iface1 = fixture_iface_singleton(network.clone());
    let iface2 = fixture_iface_singleton(network.clone());

    let broadcast_ip = network.lock().unwrap().broadcast_ip();
    let mut dummy_data0 = [1, 2, 3, 4, 255, 254, 253, 5, 28, 19, 23];
    let mut recv_buffer = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let ifaces = &mut [iface0, iface1, iface2];

    for i in 0..128 {
        println!("-- Attempt {} --", i);

        assert_empty_recv(ifaces);

        let broadcaster_ip = ifaces[i % 3].host_ip;

        ifaces[i % 3].send_udp(BigBrotherEndpoint {
            ip: broadcast_ip,
            port: UDP_PORT,
        }, &mut dummy_data0).unwrap();

        for iface in ifaces.iter_mut() {
            let (size, remote) = iface.recv_udp(&mut recv_buffer).unwrap().unwrap();
            assert_eq!(size, dummy_data0.len());
            assert_eq!(recv_buffer, dummy_data0);
            assert_eq!(remote.ip, broadcaster_ip);
            assert_eq!(remote.port, UDP_PORT);
        }

        for byte in dummy_data0.iter_mut() {
            *byte = (rand_u32() % 255) as u8;
        }

        assert_empty_recv(ifaces);
    }
}

/// Ensures that a network with many phy's routes packets correctly
#[test]
fn many_phy_routing() {
    let network = fixture_network();

    let mut dummy_data0 = [1, 2, 3, 4, 255, 254, 253, 5, 28, 19, 23];
    let mut recv_buffer = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut ifaces = Vec::new();
    for _ in 0..16 {
        ifaces.push(fixture_iface_singleton(network.clone()));
    }

    for i in 0..512 {
        println!("-- Attempt {} --", i);

        assert_empty_recv(ifaces.as_mut_slice());

        let sender_index = i % ifaces.len();
        let sender_ip = ifaces[sender_index].host_ip;
        let mut receiver_index = (rand_u32() as usize) % ifaces.len();
        if receiver_index == sender_index {
            receiver_index = (receiver_index + 1) % ifaces.len();
        }
        let destination_ip = ifaces[receiver_index].host_ip;

        ifaces[sender_index].send_udp(BigBrotherEndpoint {
            ip: destination_ip,
            port: UDP_PORT,
        }, &mut dummy_data0).unwrap();

        let (size, remote) = ifaces[receiver_index].recv_udp(&mut recv_buffer).unwrap().unwrap();
        assert_eq!(size, dummy_data0.len());
        assert_eq!(recv_buffer, dummy_data0);
        assert_eq!(remote.ip, sender_ip);
        assert_eq!(remote.port, UDP_PORT);

        for byte in dummy_data0.iter_mut() {
            *byte = (rand_u32() % 255) as u8;
        }

        assert_empty_recv(ifaces.as_mut_slice());
    }
}

/// Ensures that sending to different ports actually works
#[test]
fn many_ports_routing() {
    let network = fixture_network();

    let mut dummy_data0 = [1, 2, 3, 4, 255, 254, 253, 5, 28, 19, 23];
    let mut recv_buffer = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let phy_ifaces = [fixture_phy(network.clone()), fixture_phy(network.clone())];
    let mut virt_ifaces = Vec::new();

    for i in 0..4 {
        virt_ifaces.push(MockInterface::new_networked(phy_ifaces[i % 2].clone()));
    }

    for i in 0..512 {
        assert_empty_recv(virt_ifaces.as_mut_slice());

        let sender_index = i % virt_ifaces.len();
        let sender_ip = virt_ifaces[sender_index].host_ip;
        let sender_port = virt_ifaces[sender_index].host_port;

        let destination_ip = virt_ifaces[(i + 1) % virt_ifaces.len()].host_ip;
        let destination_port = UDP_PORT + ((rand_u32() % 2) as u16);

        println!("-- Attempt {} :: {:?}:{} -> {:?}:{}", i, sender_ip, sender_port, destination_ip, destination_port);

        virt_ifaces[sender_index].send_udp(BigBrotherEndpoint {
            ip: destination_ip,
            port: destination_port,
        }, &mut dummy_data0).unwrap();

        for iface in virt_ifaces.iter_mut() {
            if iface.host_ip == sender_ip {
                continue;
            } else if iface.host_port == destination_port {
                let (size, remote) = iface.recv_udp(&mut recv_buffer).unwrap().unwrap();
                assert_eq!(size, dummy_data0.len());
                assert_eq!(recv_buffer, dummy_data0);
                assert_eq!(remote.ip, sender_ip);
                assert_eq!(remote.port, sender_port);
            } else {
                assert!(iface.recv_udp(&mut recv_buffer).unwrap().is_none());
            }
        }

        for byte in dummy_data0.iter_mut() {
            *byte = (rand_u32() % 255) as u8;
        }

        assert_empty_recv(virt_ifaces.as_mut_slice());
    }
}

fn fixture_network() -> Arc<Mutex<MockPhysicalNet>> {
    Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 0, 0],
        [true, true, false, false],
        [192, 168, 255, 255],
    )))
}

fn fixture_phy(network: Arc<Mutex<MockPhysicalNet>>) -> Arc<Mutex<MockPhysicalInterface>> {
    Arc::new(Mutex::new(MockPhysicalInterface::new(network)))
}

fn fixture_iface_singleton(network: Arc<Mutex<MockPhysicalNet>>) -> MockInterface {
    MockInterface::new_networked(fixture_phy(network))
}

fn assert_empty_recv(ifaces: &mut [MockInterface]) {
    let mut buffer = [0; WORKING_BUFFER_SIZE];
    for iface in ifaces {
        assert!(iface.recv_udp(&mut buffer).unwrap().is_none())
    }
}

