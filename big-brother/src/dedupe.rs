use crate::{big_brother::Broadcastable, network_map::NetworkMapEntry, serdes::PacketMetadata};

pub type CounterType = u32;

/// Returns Ok() if not a duplicate w/ how many missed packets, Err() if duplicate
pub fn is_duplicate<A>(
    metadata: &PacketMetadata<A>,
    mapping: &mut NetworkMapEntry<A>,
) -> Result<usize, ()>
where
    A: Broadcastable + PartialEq + Eq + core::fmt::Debug,
{
    let compare_counter = if metadata.to_addr.is_broadcast() {
        mapping.broadcast_counter
    } else {
        mapping.from_counter
    };

    // This does a wrapped sub of new - old, it tells us how much newer the new counter is. Old
    // packets will be above MAX/2, new packets will be below MAX/2
    let diff = metadata.counter.wrapping_sub(compare_counter);

    if diff < CounterType::MAX / 2 {
        let missed_packets = metadata.counter.wrapping_sub(compare_counter);

        if metadata.to_addr.is_broadcast() {
            mapping.broadcast_counter = metadata.counter.wrapping_add(1);
        } else {
            mapping.from_counter = metadata.counter.wrapping_add(1);
        }

        Ok(missed_packets as usize)
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        big_brother::{Broadcastable, UDP_PORT},
        dedupe::{is_duplicate, CounterType},
        network_map::NetworkMapEntry,
        serdes::PacketMetadata,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TestNetworkAddress {
        Broadcast,
        A,
        B,
    }

    #[test]
    fn update_counter() {
        let mut mapping = NetworkMapEntry {
            network_address: TestNetworkAddress::A,
            ip: [192, 168, 0, 1],
            port: UDP_PORT,
            interface_index: 0,
            to_counter: 0,
            from_counter: 0,
            broadcast_counter: 0,
            from_session_id: 0,
        };

        let mut metadata = PacketMetadata {
            to_addr: TestNetworkAddress::A,
            from_addr: TestNetworkAddress::B,
            counter: 0,
        };

        // Make sure the counter goes from 0 to 1
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.from_counter, 1);
        assert_eq!(mapping.broadcast_counter, 0);

        // Make sure the counter can jump if a gap in packets is detected
        metadata.counter = 40;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.from_counter, 41);
        assert_eq!(mapping.broadcast_counter, 0);

        // Make sure the counter won't go backwards
        metadata.counter = 0;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.from_counter, 41);
        assert_eq!(mapping.broadcast_counter, 0);

        // Make sure the counter can wrap around
        metadata.counter = CounterType::MAX;
        mapping.from_counter = CounterType::MAX;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.from_counter, 0);
        assert_eq!(mapping.broadcast_counter, 0);
    }

    #[test]
    fn dedupe_ok() {
        let mut mapping = NetworkMapEntry {
            network_address: TestNetworkAddress::A,
            ip: [192, 168, 0, 1],
            port: UDP_PORT,
            interface_index: 0,
            to_counter: 0,
            from_counter: 0,
            broadcast_counter: 0,
            from_session_id: 0,
        };

        let mut metadata = PacketMetadata {
            to_addr: TestNetworkAddress::A,
            from_addr: TestNetworkAddress::B,
            counter: 0,
        };

        assert_eq!(is_duplicate(&metadata, &mut mapping), Ok(0));
        assert_eq!(is_duplicate(&metadata, &mut mapping), Err(()));

        metadata.counter = 1;
        assert_eq!(is_duplicate(&metadata, &mut mapping), Ok(0));
        assert_eq!(is_duplicate(&metadata, &mut mapping), Err(()));

        metadata.counter = 0;
        assert_eq!(is_duplicate(&metadata, &mut mapping), Err(()));

        metadata.counter = CounterType::MAX;
        mapping.from_counter = CounterType::MAX;
        // println!("from_counter: {} metadata.counter: {}", mapping.from_counter, metadata.counter);
        assert!(is_duplicate(&metadata, &mut mapping).is_ok());
        println!(
            "from_counter: {} metadata.counter: {}",
            mapping.from_counter, metadata.counter
        );
        assert_eq!(is_duplicate(&metadata, &mut mapping), Err(()));
    }

    // #[test]
    // fn start_with_high_metadata_counter() {
    //     let mut mapping = NetworkMapEntry {
    //         network_address: TestNetworkAddress::A,
    //         ip: [192, 168, 0, 1],
    //         port: UDP_PORT,
    //         interface_index: 0,
    //         to_counter: 0,
    //         from_counter: 0,
    //         broadcast_counter: 0,
    //         from_session_id: 0,
    //     };

    //     let mut metadata = PacketMetadata {
    //         to_addr: TestNetworkAddress::A,
    //         from_addr: TestNetworkAddress::B,
    //         counter: CounterType::MAX - 1024,
    //     };

    //     assert!(is_duplicate(&metadata, &mut mapping).is_ok());
    // }

    #[test]
    fn monotonic_dedupe() {
        let mut mapping = NetworkMapEntry {
            network_address: TestNetworkAddress::A,
            ip: [192, 168, 0, 1],
            port: UDP_PORT,
            interface_index: 0,
            to_counter: 0,
            from_counter: 0,
            broadcast_counter: 0,
            from_session_id: 0,
        };

        let mut metadata = PacketMetadata {
            to_addr: TestNetworkAddress::A,
            from_addr: TestNetworkAddress::B,
            counter: 0,
        };

        let increment = ((CounterType::MAX as usize) / (u16::MAX as usize)).max(1) as u128;
        let mut i = 0;

        // Do 2 full wraps around the counter
        loop {
            if i > ((CounterType::MAX) as u128) * 2 + 1 {
                break;
            }

            assert_eq!(is_duplicate(&metadata, &mut mapping), Ok(0));
            metadata.counter = metadata.counter.wrapping_add(1);

            i += increment;
        }
    }

    #[test]
    fn monotonic_broadcast_dedupe() {
        let mut mapping = NetworkMapEntry {
            network_address: TestNetworkAddress::A,
            ip: [192, 168, 0, 1],
            port: UDP_PORT,
            interface_index: 0,
            to_counter: 0,
            from_counter: 0,
            broadcast_counter: 0,
            from_session_id: 0,
        };

        let mut metadata = PacketMetadata {
            to_addr: TestNetworkAddress::Broadcast,
            from_addr: TestNetworkAddress::B,
            counter: 0,
        };

        let increment = ((CounterType::MAX as usize) / (u16::MAX as usize)).max(1) as u128;
        let mut i = 0;

        // Do 2 full wraps around the counter
        loop {
            if i > ((CounterType::MAX) as u128) * 2 + 1 {
                break;
            }

            assert_eq!(is_duplicate(&metadata, &mut mapping), Ok(0));
            metadata.counter = metadata.counter.wrapping_add(1);

            i += increment;
        }
    }

    #[test]
    fn update_broadcast_counter() {
        let mut mapping = NetworkMapEntry {
            network_address: TestNetworkAddress::A,
            ip: [192, 168, 0, 1],
            port: UDP_PORT,
            interface_index: 0,
            to_counter: 0,
            from_counter: 0,
            broadcast_counter: 0,
            from_session_id: 0,
        };

        let mut metadata = PacketMetadata {
            to_addr: TestNetworkAddress::Broadcast,
            from_addr: TestNetworkAddress::B,
            counter: 0,
        };

        // Make sure the counter goes from 0 to 1
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.broadcast_counter, 1);
        assert_eq!(mapping.from_counter, 0);

        // Make sure the counter can jump if a gap in packets is detected
        metadata.counter = 40;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.broadcast_counter, 41);
        assert_eq!(mapping.from_counter, 0);

        // Make sure the counter won't go backwards
        metadata.counter = 0;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.broadcast_counter, 41);
        assert_eq!(mapping.from_counter, 0);

        // Make sure the counter can wrap around
        metadata.counter = CounterType::MAX;
        mapping.broadcast_counter = CounterType::MAX;
        let _ = is_duplicate(&metadata, &mut mapping);
        assert_eq!(mapping.broadcast_counter, 0);
        assert_eq!(mapping.from_counter, 0);
    }

    impl Broadcastable for TestNetworkAddress {
        fn is_broadcast(&self) -> bool {
            match self {
                TestNetworkAddress::Broadcast => true,
                _ => false,
            }
        }
    }
}
