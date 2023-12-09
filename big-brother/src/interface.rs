use crate::big_brother::{BigBrotherEndpoint, BigBrotherError};

#[cfg(feature = "smoltcp")]
pub mod smoltcp_interface;

#[cfg(feature = "stdtcp")]
pub mod std_interface;

pub trait BigBrotherInterface {
    fn poll(&mut self, timestamp: u32);

    fn send_udp(
        &mut self,
        destination: BigBrotherEndpoint,
        data: &mut [u8],
    ) -> Result<(), BigBrotherError>;
    fn recv_udp(
        &mut self,
        data: &mut [u8],
    ) -> Result<Option<(usize, BigBrotherEndpoint)>, BigBrotherError>;

    fn as_mut_any(&'static mut self) -> &mut dyn core::any::Any;
}
