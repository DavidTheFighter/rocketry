use core::marker::PhantomData;

use hal::{comms_hal::Packet, ecu_hal::FuelTankState};

use super::{ECUControlPins, ECUState};

struct Idle;
struct Pressurized;

struct FSM<T> {
    _m: PhantomData<T>,
}

impl FSM<Idle> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<FuelTankState> {
        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv3_ctrl.set_low();
        pins.sv4_ctrl.set_high();
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        packet: &Packet,
    ) -> Option<FuelTankState> {
        match packet {
            Packet::PressurizeFuelTank => return Some(FuelTankState::Pressurized),
            _ => {}
        }

        None
    }
}

impl FSM<Pressurized> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<FuelTankState> {
        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv3_ctrl.set_high();
        pins.sv4_ctrl.set_low();
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        packet: &Packet,
    ) -> Option<FuelTankState> {
        match packet {
            Packet::DepressurizeFuelTank => return Some(FuelTankState::Idle),
            _ => {}
        }

        None
    }
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, _elapsed_time: f32) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => FSM::<Idle>::update(ecu_state, ecu_pins),
        FuelTankState::Pressurized => FSM::<Pressurized>::update(ecu_state, ecu_pins),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => FSM::<Idle>::on_packet(ecu_state, ecu_pins, packet),
        FuelTankState::Pressurized => FSM::<Pressurized>::on_packet(ecu_state, ecu_pins, packet),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn transition_state(
    ecu_state: &mut ECUState,
    ecu_pins: &mut ECUControlPins,
    new_state: FuelTankState,
) {
    if ecu_state.fuel_tank_state == new_state {
        return;
    }

    ecu_state.fuel_tank_state = new_state;

    match new_state {
        FuelTankState::Idle => FSM::<Idle>::enter_state(ecu_state, ecu_pins),
        FuelTankState::Pressurized => FSM::<Pressurized>::enter_state(ecu_state, ecu_pins),
    }
}
