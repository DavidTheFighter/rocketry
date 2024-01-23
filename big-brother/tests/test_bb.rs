use std::sync::{Arc, Mutex};

use big_brother::{
    big_brother::{BigBrotherError, Broadcastable, MAX_INTERFACE_COUNT},
    interface::{
        mock_interface::MockInterface,
        mock_topology::{MockPhysicalInterface, MockPhysicalNet},
        BigBrotherInterface,
    },
    BigBrother,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestPacket {
    Heartbeat,
    SomeData { a: u32, b: u32, c: bool },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestNetworkAddress {
    Broadcast,
    A,
    B,
    C,
    D,
}

const TEST_ADDRESSES: [TestNetworkAddress; 4] = [
    TestNetworkAddress::A,
    TestNetworkAddress::B,
    TestNetworkAddress::C,
    TestNetworkAddress::D,
];

fn rand_u32() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

#[test]
fn broadcast() {
    let network = fixture_network();
    let mut interfaces = Vec::new();
    let mut bbs = Vec::new();

    for _ in 0..4 {
        interfaces.push(fixture_iface_singleton(network.clone()));
    }

    for (index, iface) in interfaces.iter_mut().enumerate() {
        bbs.push(fixture_bb([Some(iface), None], TEST_ADDRESSES[index]));
    }

    assert_empty_recv(&mut bbs);

    for i in 0..128 {
        // println!("\nATTEMPT {}", i);
        let packet = TestPacket::SomeData {
            a: i,
            b: i * 2,
            c: i % 2 == 0,
        };

        let bbi = (i as usize) % bbs.len();
        bbs[bbi]
            .send_packet(&packet, TestNetworkAddress::Broadcast)
            .unwrap();

        for j in 0..bbs.len() {
            let (recv_packet, remote) = bbs[j].recv_packet().unwrap().unwrap();
            assert_eq!(recv_packet, packet);
            assert_eq!(remote, TEST_ADDRESSES[bbi]);
        }
    }
}

#[test]
fn bb_many_bb_routing() {
    let network = fixture_network();
    let mut interfaces = Vec::new();
    let mut bbs = Vec::new();

    for _ in 0..4 {
        interfaces.push(fixture_iface_singleton(network.clone()));
    }

    for (index, iface) in interfaces.iter_mut().enumerate() {
        bbs.push(fixture_bb([Some(iface), None], TEST_ADDRESSES[index]));
    }

    // Ensures that all bbs have each other network mapped
    for bb in bbs.iter_mut() {
        bb.poll_1ms(0);
    }

    for i in 0..1024 {
        assert_empty_recv(bbs.as_mut_slice());

        let packet = TestPacket::SomeData {
            a: i,
            b: i * 2,
            c: i % 2 == 0,
        };

        let sender_index = (i as usize) % bbs.len();
        let mut dest_index = (rand_u32() as usize) % bbs.len();
        while dest_index == sender_index {
            dest_index = (rand_u32() as usize) % bbs.len();
        }

        bbs[sender_index]
            .send_packet(&packet, TEST_ADDRESSES[dest_index])
            .unwrap();

        let (recv_packet, remote) = bbs[dest_index].recv_packet().unwrap().unwrap();
        assert_eq!(recv_packet, packet);
        assert_eq!(remote, TEST_ADDRESSES[sender_index]);

        assert_empty_recv(bbs.as_mut_slice());
    }
}

#[test]
fn bb_no_recv_incorrect_destination() {
    let network = fixture_network();
    let mut iface0 = fixture_iface_singleton(network.clone());
    let mut iface1 = fixture_iface_singleton(network.clone());

    let mut bb0 = fixture_bb([Some(&mut iface0), None], TestNetworkAddress::A);
    let mut bb1 = fixture_bb([Some(&mut iface1), None], TestNetworkAddress::B);

    bb0.poll_1ms(0);
    bb1.poll_1ms(0);

    let packet = TestPacket::SomeData {
        a: 0,
        b: 1,
        c: true,
    };

    assert!(matches!(
        bb0.send_packet(&packet, TestNetworkAddress::C),
        Err(BigBrotherError::UnknownNetworkAddress)
    ));
    assert!(matches!(
        bb1.send_packet(&packet, TestNetworkAddress::C),
        Err(BigBrotherError::UnknownNetworkAddress)
    ));
    assert!(matches!(
        bb1.send_packet(&packet, TestNetworkAddress::D),
        Err(BigBrotherError::UnknownNetworkAddress)
    ));
    assert!(matches!(
        bb1.send_packet(&packet, TestNetworkAddress::D),
        Err(BigBrotherError::UnknownNetworkAddress)
    ));

    assert_empty_recv(&mut [bb0, bb1]);
}

// Tests that with 2 networks and one bridged connection, both networks operate normally
// and independently of each other. This does NOT test forwarding
#[test]
fn bb_multi_network() {
    let network0 = Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 0, 0],
        [true, true, true, false],
        [192, 168, 0, 255],
    )));
    let network1 = Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 1, 0],
        [true, true, true, false],
        [192, 168, 1, 255],
    )));
    let mut iface0 = fixture_iface_singleton(network0.clone());
    let mut iface1 = fixture_iface_singleton(network1.clone());
    let iface_bridge_phy0 = fixture_phy(network0.clone());
    let iface_bridge_phy1 = fixture_phy(network1.clone());
    let mut iface_bridge0 = MockInterface::new_networked(iface_bridge_phy0.clone());
    let mut iface_bridge1 = MockInterface::new_networked(iface_bridge_phy1.clone());

    let bb0 = fixture_bb([Some(&mut iface0), None], TestNetworkAddress::A);
    let bb1 = fixture_bb([Some(&mut iface1), None], TestNetworkAddress::B);
    let bb_bridge = fixture_bb(
        [Some(&mut iface_bridge0), Some(&mut iface_bridge1)],
        TestNetworkAddress::C,
    );
    let mut bbs = [bb0, bb1, bb_bridge];

    bbs[0].poll_1ms(0);
    bbs[1].poll_1ms(0);
    bbs[2].poll_1ms(0);

    assert_empty_recv(&mut bbs);

    let mut packet = TestPacket::SomeData {
        a: 0,
        b: 1,
        c: true,
    };

    bbs[0].send_packet(&packet, TestNetworkAddress::C).unwrap();
    let (recv_packet, remote) = bbs[2].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::A);
    assert_empty_recv(&mut bbs);

    packet = TestPacket::SomeData {
        a: 1,
        b: 2,
        c: false,
    };

    bbs[1].send_packet(&packet, TestNetworkAddress::C).unwrap();

    let (recv_packet, remote) = bbs[2].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::B);

    assert_empty_recv(&mut bbs);

    packet = TestPacket::SomeData {
        a: 2,
        b: 3,
        c: true,
    };

    bbs[2].send_packet(&packet, TestNetworkAddress::A).unwrap();
    bbs[2].send_packet(&packet, TestNetworkAddress::B).unwrap();

    let (recv_packet, remote) = bbs[0].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::C);

    let (recv_packet, remote) = bbs[1].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::C);
}

