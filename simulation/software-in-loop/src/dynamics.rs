pub mod combustion;
pub mod fluid;
pub mod igniter;
pub mod tank;
pub mod vehicle;

type Scalar = f64;

pub const ATMOSPHERIC_PRESSURE_PA: Scalar = 101325.0;

pub use tank::SilTankDynamics;
pub use tank::SilTankFeedConfig;
pub use vehicle::SilVehicleDynamics;
