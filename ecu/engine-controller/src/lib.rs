#![cfg_attr(not(test), no_std)]

pub(crate) mod fuel_tank_fsm;
pub(crate) mod igniter_fsm;

use hal::{
    comms_hal::{Packet, NetworkAddress},
    ecu_hal::{EcuDriver, FuelTankState, IgniterConfig, IgniterState},
};

pub struct Ecu<'a, D> {
    pub igniter_config: IgniterConfig,
    pub igniter_state: IgniterState,
    pub(crate) igniter_fsm_storage: igniter_fsm::FsmStorage,
    pub fuel_tank_state: FuelTankState,
    pub(crate) driver: &'a mut D,
}

impl<'a, D> Ecu<'a, D> where D: EcuDriver {
    pub fn new(driver: &'a mut D) -> Self {
        let mut ecu = Self {
            igniter_config: IgniterConfig::default(),
            igniter_state: IgniterState::Idle,
            igniter_fsm_storage: igniter_fsm::FsmStorage::Empty,
            fuel_tank_state: FuelTankState::Idle,
            driver,
        };

        ecu.init_igniter_fsm();
        ecu.init_fuel_tank_fsm();

        ecu
    }

    pub fn update(&mut self, dt: f32, packet: Option<Packet>) {
        if let Some(packet) = &packet {
            match packet {
                Packet::ConfigureSensor { sensor, config } => {
                    self.driver.configure_sensor(*sensor, *config);
                },
                Packet::SetSolenoidValve { valve, state } => {
                    self.driver.set_solenoid_valve(*valve, *state);
                },
                Packet::SetSparking(state) => {
                    self.driver.set_sparking(*state);
                },
                _ => {}
            }
        }

        self.update_igniter_fsm(dt, &packet);
        self.update_fuel_tank_fsm(dt, &packet);

        let mut telem_frame = self.driver.generate_telemetry_frame();
        telem_frame.igniter_state = self.igniter_state;
        telem_frame.fuel_tank_state = self.fuel_tank_state;

        self.driver.send_packet(
            Packet::EcuTelemetry(telem_frame),
            NetworkAddress::MissionControl,
        );
    }

    pub fn driver(&mut self) -> &mut D {
        self.driver
    }
}

pub(crate) trait FiniteStateMachine<D> {
    fn update<F: EcuDriver>(ecu: &mut Ecu<F>, dt: f32, packet: &Option<Packet>) -> Option<D>;
    fn setup_state<F: EcuDriver>(ecu: &mut Ecu<F>);
}
