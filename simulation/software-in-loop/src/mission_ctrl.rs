use std::{cell::RefCell, rc::Rc};

use big_brother::{
    big_brother::MAX_INTERFACE_COUNT,
    interface::{
        bridge_interface::BridgeInterface, mock_interface::MockInterface, BigBrotherInterface,
    },
};
use fcu_rs::FcuBigBrother;
use mission_ctrl_api::CommandHandler;
use pyo3::{prelude::*, types::PyList};
use shared::{
    comms_hal::NetworkAddress,
    ecu_hal::TankType, REALTIME_SIMULATION_CTRL_PORT, REALTIME_SIMULATION_SIM_PORT,
};

use crate::network::SilNetworkIface;

#[pyclass(unsendable)]
pub struct MissionControl {
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<FcuBigBrother<'static>>>,
    _simulation_bridge_iface: Option<Rc<RefCell<BridgeInterface>>>,
    time_since_last_1ms: f32,
    timestamp: f32,

    #[pyo3(get)]
    pub command_handler: Py<CommandHandler>,
    #[pyo3(get)]
    pub fuel_tank: Py<mission_ctrl_api::tank::Tank>,
    #[pyo3(get)]
    pub oxidizer_tank: Py<mission_ctrl_api::tank::Tank>,
    #[pyo3(get)]
    pub igniter: Py<mission_ctrl_api::igniter::Igniter>,
    #[pyo3(get)]
    pub engine: Py<mission_ctrl_api::engine::Engine>,
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

            let simulation_interface =
                BridgeInterface::new(REALTIME_SIMULATION_SIM_PORT, REALTIME_SIMULATION_CTRL_PORT)
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

        let command_handler = Py::new(py, CommandHandler::from_big_brother(big_brother.clone())).unwrap();

        Self {
            _big_brother_ifaces: big_brother_ifaces,
            _big_brother: big_brother.clone(),
            _simulation_bridge_iface: simulation_bridge_iface,
            time_since_last_1ms: 0.0,
            timestamp: 0.0,
            command_handler: command_handler.clone(),
            fuel_tank: Py::new(py, mission_ctrl_api::tank::Tank::new(
                format!("{:?}", TankType::FuelMain),
                0,
                command_handler.clone(),
            )).unwrap(),
            oxidizer_tank: Py::new(py, mission_ctrl_api::tank::Tank::new(
                format!("{:?}", TankType::OxidizerMain),
                0,
                command_handler.clone(),
            )).unwrap(),
            igniter: Py::new(py, mission_ctrl_api::igniter::Igniter::new(
                0,
                command_handler.clone(),
            )).unwrap(),
            engine: Py::new(py, mission_ctrl_api::engine::Engine::new(
                0,
                command_handler.clone(),
            )).unwrap(),
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
}

