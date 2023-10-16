use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use nalgebra::Vector3;
use crate::{Fcu, FiniteStateMachine};

mod idle;
mod calibrating;
mod ascent;
mod descent;
mod landed;

pub struct Idle;

pub struct Calibrating {
    start_time: f32,
    accelerometer: Vector3<f32>,
    gyroscope: Vector3<f32>,
    magnetometer: Vector3<f32>,
    barometric_altitude: f32,
    data_count: u32,
}

pub struct Ascent;

pub struct Descent;

pub struct Landed;

pub enum FsmStorage {
    Empty,
    Idle(Idle),
    Calibrating(Calibrating),
    Ascent(Ascent),
    Descent(Descent),
    Landed(Landed),
}

impl<'a> Fcu<'a> {
    pub fn update_vehicle_fsm(&mut self, dt: f32, packets: &[(NetworkAddress, Packet)]) {
        let new_state = match self.vehicle_state {
            VehicleState::Idle => Idle::update(self, dt, packets),
            VehicleState::Calibrating => Calibrating::update(self, dt, packets),
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
            VehicleState::Calibrating => Calibrating::setup_state(self),
            VehicleState::Ascent => Ascent::setup_state(self),
            VehicleState::Descent => Descent::setup_state(self),
            VehicleState::Landed => Landed::setup_state(self),
        }
    }

    pub fn init_vehicle_fsm(&mut self) {
        self.transition_vehicle_state(self.vehicle_state);
    }
}
