use hal::{comms_hal::Packet, ecu_hal::FuelTankState};

use super::{ECUControlPins, ECUState};

struct Fsm<const T: usize>;

macro_rules! state {
    ($state: ident) => {
        FuelTankState::$state as usize
    };
}

impl Fsm<{ state!(Idle) }> {
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
        if let Packet::PressurizeFuelTank = packet {
            Some(FuelTankState::Pressurized)
        } else {
            None
        }
    }
}

impl Fsm<{ state!(Pressurized) }> {
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
        if let Packet::DepressurizeFuelTank = packet {
            Some(FuelTankState::Idle)
        } else {
            None
        }
    }
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, _elapsed_time: f32) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => Fsm::<{ state!(Idle) }>::update(ecu_state, ecu_pins),
        FuelTankState::Pressurized => Fsm::<{ state!(Pressurized) }>::update(ecu_state, ecu_pins),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => Fsm::<{ state!(Idle) }>::on_packet(ecu_state, ecu_pins, packet),
        FuelTankState::Pressurized => {
            Fsm::<{ state!(Pressurized) }>::on_packet(ecu_state, ecu_pins, packet)
        }
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
        FuelTankState::Idle => Fsm::<{ state!(Idle) }>::enter_state(ecu_state, ecu_pins),
        FuelTankState::Pressurized => {
            Fsm::<{ state!(Pressurized) }>::enter_state(ecu_state, ecu_pins)
        }
    }
}
