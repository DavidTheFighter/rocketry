use hal::{fcu_hal::VehicleState, comms_hal::Packet};
use crate::{Fcu, FiniteStateMachine};

mod idle;
mod ascent;
mod descent;
mod landed;

pub struct Idle;

pub struct Ascent;

pub struct Descent;

pub struct Landed;

pub enum FsmStorage {
    Empty,
    Idle(Idle),
    Ascent(Ascent),
    Descent(Descent),
    Landed(Landed),
}

impl<'a> Fcu<'a> {
    pub fn update_vehicle_fsm(&mut self, dt: f32, packets: &[Packet]) {
        let new_state = match self.vehicle_state {
            VehicleState::Idle => Idle::update(self, dt, packets),
            VehicleState::Ascent => Ascent::update(self, dt, packets),
            VehicleState::Descent => Descent::update(self, dt, packets),
            VehicleState::Landed => Landed::update(self, dt, packets),
        };

        if let Some(new_state) = new_state {
            self.transition_vehicle_state(new_state);
        }
    }

    fn transition_vehicle_state(&mut self, new_state: VehicleState) {
        self.vehicle_state = new_state;

        match new_state {
            VehicleState::Idle => Idle::setup_state(self),
            VehicleState::Ascent => Ascent::setup_state(self),
            VehicleState::Descent => Descent::setup_state(self),
            VehicleState::Landed => Landed::setup_state(self),
        }
    }

    pub fn init_vehicle_fsm(&mut self) {
        self.transition_vehicle_state(VehicleState::Idle);
    }
}
