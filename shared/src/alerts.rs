use core::marker::PhantomData;

use crate::comms_hal::Packet;

pub type AlertBitmaskType = u128;

pub struct AlertManager<T> {
    _marker: PhantomData<T>,
    condition_bitmask: AlertBitmaskType,
    pending_update: bool,
    time_since_last_ping_s: f32,
    ping_rate_s: f32,
}

impl<T> AlertManager<T>
where
    T: Into<AlertBitmaskType>,
{
    pub fn new(ping_rate_s: f32) -> Self {
        Self {
            _marker: PhantomData,
            condition_bitmask: 0,
            pending_update: false,
            time_since_last_ping_s: 0.0,
            ping_rate_s,
        }
    }

    pub fn set_condition(&mut self, condition: T) {
        let condition = condition.into();
        if self.condition_bitmask & (1 << condition) == 0 {
            self.condition_bitmask |= 1 << condition;
            self.pending_update = true;
        }
    }

    pub fn clear_condition(&mut self, condition: T) {
        let condition = condition.into();
        if self.condition_bitmask & (1 << condition) != 0 {
            self.condition_bitmask &= !(1 << condition);
            self.pending_update = true;
        }
    }

    pub fn assign_condition(&mut self, condition: T, value: bool) {
        if value {
            self.set_condition(condition);
        } else {
            self.clear_condition(condition);
        }
    }

    pub fn get_condition_bitmask(&mut self) -> AlertBitmaskType {
        self.pending_update = false;
        self.condition_bitmask
    }

    pub fn update(&mut self, dt: f32) -> [Option<Packet>; 2] {
        self.time_since_last_ping_s += dt;
        if self.time_since_last_ping_s > self.ping_rate_s || self.pending_update {
            self.time_since_last_ping_s = 0.0;
            self.pending_update = false;

            [Some(Packet::AlertBitmask(self.get_condition_bitmask())), None]
        } else {
            [None, None]
        }
    }

    // pub fn get_condition_packet(&mut self) -> Packet {
    //     Packet::AlertBitmask(self.get_condition_bitmask())
    // }

    pub fn has_pending_update(&self) -> bool {
        self.pending_update
    }
}

pub fn is_condition_set(bitmask: AlertBitmaskType, condition: AlertBitmaskType) -> bool {
    (bitmask & (1 << condition)) != 0
}
