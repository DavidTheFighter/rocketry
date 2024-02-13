use crate::Ecu;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{EcuCommand, EcuBinaryOutput},
    ControllerState,
};

use super::{new_state_from_command, TankFsm, TankType};

#[derive(Debug)]
pub struct Depressurized {
    tank_type: TankType,
    press_valve: EcuBinaryOutput,
    vent_valve: EcuBinaryOutput,
}

impl<'f> ControllerState<TankFsm, Ecu<'f>> for Depressurized {
    fn update<'a>(
        &mut self,
        _ecu: &mut Ecu,
        _dt: f32,
        packets: &[(NetworkAddress, Packet)],
    ) -> Option<TankFsm> {
        if let Some(new_state) = self.should_transition_state(packets) {
            return Some(new_state);
        }

        None
    }

    fn enter_state(&mut self, ecu: &mut Ecu) {
        ecu.driver.set_binary_valve(self.press_valve, false);
        ecu.driver.set_binary_valve(self.vent_valve, true);
    }

    fn exit_state(&mut self, _ecu: &mut Ecu) {
        // Nothing
    }
}

impl Depressurized {
    pub fn new(
        tank_type: TankType,
        press_valve: EcuBinaryOutput,
        vent_valve: EcuBinaryOutput,
    ) -> TankFsm {
        TankFsm::Depressurized(Self {
            tank_type,
            press_valve,
            vent_valve,
        })
    }

    fn should_transition_state(&self, packets: &[(NetworkAddress, Packet)]) -> Option<TankFsm> {
        for (_address, packet) in packets {
            if let Packet::EcuCommand(command) = packet {
                if let EcuCommand::SetFuelTank(new_state) = command {
                    return Some(new_state_from_command(
                        *new_state,
                        self.tank_type,
                        self.press_valve,
                        self.vent_valve,
                    ));
                }
            }
        }

        None
    }
}
