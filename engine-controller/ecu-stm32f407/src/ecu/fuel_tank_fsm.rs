use shared::{comms_hal::Packet, ecu_hal::FuelTankState};

use super::{ECUControlPins, ECUState};

struct Fsm<const T: usize>;

const FSM_IDLE: usize = FuelTankState::Idle as usize;
const FSM_DEPRESSURIZED: usize = FuelTankState::Depressurized as usize;
const FSM_PRESSURIZED: usize = FuelTankState::Pressurized as usize;

impl Fsm<FSM_IDLE> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<FuelTankState> {
        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv3_ctrl.set_low();
        pins.sv4_ctrl.set_low();
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        packet: &Packet,
    ) -> Option<FuelTankState> {
        if let Packet::TransitionFuelTankState(new_state) = packet {
            Some(*new_state)
        } else {
            None
        }
    }
}

impl Fsm<FSM_DEPRESSURIZED> {
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
        if let Packet::TransitionFuelTankState(new_state) = packet {
            Some(*new_state)
        } else {
            None
        }
    }
}

impl Fsm<FSM_PRESSURIZED> {
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
        if let Packet::TransitionFuelTankState(new_state) = packet {
            Some(*new_state)
        } else {
            None
        }
    }
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, _elapsed_time: f32) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => Fsm::<FSM_IDLE>::update(ecu_state, ecu_pins),
        FuelTankState::Depressurized => Fsm::<FSM_DEPRESSURIZED>::update(ecu_state, ecu_pins),
        FuelTankState::Pressurized => Fsm::<FSM_PRESSURIZED>::update(ecu_state, ecu_pins),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.fuel_tank_state {
        FuelTankState::Idle => Fsm::<FSM_IDLE>::on_packet(ecu_state, ecu_pins, packet),
        FuelTankState::Depressurized => Fsm::<FSM_DEPRESSURIZED>::on_packet(ecu_state, ecu_pins, packet),
        FuelTankState::Pressurized => Fsm::<FSM_PRESSURIZED>::on_packet(ecu_state, ecu_pins, packet),
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
        FuelTankState::Idle => Fsm::<FSM_IDLE>::enter_state(ecu_state, ecu_pins),
        FuelTankState::Depressurized => Fsm::<FSM_DEPRESSURIZED>::enter_state(ecu_state, ecu_pins),
        FuelTankState::Pressurized => Fsm::<FSM_PRESSURIZED>::enter_state(ecu_state, ecu_pins),
    }
}
