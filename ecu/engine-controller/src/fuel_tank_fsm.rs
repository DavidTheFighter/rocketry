use core::borrow::BorrowMut;

use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuSolenoidValve, FuelTankState},
};

use crate::{Ecu, FiniteStateMachine};

struct Idle;
struct Pressurized;
struct Depressurized;

impl FiniteStateMachine<FuelTankState> for Idle {
    fn update(_ecu: &mut Ecu, _dt: f32, packet: Option<Packet>) -> Option<FuelTankState> {
        if let Some(Packet::TransitionFuelTankState(new_state)) = packet {
            Some(new_state)
        } else {
            None
        }
    }

    fn setup_state(ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::FuelPress, false);
        driver.set_solenoid_valve(EcuSolenoidValve::FuelVent, false);
    }
}

impl FiniteStateMachine<FuelTankState> for Depressurized {
    fn update(_ecu: &mut Ecu, _dt: f32, packet: Option<Packet>) -> Option<FuelTankState> {
        if let Some(Packet::TransitionFuelTankState(new_state)) = packet {
            Some(new_state)
        } else {
            None
        }
    }

    fn setup_state(ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::FuelPress, true);
        driver.set_solenoid_valve(EcuSolenoidValve::FuelVent, false);
    }
}

impl FiniteStateMachine<FuelTankState> for Pressurized {
    fn update(_ecu: &mut Ecu, _dt: f32, packet: Option<Packet>) -> Option<FuelTankState> {
        if let Some(Packet::TransitionFuelTankState(new_state)) = packet {
            Some(new_state)
        } else {
            None
        }
    }

    fn setup_state(ecu: &mut Ecu) {
        let driver = ecu.driver.borrow_mut();

        driver.set_solenoid_valve(EcuSolenoidValve::FuelPress, false);
        driver.set_solenoid_valve(EcuSolenoidValve::FuelVent, true);
    }
}

impl<'a> Ecu<'a> {
    pub(crate) fn update_fuel_tank_fsm(&mut self, dt: f32, packet: Option<Packet>) {
        let new_state = match self.fuel_tank_state {
            FuelTankState::Idle => Idle::update(self, dt, packet),
            FuelTankState::Depressurized => Depressurized::update(self, dt, packet),
            FuelTankState::Pressurized => Pressurized::update(self, dt, packet),
        };

        if let Some(new_state) = new_state {
            self.transition_fuel_tank_state(new_state);
        }
    }

    fn transition_fuel_tank_state(&mut self, new_state: FuelTankState) {
        if self.fuel_tank_state == new_state {
            return;
        }

        self.fuel_tank_state = new_state;

        match new_state {
            FuelTankState::Idle => Idle::setup_state(self),
            FuelTankState::Depressurized => Depressurized::setup_state(self),
            FuelTankState::Pressurized => Pressurized::setup_state(self),
        }
    }

    pub fn init_fuel_tank_fsm(&mut self) {
        self.transition_fuel_tank_state(FuelTankState::Idle);
    }
}
