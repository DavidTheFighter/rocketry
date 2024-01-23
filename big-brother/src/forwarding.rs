use crate::{
    big_brother::{BigBrotherEndpoint, BigBrotherError, Broadcastable, UDP_PORT},
    BigBrother,
};

impl<'a, 'b, const NETWORK_MAP_SIZE: usize, P, A> BigBrother<'a, NETWORK_MAP_SIZE, P, A>
where
    A: Copy + PartialEq + Eq + Broadcastable + core::fmt::Debug,
{
    pub fn try_forward_udp(
        &mut self,
        source_interface_index: u8,
        remote: &BigBrotherEndpoint,
        destination: A,
        buffer_size: usize,
    ) -> Result<(), BigBrotherError> {
        if destination == self.host_addr {
            return Ok(());
        }

        if destination.is_broadcast() {
            // Rebroadcast to all interfaces except the one we received it on
            for (interface_index, interface) in self.interfaces.iter_mut().enumerate() {
                if let Some(interface) = interface {
                    if interface_index == source_interface_index as usize {
                        continue;
                    }

                    let destination_endpoint = BigBrotherEndpoint {
                        ip: interface.broadcast_ip(),
                        port: UDP_PORT,
                    };

                    // println!("BForwarding from i{}@{:?}:{} to i{}@{:?}:{}", source_interface_index, remote.ip, remote.port, interface_index, destination_endpoint.ip, destination_endpoint.port);
                    interface.send_udp(
                        destination_endpoint.clone(),
                        &mut self.working_buffer[..buffer_size],
                    )?;
                }
            }

            // Rebroadcast to any upstream local ports
            for port in self.network_map.get_upstream_local_ports() {
                if *port == remote.port {
                    continue;
                }

                let destination_endpoint = BigBrotherEndpoint {
                    ip: [127, 0, 0, 1],
                    port: *port,
                };

                for (i, interface) in self.interfaces.iter_mut().enumerate() {
                    if let Some(interface) = interface {
                        // println!("Upforwarding from i{}@{:?}:{} to i{}@{:?}:{}", source_interface_index, remote.ip, remote.port, i, destination_endpoint.ip, destination_endpoint.port);
                        interface.send_udp(
                            destination_endpoint.clone(),
                            &mut self.working_buffer[..buffer_size],
                        )?;
                    }
                }
            }
        } else if let Ok(network_mapping) = self.network_map.get_address_mapping(destination) {
            let destination_endpoint = BigBrotherEndpoint {
                ip: network_mapping.ip,
                port: network_mapping.port,
            };

            if let Some(interface) =
                self.interfaces[network_mapping.interface_index as usize].as_mut()
            {
                // println!("Forwarding(i{}->i{}) from i{}@{:?}:{} to {:?}:{}", source_interface_index, network_mapping.interface_index, source_interface_index, remote.ip, remote.port, destination_endpoint.ip, destination_endpoint.port);
                interface.send_udp(
                    destination_endpoint,
                    &mut self.working_buffer[..buffer_size],
                )?;
            } else {
                return Err(BigBrotherError::SendUnnaddressable);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
