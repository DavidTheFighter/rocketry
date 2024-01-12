
use std::{rc::Rc, cell::RefCell};

use big_brother::{interface::{mock_interface::MockInterface, BigBrotherInterface}, big_brother::MAX_INTERFACE_COUNT};
use flight_controller_rs::FcuBigBrother;
use pyo3::{prelude::*, types::PyList};
use shared::{comms_hal::{NetworkAddress, Packet}, fcu_hal};

use crate::network::SilNetworkIface;

#[pyclass(unsendable)]
pub struct MissionControl {
    pub(crate) _big_brother_ifaces: [Option<Rc<RefCell<MockInterface>>>; 2],
    pub(crate) _big_brother: Rc<RefCell<FcuBigBrother<'static>>>,
    time_since_last_1ms: f32,
    timestamp: f32,
}

#[pymethods]
impl MissionControl {
    #[new]
    pub fn new(network_ifaces: &PyList) -> Self {
        let mut big_brother_ifaces = [None, None];
        let mut big_brother_ifaces_ref: [Option<&'static mut dyn BigBrotherInterface>; MAX_INTERFACE_COUNT] = [None, None];

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

        let big_brother = Rc::new(RefCell::new(FcuBigBrother::new(
            NetworkAddress::MissionControl,
            rand::random(),
            NetworkAddress::Broadcast,
            big_brother_ifaces_ref,
        )));

        Self {
            _big_brother_ifaces: big_brother_ifaces,
            _big_brother: big_brother,
            time_since_last_1ms: 0.0,
            timestamp: 0.0,
        }
    }

    pub fn update_timestep(&mut self, dt: f32) {
        let mut comms = self._big_brother.borrow_mut();

        if self.time_since_last_1ms >= 0.001 {
            self.time_since_last_1ms -= 0.001;
            comms.poll_1ms((self.timestamp * 1e3) as u32);
        }

        self.timestamp += dt;
        self.time_since_last_1ms += dt;

        loop {
            if let Ok(packet) = comms.recv_packet() {
                if packet.is_none() {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn send_arm_vehicle_packet(&mut self) {
        let packet = Packet::ArmVehicle { magic_number: fcu_hal::ARMING_MAGIC_NUMBER };
        self._big_brother.borrow_mut().send_packet(&packet, NetworkAddress::FlightController).unwrap();
    }

    pub fn send_ignite_solid_motor_packet(&mut self) {
        let packet = Packet::IgniteSolidMotor { magic_number: fcu_hal::IGNITION_MAGIC_NUMBER };
        self._big_brother.borrow_mut().send_packet(&packet, NetworkAddress::FlightController).unwrap();
    }
}
