use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuAlert, EcuBinaryOutput, EcuCommand, IgniterConfig, TankState},
    ControllerState,
};

use crate::Ecu;

use super::{startup::Startup, IgniterFsm};

pub struct Idle {
    igniter_config: IgniterConfig,
}

impl<'f> ControllerState<IgniterFsm, Ecu<'f>> for Idle {
    fn update<'a>(
        &mut self,
        ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<IgniterFsm> {
        if self.received_fire_igniter(packets) {
            if self.tanks_pressurized(ecu) {
                ecu.alert_manager
                    .clear_condition(EcuAlert::IgniterTankOffNominal);

                return Some(Startup::new(self.igniter_config.clone()));
            } else {
                ecu.alert_manager
                    .set_condition(EcuAlert::IgniterTankOffNominal);
            }
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::IgniterFuelValve, false);
        ecu.driver
            .set_binary_valve(EcuBinaryOutput::IgniterOxidizerValve, false);
        ecu.driver.set_sparking(false);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Idle {
    pub fn new(igniter_config: IgniterConfig) -> IgniterFsm {
        IgniterFsm::Idle(Self { igniter_config })
    }

    fn received_fire_igniter(&self, packets: &[(NetworkAddress, Packet)]) -> bool {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::FireIgniter = command {
                    return true;
                }
            }
        }

        false
    }

    fn tanks_pressurized(&self, ecu: &Ecu) -> bool {
        let fuel_tank_pressurized = ecu
            .fuel_tank_state()
            .map_or(true, |state| state == TankState::Pressurized);

        let oxidizer_tank_pressurized = ecu
            .oxidizer_tank_state()
            .map_or(true, |state| state == TankState::Pressurized);

        fuel_tank_pressurized && oxidizer_tank_pressurized
    }
}
