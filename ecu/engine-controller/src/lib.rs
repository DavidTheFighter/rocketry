#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub(crate) mod fuel_tank_fsm;
pub(crate) mod igniter_fsm;

use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuDriver, FuelTankState, IgniterConfig, IgniterState},
};

pub struct Ecu<'a> {
    pub igniter_config: IgniterConfig,
    pub igniter_state: IgniterState,
    pub(crate) igniter_fsm_storage: igniter_fsm::FsmStorage,
    pub fuel_tank_state: FuelTankState,
    pub(crate) driver: &'a mut dyn EcuDriver,
}

impl<'a> Ecu<'a> {
    pub fn new(driver: &'a mut dyn EcuDriver) -> Self {
        let mut ecu = Self {
            igniter_config: IgniterConfig::default(),
            igniter_state: IgniterState::Idle,
            igniter_fsm_storage: igniter_fsm::FsmStorage::Empty,
            fuel_tank_state: FuelTankState::Idle,
            driver,
        };

        ecu.init_igniter_fsm();
        ecu.init_fuel_tank_fsm();

        ecu
    }

    pub fn update(&mut self, dt: f32, packet: Option<Packet>) {
        self.update_fuel_tank_fsm(dt, packet);
    }
}

pub(crate) trait FiniteStateMachine<T> {
    fn update(ecu: &mut Ecu, dt: f32, packet: Option<Packet>) -> Option<T>;
    fn setup_state(ecu: &mut Ecu);
}