#[test]
fn simple_two_network_forwarding() {
    let network0 = Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 0, 0],
        [true, true, true, false],
        [192, 168, 255, 255],
    )));
    let network1 = Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 1, 0],
        [true, true, true, false],
        [192, 168, 255, 255],
    )));
    let mut iface0 = fixture_iface_singleton(network0.clone());
    let mut iface1 = fixture_iface_singleton(network1.clone());
    let iface_bridge_phy0 = fixture_phy(network0.clone());
    let iface_bridge_phy1 = fixture_phy(network1.clone());
    let mut iface_bridge0 = MockInterface::new_networked(iface_bridge_phy0.clone());
    let mut iface_bridge1 = MockInterface::new_networked(iface_bridge_phy1.clone());

    // A <-> C <-> B
    let bb0 = fixture_bb([Some(&mut iface0), None], TestNetworkAddress::A);
    let bb1 = fixture_bb([Some(&mut iface1), None], TestNetworkAddress::B);
    let bb_bridge = fixture_bb(
        [Some(&mut iface_bridge0), Some(&mut iface_bridge1)],
        TestNetworkAddress::C,
    );
    let mut bbs = [bb0, bb1, bb_bridge];

    // println!("Recving bbs");
    assert_empty_recv(&mut bbs);
    for bb in &mut bbs {
        bb.poll_1ms(101);
    }
    // println!("Recving bbs");
    assert_empty_recv(&mut bbs);

    let mut packet = TestPacket::SomeData {
        a: 0,
        b: 1,
        c: true,
    };

    // Send to other network
    // println!("Sending packet");
    bbs[0].send_packet(&packet, TestNetworkAddress::B).unwrap();

    // Forwarding (as of now) takes place when recv_packet() is called so ensure it doesn't get there early
    assert!(matches!(bbs[1].recv_packet().unwrap(), None));
    // Call recv on the forwarding iface and make sure it doesn't return that packet
    assert!(matches!(bbs[2].recv_packet().unwrap(), None));

    // Test the packet made it to the other network
    let (recv_packet, remote) = bbs[1].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::A);
    assert_empty_recv(&mut bbs);

    // println!("Testing broadcast forwarding");

    bbs[1]
        .send_packet(&packet, TestNetworkAddress::Broadcast)
        .unwrap();

    // Test loopback
    let (recv_packet, remote) = bbs[1].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::B);

    // Test the bridge bb to ensure it forwards
    let (recv_packet, remote) = bbs[2].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::B);

    // Test the last bb on the other network
    let (recv_packet, remote) = bbs[0].recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::B);

    assert_empty_recv(&mut bbs);
}

