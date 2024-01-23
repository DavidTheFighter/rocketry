use big_brother::BigBrother;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{
        EcuConfig, EcuDriver, EcuSolenoidValve, EcuTelemetryFrame, EngineState, IgniterState,
        TankState,
    },
    ControllerEntity, COMMS_NETWORK_MAP_SIZE,
};

use crate::{
    silprintln,
    tank_fsm::{idle, TankFsm, TankType},
};

pub const PACKET_QUEUE_SIZE: usize = 16;

pub type EcuBigBrother<'a> = BigBrother<'a, COMMS_NETWORK_MAP_SIZE, Packet, NetworkAddress>;

pub struct Ecu<'a> {
    pub config: EcuConfig,
    pub driver: &'a mut dyn EcuDriver,
    pub comms: &'a mut EcuBigBrother<'a>,
    pub fuel_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,
    pub oxidizer_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,
    time_since_last_telemetry: f32,
}

impl<'a> Ecu<'a> {
    pub fn new(driver: &'a mut dyn EcuDriver, comms: &'a mut EcuBigBrother<'a>) -> Self {
        let mut ecu = Self {
            config: EcuConfig::default(),
            driver,
            comms,
            fuel_tank: None,
            oxidizer_tank: None,
            time_since_last_telemetry: 0.0,
        };

        ecu.fuel_tank = Some(ControllerEntity::new(
            &mut ecu,
            idle::Idle::new(
                TankType::Fuel,
                EcuSolenoidValve::FuelPress,
                EcuSolenoidValve::FuelVent,
            ),
        ));
        ecu.oxidizer_tank = Some(ControllerEntity::new(
            &mut ecu,
            idle::Idle::new(
                TankType::Oxidizer,
                EcuSolenoidValve::OxidizerPress,
                EcuSolenoidValve::OxidizerVent,
            ),
        ));

        ecu
    }

    pub fn update(&mut self, dt: f32) {
        let mut packets = empty_packet_array();
        let mut num_packets = 0;
        while let Some((packet, source)) = self.comms.recv_packet().ok().flatten() {
            silprintln!("ECU: From {:?} received packet: {:?}", source, packet);
            packets[num_packets] = (source, packet);
            num_packets += 1;
        }
        let packets = &packets[..num_packets];

        if let Some(mut fuel_tank) = self.fuel_tank.take() {
            fuel_tank.update(self, dt, packets);
            self.fuel_tank = Some(fuel_tank);
        }

        if let Some(mut oxidizer_tank) = self.oxidizer_tank.take() {
            oxidizer_tank.update(self, dt, packets);
            self.oxidizer_tank = Some(oxidizer_tank);
        }

        self.time_since_last_telemetry += dt;
        if self.time_since_last_telemetry >= self.config.telemetry_rate_s {
            self.time_since_last_telemetry = 0.0;
            let telemetry_frame = self.generate_telemetry_frame();
            self.send_packet(
                &Packet::EcuTelemetry(telemetry_frame),
                NetworkAddress::MissionControl,
            );
        }
    }

    pub(crate) fn send_packet(&mut self, packet: &Packet, destination: NetworkAddress) {
        let _ = self.comms.send_packet(packet, destination);
    }

    pub fn generate_telemetry_frame(&self) -> EcuTelemetryFrame {
        EcuTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            engine_state: EngineState::Idle,
            igniter_state: IgniterState::Idle,
            fuel_tank_state: self.fuel_tank_state().unwrap_or(TankState::Idle),
            oxidizer_tank_state: self.oxidizer_tank_state().unwrap_or(TankState::Idle),
        }
    }

    pub(crate) fn fuel_tank_state(&self) -> Option<TankState> {
        self.fuel_tank.as_ref().map(|fsm| fsm.hal_state())
    }

    pub(crate) fn oxidizer_tank_state(&self) -> Option<TankState> {
        self.oxidizer_tank.as_ref().map(|fsm| fsm.hal_state())
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Ecu<'_> {}

fn empty_packet_array() -> [(NetworkAddress, Packet); PACKET_QUEUE_SIZE] {
    [
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
        (NetworkAddress::Unknown, Packet::DoNothing),
    ]
}
