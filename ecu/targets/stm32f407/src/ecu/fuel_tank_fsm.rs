use core::marker::PhantomData;

use hal::{ecu_hal::FuelTankState, comms_hal::Packet};

use super::{ECUState, ECUControlPins};

struct Idle;
struct Pressurized;

struct FuelTankFSM<T> {
    _m: PhantomData<T>,
}

impl FuelTankFSM<Idle> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<FuelTankState> { None }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv3_ctrl.set_low();
        pins.sv4_ctrl.set_high();
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, packet: &Packet) -> Option<FuelTankState> {
        match packet {
            Packet::PressurizeFuelTank => return Some(FuelTankState::Pressurized),
            _ => {}
        }

        None
    }
}

impl FuelTankFSM<Pressurized> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<FuelTankState> { None }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv3_ctrl.set_high();
        pins.sv4_ctrl.set_low();
    }

    fn on_packet(_state: &mut ECUState, _pins: &mut ECUControlPins, packet: &Packet) -> Option<FuelTankState> {
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
        FuelTankState::Idle => FuelTankFSM::<Idle>::update(ecu_state, ecu_pins),
        FuelTankState::Pressurized => FuelTankFSM::<Pressurized>::update(ecu_state, ecu_pins),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => FuelTankFSM::<Idle>::on_packet(ecu_state, ecu_pins, packet),
        FuelTankState::Pressurized => FuelTankFSM::<Pressurized>::on_packet(ecu_state, ecu_pins, packet),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn transition_state(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, new_state: FuelTankState) {
    if ecu_state.fuel_tank_state == new_state {
        return;
    }

    ecu_state.fuel_tank_state = new_state;

    match new_state {
        FuelTankState::Idle => FuelTankFSM::<Idle>::enter_state(ecu_state, ecu_pins),
        FuelTankState::Pressurized => FuelTankFSM::<Pressurized>::enter_state(ecu_state, ecu_pins),
    }
}