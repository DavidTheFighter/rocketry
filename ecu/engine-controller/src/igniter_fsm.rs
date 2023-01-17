use core::marker::PhantomData;

use enum_dispatch::enum_dispatch;
use hal::{ecu_hal::{EcuDriver, IgniterState, ECUSolenoidValve, FuelTankState}, comms_hal::Packet};

use crate::EcuState;

struct IgniterFSM<T> {
    _m: PhantomData<T>,
}

struct Idle;
struct Startup;
struct Firing;
struct Shutdown;

impl IgniterFSM<Idle> {
    fn update(_state: &mut EcuState, _driver: &mut dyn EcuDriver, delta_time: f32) -> Option<IgniterState> {
        None
    }

    fn enter_state(_state: &mut EcuState, driver: &mut dyn EcuDriver) {
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);
    }

    fn on_packet(state: &mut EcuState, _driver: &mut dyn EcuDriver, packet: &Packet) -> Option<IgniterState> {
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

impl IgniterFSM<Startup> {
    fn update(_state: &mut EcuState, _driver: &mut dyn EcuDriver, delta_time: f32) -> Option<IgniterState> {
        None
    }

    fn enter_state(_state: &mut EcuState, driver: &mut dyn EcuDriver) {
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);
    }

    fn on_packet(state: &mut EcuState, _driver: &mut dyn EcuDriver, packet: &Packet) -> Option<IgniterState> {
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

impl IgniterFSM<Firing> {
    fn update(_state: &mut EcuState, _driver: &mut dyn EcuDriver, delta_time: f32) -> Option<IgniterState> {
        None
    }

    fn enter_state(_state: &mut EcuState, driver: &mut dyn EcuDriver) {
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);
    }

    fn on_packet(state: &mut EcuState, _driver: &mut dyn EcuDriver, packet: &Packet) -> Option<IgniterState> {
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

impl IgniterFSM<Shutdown> {
    fn update(_state: &mut EcuState, _driver: &mut dyn EcuDriver, delta_time: f32) -> Option<IgniterState> {
        None
    }

    fn enter_state(_state: &mut EcuState, driver: &mut dyn EcuDriver) {
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterFuelMain, false);
        driver.set_solenoid_valve(ECUSolenoidValve::IgniterGOxMain, false);
        driver.set_sparking(false);
    }

    fn on_packet(state: &mut EcuState, _driver: &mut dyn EcuDriver, packet: &Packet) -> Option<IgniterState> {
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

// #[enum_dispatch(IgniterFsmState)]
// trait IgniterFsm {
//     fn update(&self, state: &mut EcuState, driver: &mut dyn EcuDriver) -> Option<IgniterState>;
//     fn enter_state(&self, state: &mut EcuState, driver: &mut dyn EcuDriver);
//     fn on_packet(&self, state: &mut EcuState, driver: &mut dyn EcuDriver, packet: &Packet) -> Option<IgniterState>;
// }

// #[enum_dispatch]
// enum IgniterFsmState {
//     Idle,
// }

pub fn update(state: &mut EcuState, driver: &mut dyn EcuDriver, delta_time: f32) {
    let transition = match state.igniter_state {
        IgniterState::Idle => IgniterFSM::<Idle>::update(state, driver, delta_time),
        IgniterState::Startup => IgniterFSM::<Startup>::update(state, driver, delta_time),
        IgniterState::Firing => IgniterFSM::<Firing>::update(state, driver, delta_time),
        IgniterState::Shutdown => IgniterFSM::<Shutdown>::update(state, driver, delta_time),
    };

    
}

pub fn on_packet(state: &mut EcuState, driver: &mut dyn EcuDriver, packet: Packet) {
    
}