use hal::{
    comms_hal::Packet,
    ecu_hal::{EcuDriver, EcuSensor, IgniterState},
};

use crate::{Ecu, FiniteStateMachine};

mod firing;
mod idle;
mod shutdown;
mod startup;

pub struct Idle;
pub struct Startup {
    startup_elapsed_time: f32,
    stable_pressure_time: f32,
}
pub struct Firing {
    elapsed_time: f32,
}
pub struct Shutdown {
    elapsed_time: f32,
}

pub enum FsmStorage {
    Empty,
    Idle(Idle),
    Startup(Startup),
    Firing(Firing),
    Shutdown(Shutdown),
}

pub const IGNITER_SENSORS: [EcuSensor; 4] = [
    EcuSensor::IgniterFuelInjectorPressure,
    EcuSensor::IgniterGOxInjectorPressure,
    EcuSensor::IgniterChamberPressure,
    EcuSensor::IgniterThroatTemp,
];

impl<'a> Ecu<'a> {
    pub fn update_igniter_fsm(&mut self, dt: f32, packet: Option<Packet>) {
        let new_state = match self.igniter_state {
            IgniterState::Idle => Idle::update(self, dt, packet),
            IgniterState::Startup => Startup::update(self, dt, packet),
            IgniterState::Firing => Firing::update(self, dt, packet),
            IgniterState::Shutdown => Shutdown::update(self, dt, packet),
        };

        if let Some(new_state) = new_state {
            self.transition_igniter_state(new_state);
        }
    }

    fn transition_igniter_state(&mut self, new_state: IgniterState) {
        self.igniter_state = new_state;

        match new_state {
            IgniterState::Idle => Idle::setup_state(self),
            IgniterState::Startup => Startup::setup_state(self),
            IgniterState::Firing => Firing::setup_state(self),
            IgniterState::Shutdown => Shutdown::setup_state(self),
        }
    }

    pub fn init_igniter_fsm(&mut self) {
        self.transition_igniter_state(IgniterState::Idle);
    }
}

fn reset_igniter_daq_collections(driver: &mut dyn EcuDriver) {
    for sensor in IGNITER_SENSORS {
        driver.collect_daq_sensor_data(sensor);
    }
}

#[cfg(test)]
mod tests {
    use hal::ecu_mock::EcuDriverMock;

    use super::*;
    use strum::IntoEnumIterator;

    // Ensure that each state transition sets up its state storage
    #[test]
    fn test_state_storage_setup() {
        let mut driver = EcuDriverMock::new();
        let mut ecu = Ecu::new(&mut driver);

        for state in IgniterState::iter() {
            ecu.igniter_fsm_storage = FsmStorage::Empty;

            ecu.transition_igniter_state(state);

            if let FsmStorage::Empty = ecu.igniter_fsm_storage {
                panic!("State storage not setup for state {:?}", state);
            }
        }
    }

    // Ensure that each state transition resets the DAQ collections
    #[test]
    fn test_state_daq_reset() {
        let sensor_min = 0_f32;
        let sensor_max = 10_f32;
        let sensor_current = 5_f32;

        for state in IgniterState::iter() {
            println!("Testing state {:?}", state);
            let mut driver = EcuDriverMock::new();
            let mut ecu = Ecu::new(&mut driver);

            // Update sensor values to create stored mins/maxs
            for sensor in IGNITER_SENSORS {
                let driver: &mut EcuDriverMock = ecu.driver.as_mut_any().downcast_mut().unwrap();

                driver.update_sensor_value(sensor, sensor_min);
                driver.update_sensor_value(sensor, sensor_max);
                driver.update_sensor_value(sensor, sensor_current);

                let dummy_collection = (sensor_current, sensor_min, sensor_max);
                let daq_collection = driver.get_daq_sensor_collection(sensor);

                assert_eq!(
                    daq_collection, dummy_collection,
                    "Mock DAQ collection not setup correctly for state {:?}", state,
                );
            }

            ecu.transition_igniter_state(state);

            for sensor in IGNITER_SENSORS {
                let driver: &mut EcuDriverMock = ecu.driver.as_mut_any().downcast_mut().unwrap();

                let dummy_collection = (sensor_current, sensor_current, sensor_current);
                let daq_collection = driver.get_daq_sensor_collection(sensor);

                assert_eq!(daq_collection, dummy_collection);
            }
        }
    }

    // Ensure the Igniter FSM init function is being called at startup
    #[test]
    fn test_fsm_init() {
        let mut driver = EcuDriverMock::new();
        let ecu = Ecu::new(&mut driver);

        if let FsmStorage::Empty = ecu.igniter_fsm_storage {
            panic!("Igniter FSM init function isn't being called at startup");
        }
    }
}
