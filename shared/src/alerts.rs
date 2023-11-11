use core::marker::PhantomData;

use crate::comms_hal::Packet;

pub type AlertBitmaskType = u32;

pub struct AlertManager<T> {
    _marker: PhantomData<T>,
    condition_bitmask: AlertBitmaskType,
    pending_update: bool,
}

impl<T> AlertManager<T>
where
    T: Into<AlertBitmaskType>,
{
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            condition_bitmask: 0,
            pending_update: false,
        }
    }

    pub fn set_condition(&mut self, condition: T) {
        self.condition_bitmask |= 1 << condition.into();
        self.pending_update = true;
    }

    pub fn clear_condition(&mut self, condition: T) {
        self.condition_bitmask &= !(1 << condition.into());
        self.pending_update = true;
    }

    pub fn get_condition_bitmask(&mut self) -> AlertBitmaskType {
        self.pending_update = false;
        self.condition_bitmask
    }

    pub fn get_condition_packet(&mut self) -> Packet {
        Packet::AlertBitmask(self.get_condition_bitmask())
    }

    pub fn has_pending_update(&self) -> bool {
        self.pending_update
    }
}
