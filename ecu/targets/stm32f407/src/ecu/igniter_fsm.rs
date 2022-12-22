use core::marker::PhantomData;

use hal::{ecu_hal::{IgniterState, FuelTankState}, comms_hal::Packet};

use super::{ECUState, ECUControlPins};

pub struct IgniterStateStorage {
    elapsed_since_state_transition: f32,
}

struct Idle;
struct StartupGOx;
struct StartupIgnition;
struct Firing;
struct Shutdown;

struct IgniterFSM<T> {
    _m: PhantomData<T>,
}

impl IgniterFSM<Idle> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> { None }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_low();
        pins.sv2_ctrl.set_low();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(state: &mut ECUState, _pins: &mut ECUControlPins, packet: &Packet) -> Option<IgniterState> {
        match packet {
            Packet::FireIgniter => {
                if state.fuel_tank_state == FuelTankState::Pressurized {
                    return Some(IgniterState::StartupGOxLead);
                }
            },
            _ => {}
        }

        None
    }
}

impl IgniterFSM<StartupGOx> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.fuel_tank_state != FuelTankState::Pressurized {
            // Do an abort
        }
    
        if state.igniter_state_storage.elapsed_since_state_transition > 0.250 {
            return Some(IgniterState::StartupIgnition);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_low();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, _packet: &Packet) -> Option<IgniterState> { None }
}

impl IgniterFSM<StartupIgnition> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.fuel_tank_state != FuelTankState::Pressurized {
            // Do an abort
        }
    
        if state.igniter_state_storage.elapsed_since_state_transition > 0.250 {
            return Some(IgniterState::Firing);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_high();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.enable();
        pins.spark_ctrl.set_duty(pins.spark_ctrl.get_max_duty() / 4);
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, _packet: &Packet) -> Option<IgniterState> { None }
}

impl IgniterFSM<Firing> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.fuel_tank_state != FuelTankState::Pressurized {
            // Do an abort
        }
    
        if state.igniter_state_storage.elapsed_since_state_transition > 1.0 {
            return Some(IgniterState::Shutdown);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_high();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.enable();
        pins.spark_ctrl.set_duty(pins.spark_ctrl.get_max_duty() / 4);
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, _packet: &Packet) -> Option<IgniterState> { None }
}

impl IgniterFSM<Shutdown> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.igniter_state_storage.elapsed_since_state_transition > 0.5 {
            return Some(IgniterState::Idle);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_low();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, _packet: &Packet) -> Option<IgniterState> { None }
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, elapsed_time: f32) {
    ecu_state.igniter_state_storage.elapsed_since_state_transition += elapsed_time;

    let transition = match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::update(ecu_state, ecu_pins),
        IgniterState::StartupGOxLead => IgniterFSM::<StartupGOx>::update(ecu_state, ecu_pins),
        IgniterState::StartupIgnition => IgniterFSM::<StartupIgnition>::update(ecu_state, ecu_pins),
        IgniterState::Firing => IgniterFSM::<Firing>::update(ecu_state, ecu_pins),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::update(ecu_state, ecu_pins),
        IgniterState::Abort => None,
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::StartupGOxLead => IgniterFSM::<StartupGOx>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::StartupIgnition => IgniterFSM::<StartupIgnition>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::Firing => IgniterFSM::<Firing>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::Abort => None,
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn transition_state(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, new_state: IgniterState) {
    if ecu_state.igniter_state == new_state {
        return;
    }

    ecu_state.igniter_state = new_state;
    ecu_state.igniter_state_storage.elapsed_since_state_transition = 0.0;

    match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::enter_state(ecu_state, ecu_pins),
        IgniterState::StartupGOxLead => IgniterFSM::<StartupGOx>::enter_state(ecu_state, ecu_pins),
        IgniterState::StartupIgnition => IgniterFSM::<StartupIgnition>::enter_state(ecu_state, ecu_pins),
        IgniterState::Firing => IgniterFSM::<Firing>::enter_state(ecu_state, ecu_pins),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::enter_state(ecu_state, ecu_pins),
        IgniterState::Abort => {},
    }
}

impl IgniterStateStorage {
    pub const fn default() -> Self {
        Self {
            elapsed_since_state_transition: 0.0,
        }
    }
}