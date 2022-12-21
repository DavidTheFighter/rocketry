use hal::ecu_hal::FuelTankState;

use super::{ECUState, ECUControlPins};

pub struct FuelTankStateStorage {
    pub pressurize_command_received: bool,
    pub depressurize_command_received: bool,
}

// ------- IDLE STATE ------- //

fn fuel_tank_idle_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.fuel_tank_state_storage.pressurize_command_received {
        transition_state(ecu_state, ecu_pins, FuelTankState::Pressurized);
    }
}

fn fuel_tank_idle_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv3_ctrl.set_low();
    ecu_pins.sv4_ctrl.set_high();
}

// ------- PRESSURIZED STATE ------- //

fn fuel_tank_pressurized_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.fuel_tank_state_storage.depressurize_command_received {
        transition_state(ecu_state, ecu_pins, FuelTankState::Idle);
    }
}

fn fuel_tank_pressurized_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv3_ctrl.set_high();
    ecu_pins.sv4_ctrl.set_low();
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, _elapsed_time: f32) {
    match ecu_state.fuel_tank_state {
        FuelTankState::Idle => fuel_tank_idle_update(ecu_state, ecu_pins),
        FuelTankState::Pressurized => fuel_tank_pressurized_update(ecu_state, ecu_pins),
    }

    ecu_state.fuel_tank_state_storage.pressurize_command_received = false;
    ecu_state.fuel_tank_state_storage.depressurize_command_received = false;
}

pub fn transition_state(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, new_state: FuelTankState) {
    if ecu_state.fuel_tank_state == new_state {
        return;
    }

    ecu_state.fuel_tank_state = new_state;

    match new_state {
        FuelTankState::Idle => fuel_tank_idle_transition_into(ecu_state, ecu_pins),
        FuelTankState::Pressurized => fuel_tank_pressurized_transition_into(ecu_state, ecu_pins),
    }
}

impl FuelTankStateStorage {
    pub const fn default() -> Self {
        Self {
            pressurize_command_received: false,
            depressurize_command_received: false,
        }
    }
}