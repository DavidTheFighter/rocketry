#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub mod alerts;
pub mod comms_hal;
// pub mod comms_manager;
pub mod ecu_hal;
pub mod ecu_mock;
pub mod fcu_hal;
pub mod fcu_mock;
pub mod logger;
pub mod standard_atmosphere;
pub mod streamish_hal;

use comms_hal::{NetworkAddress, Packet};
use serde::{Deserialize, Serialize};

pub use logger::{DataPointLogger, FlashDataLogger};

pub const COMMS_NETWORK_MAP_SIZE: usize = 16;

pub const GRAVITY: f32 = 9.80665; // In m/s^2
pub const RESET_MAGIC_NUMBER: u64 = 0xabcd1234_5678ef90;

pub trait ControllerState<F, C> {
    fn update(
        &mut self,
        controller: &mut C,
        dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<F>;
    fn enter_state(&mut self, controller: &mut C);
    fn exit_state(&mut self, controller: &mut C);
}

pub trait ControllerFsm<F, C, S> {
    fn to_controller_state<'a>(&mut self) -> &mut dyn ControllerState<F, C>;
    fn hal_state(&self) -> S;
}

pub struct ControllerEntity<F, C, S> {
    fsm_state: Option<F>,
    _controller_marker: core::marker::PhantomData<C>,
    _hal_state_marker: core::marker::PhantomData<S>,
}

impl<F, C, S> ControllerEntity<F, C, S>
where
    F: ControllerFsm<F, C, S>,
{
    pub fn new(controller: &mut C, fsm_state: F) -> Self {
        let mut controller_fsm = Self {
            fsm_state: None,
            _controller_marker: core::marker::PhantomData,
            _hal_state_marker: core::marker::PhantomData,
        };

        controller_fsm.transition_state(controller, None, fsm_state);

        controller_fsm
    }

    pub fn update<'a>(
        &mut self,
        controller: &mut C,
        dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) {
        if let Some(mut current_state) = self.fsm_state.take() {
            let new_state = current_state
                .to_controller_state()
                .update(controller, dt, packets);

            if let Some(new_state) = new_state {
                self.transition_state(controller, Some(current_state), new_state);
            } else {
                self.fsm_state = Some(current_state);
            }
        }
    }

    fn transition_state(&mut self, controller: &mut C, old_state: Option<F>, mut new_state: F) {
        if let Some(mut old_state) = old_state {
            old_state.to_controller_state().exit_state(controller);
        }

        new_state.to_controller_state().enter_state(controller);

        self.fsm_state = Some(new_state);
    }

    pub fn hal_state(&self) -> S {
        self.fsm_state
            .as_ref()
            .expect("No FSM state, impl must be wrong")
            .hal_state()
    }
}

// Describes a polynomial calibration curve for a sensor.
// Given in the form: y = x0 + x1 * x + x2 * x^2 + x3 * x^3
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub x0: f32,
    pub x1: f32,
    pub x2: f32,
    pub x3: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
    pub calibration: Option<SensorCalibration>,
}

impl SensorConfig {
    pub const fn default() -> Self {
        Self {
            premin: 0.0,
            premax: 1.0,
            postmin: 0.0,
            postmax: 1.0,
            calibration: None,
        }
    }

    pub fn apply(&self, val: f32) -> f32 {
        let lerp = (val - self.premin) / (self.premax - self.premin);
        let mut value = lerp * (self.postmax - self.postmin) + self.postmin;

        if let Some(curve) = self.calibration {
            let x = value;
            value += curve.x0 + curve.x1 * x + curve.x2 * x * x + curve.x3 * x * x * x;
        }

        value
    }
}
