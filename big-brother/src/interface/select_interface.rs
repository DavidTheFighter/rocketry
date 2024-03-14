use super::BigBrotherInterface;


pub struct SelectInterface {
    iface0: Box<dyn BigBrotherInterface>,
    iface1: Box<dyn BigBrotherInterface>,
    selected: u8,
}

impl SelectInterface {
    pub fn new(iface0: Box<dyn BigBrotherInterface>, iface1: Box<dyn BigBrotherInterface>) -> Self {
        Self {
            iface0,
            iface1,
            selected: 0,
        }
    }

    pub fn select(&mut self, iface: u8) {
        self.selected = iface;
    }

    fn iface(&self) -> &dyn BigBrotherInterface {
        match self.selected {
            0 => &*self.iface0,
            1 => &*self.iface1,
            _ => panic!("Invalid interface selected"),
        }
    }

    fn iface_mut(&mut self) -> &mut dyn BigBrotherInterface {
        match self.selected {
            0 => &mut *self.iface0,
            1 => &mut *self.iface1,
            _ => panic!("Invalid interface selected"),
        }
    }
}

impl BigBrotherInterface for SelectInterface {
    fn poll(&mut self, timestamp: u32) {
        self.iface_mut().poll(timestamp);
    }

    fn send_udp(
        &mut self,
        destination: super::BigBrotherEndpoint,
        data: &mut [u8],
    ) -> Result<(), super::BigBrotherError> {
        self.iface_mut().send_udp(destination, data)
    }

    fn recv_udp(
        &mut self,
        data: &mut [u8],
    ) -> Result<Option<(usize, crate::big_brother::BigBrotherEndpoint)>, crate::big_brother::BigBrotherError> {
        self.iface_mut().recv_udp(data)
    }

    fn broadcast_ip(&self) -> [u8; 4] {
        self.iface().broadcast_ip()
    }

    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any> {
        self.iface_mut().as_mut_any()
    }
}


