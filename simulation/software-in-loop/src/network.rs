use std::sync::{Arc, Mutex};

use big_brother::interface::{
    mock_interface::MockInterface,
    mock_topology::{MockPhysicalInterface, MockPhysicalNet},
    std_interface::StdInterface,
};
use pyo3::{prelude::*, types::PyList};

#[pyclass]
pub struct SilNetwork {
    pub(crate) network: Arc<Mutex<MockPhysicalNet>>,
}

#[pyclass]
pub struct SilNetworkPhy {
    pub(crate) phy: Arc<Mutex<MockPhysicalInterface>>,
}

#[pyclass]
pub struct SilNetworkIface {
    pub(crate) iface: Option<MockInterface>,
}

#[pyclass]
pub struct SimBridgeIface {
    pub(crate) iface: Option<StdInterface>,
}

#[pymethods]
impl SilNetwork {
    #[new]
    pub fn new(subnet_ip: &PyList) -> Self {
        let subnet_ip: [u8; 4] = subnet_ip
            .extract()
            .expect("Failed to extract subnet IP from Python list");

        let mut subnet_mask = [false; 4];
        for i in 0..4 {
            subnet_mask[i] = subnet_ip[i] != 0;
        }

        let broadcast_ip = [
            subnet_ip[0] | (!subnet_mask[0] as u8),
            subnet_ip[1] | (!subnet_mask[1] as u8),
            subnet_ip[2] | (!subnet_mask[2] as u8),
            subnet_ip[3] | (!subnet_mask[3] as u8),
        ];

        let mut network = MockPhysicalNet::new(subnet_ip, subnet_mask, broadcast_ip);
        network.enable_payload_logging();

        Self {
            network: Arc::new(Mutex::new(network)),
        }
    }
}

#[pymethods]
impl SilNetworkPhy {
    #[new]
    pub fn new(network: PyRef<'_, SilNetwork>) -> Self {
        Self {
            phy: Arc::new(Mutex::new(MockPhysicalInterface::new(
                network.network.clone(),
            ))),
        }
    }
}

#[pymethods]
impl SilNetworkIface {
    #[new]
    pub fn new(phy: PyRef<'_, SilNetworkPhy>) -> Self {
        Self {
            iface: Some(MockInterface::new_networked(phy.phy.clone())),
        }
    }
}

#[pymethods]
impl SimBridgeIface {
    #[new]
    pub fn new() -> Self {
        Self {
            iface: Some(StdInterface::new([127, 0, 0, 1]).expect("Failed to create std interface")),
        }
    }
}

impl SilNetworkIface {
    pub fn take(&mut self) -> Option<MockInterface> {
        self.iface.take()
    }
}
