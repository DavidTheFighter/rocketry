use core::marker::PhantomData;

use hal::comms_hal::{NetworkAddress, Packet};
use serde::Serialize;


pub struct AlertManager<E, C, F> {
    _marker_e: PhantomData<E>,
    _marker_c: PhantomData<C>,
    send_packet_fn: F,
    host_address: NetworkAddress,
    condition_bitmask: u64,
}

impl<E, C, F> AlertManager<E, C, F>
where
    E: Into<u64>,
    C: Into<u64>,
    F: Fn((NetworkAddress, Packet)),
{
    pub fn new(send_packet_fn: F, host_address: NetworkAddress) -> Self {
        Self {
            _marker_e: PhantomData,
            _marker_c: PhantomData,
            send_packet_fn,
            host_address,
            condition_bitmask: 0,
        }
    }

    pub fn report_event(&mut self, event: E) {

    }

    pub fn set_condition(&mut self, condition: C) {
        self.condition_bitmask |= 1 << condition.into();

        self.send_condition_update();
    }

    pub fn clear_condition(&mut self, condition: C) {
        self.condition_bitmask &= !(1 << condition.into());

        self.send_condition_update();
    }

    fn send_condition_update(&mut self) {
        // let packet = Packet::AlertCondition { condition_bitmask };

        // self.send_packet_fn((self.host_address, packet));
    }
}