use crate::big_brother::{BigBrotherEndpoint, BigBrotherError};

#[cfg(feature = "smoltcp")]
pub mod smoltcp_interface;

#[cfg(not(feature = "no_std"))]
pub mod std_interface;

// #[cfg(not(feature = "no_std"))]
// pub mod select_interface;

#[cfg(any(not(feature = "no_std"), test))]
pub mod mock_interface;

#[cfg(any(not(feature = "no_std"), test))]
pub mod mock_topology;

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

    fn broadcast_ip(&self) -> [u8; 4];
    fn as_mut_any(&mut self) -> Option<&mut dyn core::any::Any>;
}
