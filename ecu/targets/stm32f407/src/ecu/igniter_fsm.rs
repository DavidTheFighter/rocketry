use core::marker::PhantomData;

use hal::{
    comms_hal::Packet,
    ecu_hal::{FuelTankState, IgniterState, EcuSensor},
};

use super::{ECUControlPins, ECUState};

pub struct IgniterStateStorage {
    elapsed_since_state_transition: f32,
    stable_pressure_counter: f32,
}

struct Idle;
struct StartupGOx;
struct Startup;
struct Firing;
struct Shutdown;

struct IgniterFSM<T> {
    _m: PhantomData<T>,
}

impl IgniterFSM<Idle> {
    fn update(_state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_low();
        pins.sv2_ctrl.set_low();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(
        state: &mut ECUState,
        _pins: &mut ECUControlPins,
        packet: &Packet,
    ) -> Option<IgniterState> {
        match packet {
            Packet::FireIgniter => {
                if state.fuel_tank_state == FuelTankState::Pressurized {
                    return Some(IgniterState::Startup);
                }
            }
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

        if state.igniter_state_storage.elapsed_since_state_transition > state.igniter_config.gox_lead_duration {
            return Some(IgniterState::Startup);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_low();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        _packet: &Packet,
    ) -> Option<IgniterState> {
        None
    }
}

impl IgniterFSM<Startup> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins, delta_time: f32) -> Option<IgniterState> {
        if state.fuel_tank_state != FuelTankState::Pressurized {
            // Do an abort
        }

        if state.sensor_maxs[EcuSensor::IgniterThroatTemp as usize] >= state.igniter_config.max_throat_temp {
            return Some(IgniterState::Shutdown);
        }

        if state.igniter_state_storage.elapsed_since_state_transition > state.igniter_config.startup_timeout {
            return Some(IgniterState::Shutdown);
        }

        if state.sensor_mins[EcuSensor::IgniterChamberPressure as usize] >= state.igniter_config.startup_pressure_threshold {
            state.igniter_state_storage.stable_pressure_counter += delta_time;
        } else {
            state.igniter_state_storage.stable_pressure_counter = 0.0;
        }

        if state.igniter_state_storage.stable_pressure_counter > state.igniter_config.startup_stable_time {
            return Some(IgniterState::Firing);
        }

        None
    }

    fn enter_state(state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_high();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.enable();
        pins.spark_ctrl.set_duty(pins.spark_ctrl.get_max_duty() / 8);

        state.igniter_state_storage.stable_pressure_counter = 0.0;
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        _packet: &Packet,
    ) -> Option<IgniterState> {
        None
    }
}

impl IgniterFSM<Firing> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.fuel_tank_state != FuelTankState::Pressurized {
            // Do an abort
        }

        if state.sensor_maxs[EcuSensor::IgniterThroatTemp as usize] >= state.igniter_config.max_throat_temp {
            return Some(IgniterState::Shutdown);
        }

        if state.igniter_state_storage.elapsed_since_state_transition > state.igniter_config.firing_duration {
            return Some(IgniterState::Shutdown);
        }

        None
    }

    fn enter_state(_state: &mut ECUState, pins: &mut ECUControlPins) {
        pins.sv1_ctrl.set_high();
        pins.sv2_ctrl.set_high();
        pins.spark_ctrl.disable();
        pins.spark_ctrl.set_duty(0);
    }

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        _packet: &Packet,
    ) -> Option<IgniterState> {
        None
    }
}

impl IgniterFSM<Shutdown> {
    fn update(state: &mut ECUState, _pins: &mut ECUControlPins) -> Option<IgniterState> {
        if state.igniter_state_storage.elapsed_since_state_transition > state.igniter_config.shutdown_duration {
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

    fn on_packet(
        _state: &mut ECUState,
        _pins: &mut ECUControlPins,
        _packet: &Packet,
    ) -> Option<IgniterState> {
        None
    }
}

// ---------------------------- //

pub fn update(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, delta_time: f32) {
    ecu_state
        .igniter_state_storage
        .elapsed_since_state_transition += delta_time;

    let transition = match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::update(ecu_state, ecu_pins),
        IgniterState::Startup => IgniterFSM::<Startup>::update(ecu_state, ecu_pins, delta_time),
        IgniterState::Firing => IgniterFSM::<Firing>::update(ecu_state, ecu_pins),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::update(ecu_state, ecu_pins),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn on_packet(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins, packet: &Packet) {
    let transition = match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::Startup => {
            IgniterFSM::<Startup>::on_packet(ecu_state, ecu_pins, packet)
        }
        IgniterState::Firing => IgniterFSM::<Firing>::on_packet(ecu_state, ecu_pins, packet),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::on_packet(ecu_state, ecu_pins, packet),
    };

    if let Some(new_state) = transition {
        transition_state(ecu_state, ecu_pins, new_state);
    }
}

pub fn transition_state(
    ecu_state: &mut ECUState,
    ecu_pins: &mut ECUControlPins,
    new_state: IgniterState,
) {
    if ecu_state.igniter_state == new_state {
        return;
    }

    ecu_state.igniter_state = new_state;
    ecu_state
        .igniter_state_storage
        .elapsed_since_state_transition = 0.0;

    match ecu_state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::enter_state(ecu_state, ecu_pins),
        IgniterState::Startup => IgniterFSM::<Startup>::enter_state(ecu_state, ecu_pins),
        IgniterState::Firing => IgniterFSM::<Firing>::enter_state(ecu_state, ecu_pins),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::enter_state(ecu_state, ecu_pins),
    }
}

impl IgniterStateStorage {
    pub const fn default() -> Self {
        Self {
            elapsed_since_state_transition: 0.0,
            stable_pressure_counter: 0.0,
        }
    }
}