// TODO fn multi_network_new_session()

#[test]
fn simple_local_forwarding() {
    let network = fixture_network();
    let phy0 = fixture_phy(network.clone());
    let mut iface_sep = fixture_iface_singleton(network.clone());
    let mut iface_host = MockInterface::new_networked(phy0.clone());
    let mut iface_chained = MockInterface::new_networked(phy0.clone());

    // Ensure the network was set up properly
    assert!(iface_sep.host_ip != iface_host.host_ip);
    assert!(iface_sep.host_ip != iface_chained.host_ip);
    assert!(iface_host.host_ip == iface_chained.host_ip);
    assert!(iface_chained.host_port == iface_host.host_port + 1);

    let mut bb_sep = fixture_bb([Some(&mut iface_sep), None], TestNetworkAddress::A);
    let mut bb_host = fixture_bb([Some(&mut iface_host), None], TestNetworkAddress::B);
    let mut bb_chained = fixture_bb([Some(&mut iface_chained), None], TestNetworkAddress::C);

    assert_empty_recv_slice(&mut [&mut bb_sep, &mut bb_host, &mut bb_chained]);

    // println!("Polling");
    for bb in &mut [&mut bb_sep, &mut bb_host, &mut bb_chained] {
        bb.poll_1ms(101);
    }
    // println!("Recving bbs");
    assert_empty_recv_slice(&mut [&mut bb_sep, &mut bb_host, &mut bb_chained]);

    let packet = TestPacket::SomeData {
        a: 0,
        b: 1,
        c: true,
    };

    // Test that packets aren't forwarded when they're not supposed to be
    // println!("Testing no forwarding");
    bb_sep.send_packet(&packet, TestNetworkAddress::B).unwrap();
    let (recv_packet, remote) = bb_host.recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::A);
    assert_empty_recv_slice(&mut [&mut bb_sep, &mut bb_host, &mut bb_chained]);

    // Test a packet that should be forwarded down
    // println!("Testing forwarding down");
    bb_chained
        .send_packet(&packet, TestNetworkAddress::A)
        .unwrap();
    assert!(bb_host.recv_packet().unwrap().is_none()); // Runs forwarding on the host bb
    let (recv_packet, remote) = bb_sep.recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::C);
    assert_empty_recv_slice(&mut [&mut bb_sep, &mut bb_host, &mut bb_chained]);

    // Test a packet that should be forwarded up
    // println!("Testing forwarding up");
    bb_sep.send_packet(&packet, TestNetworkAddress::C).unwrap();
    assert!(bb_host.recv_packet().unwrap().is_none()); // Runs forwarding on the host bb
    let (recv_packet, remote) = bb_chained.recv_packet().unwrap().unwrap();
    assert_eq!(recv_packet, packet);
    assert_eq!(remote, TestNetworkAddress::A);
    assert_empty_recv_slice(&mut [&mut bb_sep, &mut bb_host, &mut bb_chained]);
}

fn assert_empty_recv<'a, const N: usize>(
    bbs: &mut [BigBrother<'a, N, TestPacket, TestNetworkAddress>],
) {
    for bb in bbs {
        assert!(bb.recv_packet().unwrap().is_none())
    }
}

fn assert_empty_recv_slice<'a, const N: usize>(
    bbs: &mut [&mut BigBrother<'a, N, TestPacket, TestNetworkAddress>],
) {
    for bb in bbs {
        assert!(bb.recv_packet().unwrap().is_none())
    }
}

fn fixture_bb<'a>(
    interfaces: [Option<&'a mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT],
    address: TestNetworkAddress,
) -> BigBrother<'a, 32, TestPacket, TestNetworkAddress> {
    let bb = BigBrother::new(
        address,
        rand_u32(),
        TestNetworkAddress::Broadcast,
        interfaces,
    );

    bb
}

fn fixture_iface_singleton(network: Arc<Mutex<MockPhysicalNet>>) -> MockInterface {
    MockInterface::new_networked(fixture_phy(network))
}

fn fixture_phy(network: Arc<Mutex<MockPhysicalNet>>) -> Arc<Mutex<MockPhysicalInterface>> {
    Arc::new(Mutex::new(MockPhysicalInterface::new(network)))
}

fn fixture_network() -> Arc<Mutex<MockPhysicalNet>> {
    Arc::new(Mutex::new(MockPhysicalNet::new(
        [192, 168, 0, 0],
        [true, true, false, false],
        [192, 168, 255, 255],
    )))
}

impl Broadcastable for TestNetworkAddress {
    fn is_broadcast(&self) -> bool {
        matches!(self, TestNetworkAddress::Broadcast)
    }
}
