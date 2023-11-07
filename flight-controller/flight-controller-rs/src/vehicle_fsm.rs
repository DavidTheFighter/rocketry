use crate::Fcu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::VehicleState,
};
use nalgebra::Vector3;

mod ascent;
mod armed;
mod calibrating;
mod descent;
mod idle;
mod ignition;
mod landed;

#[derive(Debug)]
pub struct Idle;

#[derive(Debug)]
pub struct Calibrating {
    start_time: f32,
    accelerometer: Vector3<f32>,
    gyroscope: Vector3<f32>,
    magnetometer: Vector3<f32>,
    barometer_pressure: f32,
    data_count: u32,
    zero: bool,
}

#[derive(Debug)]
pub struct Armed;

#[derive(Debug)]
pub struct Ignition;

#[derive(Debug)]
pub struct Ascent;

#[derive(Debug)]
pub struct Descent;

#[derive(Debug)]
pub struct Landed;

#[derive(Debug)]
pub enum FsmState {
    Idle(Idle),
    Calibrating(Calibrating),
    Armed(Armed),
    Ignition(Ignition),
    Ascent(Ascent),
    Descent(Descent),
    Landed(Landed),
}

impl FsmState {
    fn to_fsm_component(&mut self) -> &mut dyn ComponentStateMachine<FsmState> {
        match self {
            FsmState::Idle(state) => state,
            FsmState::Calibrating(state) => state,
            FsmState::Armed(state) => state,
            FsmState::Ignition(state) => state,
            FsmState::Ascent(state) => state,
            FsmState::Descent(state) => state,
            FsmState::Landed(state) => state,
        }
    }
}

impl<'a> Fcu<'a> {
    pub fn update_vehicle_fsm(&mut self, dt: f32, packets: &[(NetworkAddress, Packet)]) {
        let mut current_state = self.vehicle_fsm_state.take().unwrap();
        let new_state = current_state.to_fsm_component().update(self, dt, packets);

        if let Some(new_state) = new_state {
            self.transition_vehicle_state(Some(current_state), new_state);
        } else {
            self.vehicle_fsm_state = Some(current_state);
        }
    }

    fn transition_vehicle_state(&mut self, old_state: Option<FsmState>, mut new_state: FsmState) {
        if let Some(mut old_state) = old_state {
            old_state.to_fsm_component().exit_state(self);
        }

        new_state.to_fsm_component().enter_state(self);

        self.vehicle_state = new_state.to_fsm_component().hal_state();
        self.vehicle_fsm_state = Some(new_state);
    }

    pub fn init_vehicle_fsm(&mut self) {
        let new_state = Calibrating::new(self, true);
        self.transition_vehicle_state(None, new_state);
    }
}

pub(crate) trait ComponentStateMachine<D> {
    fn update<'a>(
        &mut self,
        fcu: &'a mut Fcu,
        dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<D>;
    fn enter_state<'a>(&mut self, fcu: &'a mut Fcu);
    fn exit_state<'a>(&mut self, fcu: &'a mut Fcu);
    fn hal_state(&self) -> VehicleState;
}
