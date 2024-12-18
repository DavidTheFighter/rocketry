use std::{cell::RefCell, net::UdpSocket, rc::Rc};

use big_brother::{
    big_brother::{BigBrotherError, MAX_INTERFACE_COUNT, WORKING_BUFFER_SIZE},
    interface::{bridge_interface::BridgeInterface, mock_interface::MockInterface, BigBrotherInterface},
};
use fcu_rs::FcuBigBrother;
use pyo3::{prelude::*, types::PyList};
use shared::{
    comms_hal::{NetworkAddress, Packet}, ecu_hal, fcu_hal, REALTIME_SIMULATION_CTRL_PORT, REALTIME_SIMULATION_SIM_PORT
};

use crate::network::{SilNetworkIface, SimBridgeIface};

#[pyclass(unsendable)]
pub struct MissionControl {
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<FcuBigBrother<'static>>>,
    _simulation_bridge_iface: Option<Rc<RefCell<BridgeInterface>>>,
    time_since_last_1ms: f32,
    timestamp: f32,
}

#[pymethods]
impl MissionControl {
    #[new]
    pub fn new(py: Python, network_ifaces: &PyList, realtime: Option<bool>) -> Self {
        let mut big_brother_ifaces = [None, None];
        let mut big_brother_ifaces_ref: [Option<&'static mut dyn BigBrotherInterface>;
            MAX_INTERFACE_COUNT] = [None, None];
        let mut simulation_bridge_iface = None;

        for (i, sil_iface) in network_ifaces.iter().enumerate().take(2) {
            let mut sil_iface = sil_iface
                .extract::<PyRefMut<'_, SilNetworkIface>>()
                .expect("Failed to extract interface");

            big_brother_ifaces[i].replace(Rc::new(RefCell::new(
                sil_iface.iface.take().expect("Failed to take interface"),
            )));

            let bb_iface_ref: &'static mut MockInterface = unsafe {
                std::mem::transmute(&mut *big_brother_ifaces[i].as_ref().unwrap().borrow_mut())
            };

            big_brother_ifaces_ref[i] = Some(bb_iface_ref);
        }

        if realtime.unwrap_or(false) {
            if network_ifaces.len() == 2 {
                panic!("Cannot have both network interfaces and simulation bridge");
            }

            println!("Doing simulation bridge");

            let simulation_interface = BridgeInterface::new(REALTIME_SIMULATION_SIM_PORT, REALTIME_SIMULATION_CTRL_PORT)
                .expect("Failed to create simulation interface for comms thread");

            simulation_bridge_iface.replace(Rc::new(RefCell::new(simulation_interface)));

            let bb_iface_ref: &'static mut BridgeInterface = unsafe {
                std::mem::transmute(&mut *simulation_bridge_iface.as_ref().unwrap().borrow_mut())
            };

            big_brother_ifaces_ref[1] = Some(bb_iface_ref);
        }

        let network_address = if realtime.unwrap_or(false) {
            NetworkAddress::MissionControlSimBridge
        } else {
            NetworkAddress::MissionControl
        };

        println!("MissionControl: {:?}", network_address);

        let big_brother = Rc::new(RefCell::new(FcuBigBrother::new(
            network_address,
            rand::random(),
            NetworkAddress::Broadcast,
            big_brother_ifaces_ref,
        )));

        Self {
            _big_brother_ifaces: big_brother_ifaces,
            _big_brother: big_brother,
            _simulation_bridge_iface: simulation_bridge_iface,
            time_since_last_1ms: 0.0,
            timestamp: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut comms = self._big_brother.borrow_mut();

        if self.time_since_last_1ms >= 0.001 {
            self.time_since_last_1ms -= 0.001;
            comms.poll_1ms((self.timestamp * 1e3) as u32);
        }

        self.timestamp += dt;
        self.time_since_last_1ms += dt;

        loop {
            if let Ok(packet) = comms.recv_packet_raw() {
                if packet.is_none() {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn post_update(&mut self) {}

    pub fn send_arm_vehicle_packet(&mut self) {
        let command = fcu_hal::VehicleCommand::Arm {
            magic_number: fcu_hal::ARMING_MAGIC_NUMBER,
        };
        let packet = Packet::VehicleCommand(command);
        self.send_packet(&packet, NetworkAddress::FlightController);
    }

    pub fn send_ignite_solid_motor_packet(&mut self) {
        let command = fcu_hal::VehicleCommand::IgniteSolidMotor {
            magic_number: fcu_hal::IGNITION_MAGIC_NUMBER,
        };
        let packet = Packet::VehicleCommand(command);
        self.send_packet(&packet, NetworkAddress::FlightController);
    }

    pub fn send_set_fuel_tank_packet(&mut self, ecu_index: u8, pressurized: bool) {
        let command = match pressurized {
            true => ecu_hal::EcuCommand::SetTankState((ecu_hal::TankType::FuelMain, ecu_hal::TankState::Pressurized)),
            false => ecu_hal::EcuCommand::SetTankState((ecu_hal::TankType::FuelMain, ecu_hal::TankState::Venting)),
        };

        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }

    pub fn send_set_oxidizer_tank_packet(&mut self, ecu_index: u8, pressurized: bool) {
        let command = match pressurized {
            true => ecu_hal::EcuCommand::SetTankState((ecu_hal::TankType::OxidizerMain, ecu_hal::TankState::Pressurized)),
            false => ecu_hal::EcuCommand::SetTankState((ecu_hal::TankType::OxidizerMain, ecu_hal::TankState::Venting)),
        };

        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }

    pub fn send_fire_engine_packet(&mut self, ecu_index: u8) {
        let command = ecu_hal::EcuCommand::FireEngine;
        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }

    pub fn send_fire_igniter_packet(&mut self, ecu_index: u8) {
        let command = ecu_hal::EcuCommand::FireIgniter;
        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }

    pub fn send_set_fuel_pump_packet(&mut self, ecu_index: u8, duty: f32) {
        let command = ecu_hal::EcuCommand::SetPumpDuty((ecu_hal::PumpType::FuelMain, duty));
        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }

    pub fn send_set_oxidizer_pump_packet(&mut self, ecu_index: u8, duty: f32) {
        let command = ecu_hal::EcuCommand::SetPumpDuty((ecu_hal::PumpType::OxidizerMain, duty));
        let packet = Packet::EcuCommand(command);
        self.send_packet(&packet, NetworkAddress::EngineController(ecu_index));
    }
}

impl MissionControl {
    fn send_packet(&mut self, packet: &Packet, destination: NetworkAddress) {
        if let Err(e) = self
            ._big_brother
            .borrow_mut()
            .send_packet(&packet, destination)
        {
            if let BigBrotherError::UnknownNetworkAddress = e {
                eprintln!("mission_ctrl.rs: Unknown network address");
            } else {
                eprintln!("mission_ctrl.rs: Failed to send packet: {:?}", e);
            }
        }
    }
}
