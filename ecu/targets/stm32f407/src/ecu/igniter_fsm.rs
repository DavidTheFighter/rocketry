use hal::ecu_hal::{IgniterState, FuelTankState};

use super::{ECUState, ECUControlPins};

pub struct IgniterStateStorage {
    elapsed_since_state_transition: f32,
    pub received_fire_igniter_command: bool,
}

// ------- IDLE STATE ------- //

fn igniter_idle_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.igniter_state_storage.received_fire_igniter_command {
        if ecu_state.fuel_tank_state == FuelTankState::Pressurized {
            transition_state(ecu_state, ecu_pins, IgniterState::StartupGOxLead);
            return;
        } else {
            // TODO Output error
        }
    }
}

fn igniter_idle_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv1_ctrl.set_low();
    ecu_pins.sv2_ctrl.set_low();
    ecu_pins.spark_ctrl.disable();
    ecu_pins.spark_ctrl.set_duty(0);
}

// ------- STARTUP GOX LEAD STATE ------- //

fn igniter_startup_gox_lead_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.fuel_tank_state != FuelTankState::Pressurized {
        // Do an abort
    }

    if ecu_state.igniter_state_storage.elapsed_since_state_transition > 0.250 {
        transition_state(ecu_state, ecu_pins, IgniterState::StartupIgnition);
    }
}

fn igniter_startup_gox_lead_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv1_ctrl.set_low();
    ecu_pins.sv2_ctrl.set_high();
    ecu_pins.spark_ctrl.disable();
    ecu_pins.spark_ctrl.set_duty(0);
}

// ------- STARTUP IGNITION STATE ------- //

fn igniter_startup_ignition_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.fuel_tank_state != FuelTankState::Pressurized {
        // Do an abort
    }

    if ecu_state.igniter_state_storage.elapsed_since_state_transition > 0.250 {
        transition_state(ecu_state, ecu_pins, IgniterState::Firing);
    }
}

fn igniter_startup_ignition_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv1_ctrl.set_high();
    ecu_pins.sv2_ctrl.set_high();
    ecu_pins.spark_ctrl.enable();
    ecu_pins.spark_ctrl.set_duty(ecu_pins.spark_ctrl.get_max_duty() / 4);
}

// ------- FIRING STATE ------- //

fn igniter_firing_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.igniter_state_storage.elapsed_since_state_transition > 1.0 {
        transition_state(ecu_state, ecu_pins, IgniterState::Shutdown);
    }
}

fn igniter_firing_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv1_ctrl.set_high();
    ecu_pins.sv2_ctrl.set_high();
    ecu_pins.spark_ctrl.enable();
    ecu_pins.spark_ctrl.set_duty(ecu_pins.spark_ctrl.get_max_duty() / 4);
}

// ------- SHUTDOWN STATE ------- //

fn igniter_shutdown_update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    if ecu_state.igniter_state_storage.elapsed_since_state_transition > 0.5 {
        transition_state(ecu_state, ecu_pins, IgniterState::Idle);
    }
}

fn igniter_shutdown_transition_into(_ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    ecu_pins.sv1_ctrl.set_low();
    ecu_pins.sv2_ctrl.set_high();
    ecu_pins.spark_ctrl.disable();
    ecu_pins.spark_ctrl.set_duty(0);
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, elapsed_time: f32) {
    ecu_state.igniter_state_storage.elapsed_since_state_transition += elapsed_time;

    match ecu_state.igniter_state {
        IgniterState::Idle => igniter_idle_update(ecu_state, ecu_pins),
        IgniterState::StartupGOxLead => igniter_startup_gox_lead_update(ecu_state, ecu_pins),
        IgniterState::StartupIgnition => igniter_startup_ignition_update(ecu_state, ecu_pins),
        IgniterState::Firing => igniter_firing_update(ecu_state, ecu_pins),
        IgniterState::Shutdown => igniter_shutdown_update(ecu_state, ecu_pins),
        IgniterState::Abort => todo!(),
    }

    ecu_state.igniter_state_storage.received_fire_igniter_command = false;
}

pub fn transition_state(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, new_state: IgniterState) {
    if ecu_state.igniter_state == new_state {
        return;
    }

    ecu_state.igniter_state = new_state;
    ecu_state.igniter_state_storage.elapsed_since_state_transition = 0.0;

    match ecu_state.igniter_state {
        IgniterState::Idle => igniter_idle_transition_into(ecu_state, ecu_pins),
        IgniterState::StartupGOxLead => igniter_startup_gox_lead_transition_into(ecu_state, ecu_pins),
        IgniterState::StartupIgnition => igniter_startup_ignition_transition_into(ecu_state, ecu_pins),
        IgniterState::Firing => igniter_firing_transition_into(ecu_state, ecu_pins),
        IgniterState::Shutdown => igniter_shutdown_transition_into(ecu_state, ecu_pins),
        IgniterState::Abort => todo!(),
    }
}

impl IgniterStateStorage {
    pub const fn default() -> Self {
        Self {
            elapsed_since_state_transition: 0.0,
            received_fire_igniter_command: false,
        }
    }
}