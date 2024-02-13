use big_brother::BigBrother;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{
        EcuBinaryOutput, EcuConfig, EcuDebugInfoVariant, EcuDriver, EcuSensor, EcuTankTelemetryFrame, EcuTelemetryFrame, EngineState, IgniterState, TankState
    },
    ControllerEntity, COMMS_NETWORK_MAP_SIZE,
};

use crate::{
    silprintln,
    tank_fsm::{TankFsm, TankType, self}, igniter_fsm::{IgniterFsm, self}, engine_fsm::{EngineFsm, self},
};

use strum::IntoEnumIterator;

pub const PACKET_QUEUE_SIZE: usize = 16;

pub type EcuBigBrother<'a> = BigBrother<'a, COMMS_NETWORK_MAP_SIZE, Packet, NetworkAddress>;

pub struct Ecu<'a> {
    pub config: EcuConfig,
    pub debug_info_enabled: bool,
    pub driver: &'a mut dyn EcuDriver,
    pub comms: &'a mut EcuBigBrother<'a>,
    pub engine: Option<ControllerEntity<EngineFsm, Ecu<'a>, EngineState>>,
    pub igniter: Option<ControllerEntity<IgniterFsm, Ecu<'a>, IgniterState>>,
    pub fuel_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,
    pub oxidizer_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,

    pub fuel_tank_pressure_pa: f32,
    pub oxidizer_tank_pressure_pa: f32,
    pub igniter_chamber_pressure_pa: f32,

    pub last_telemetry_frame: Option<EcuTelemetryFrame>,
    time_since_last_telemetry: f32,
}

impl<'a> Ecu<'a> {
    pub fn new(driver: &'a mut dyn EcuDriver, comms: &'a mut EcuBigBrother<'a>) -> Self {
        let mut ecu = Self {
            config: EcuConfig::default(),
            debug_info_enabled: true,
            driver,
            comms,
            engine: None,
            igniter: None,
            fuel_tank: None,
            oxidizer_tank: None,
            fuel_tank_pressure_pa: 0.0,
            oxidizer_tank_pressure_pa: 0.0,
            igniter_chamber_pressure_pa: 0.0,
            last_telemetry_frame: None,
            time_since_last_telemetry: 1e3,
        };

        ecu.engine = Some(ControllerEntity::new(
            &mut ecu,
            engine_fsm::idle::Idle::new(),
        ));

        ecu.igniter = Some(ControllerEntity::new(
            &mut ecu,
            igniter_fsm::idle::Idle::new(),
        ));

        ecu
    }

    pub fn update(&mut self, dt: f32) {
        self.comms.poll_1ms((self.driver.timestamp() * 1e3) as u32);

        let mut packets = empty_packet_array();
        let mut num_packets = 0;
        while let Some((packet, source)) = self.comms.recv_packet().ok().flatten() {
            silprintln!("ECU: From {:?} received packet: {:?}", source, packet);
            packets[num_packets] = (source, packet);
            num_packets += 1;
        }
        let packets = &packets[..num_packets];

        if let Some(mut igniter) = self.igniter.take() {
            igniter.update(self, dt, packets);
            self.igniter = Some(igniter);
        }

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
            self.last_telemetry_frame = Some(telemetry_frame.clone());
            self.send_packet(
                &Packet::EcuTelemetry(telemetry_frame),
                NetworkAddress::MissionControl,
            );

            if let Some(tank_telemetry_frame) = self.generate_tank_telemetry_frame() {
                self.send_packet(
                    &Packet::EcuTankTelemetry(tank_telemetry_frame),
                    NetworkAddress::MissionControl,
                );
            }
        }

        if self.debug_info_enabled {
            for variant in EcuDebugInfoVariant::iter() {
                let variant_data = self.generate_debug_info(variant);
                self.send_packet(
                    &Packet::EcuDebugInfo(variant_data),
                    NetworkAddress::MissionControl,
                );
            }
        }
    }

    pub fn generate_telemetry_frame(&self) -> EcuTelemetryFrame {
        EcuTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            engine_state: EngineState::Idle,
            igniter_state: self.igniter_state(),
            igniter_chamber_pressure_pa: self.igniter_chamber_pressure_pa,
        }
    }

    pub fn generate_tank_telemetry_frame(&self) -> Option<EcuTankTelemetryFrame> {
        if self.config.tanks_config.is_none() {
            return None;
        }

        let fuel_tank_state = self.fuel_tank_state().unwrap_or(TankState::Idle);
        let oxidizer_tank_state = self.oxidizer_tank_state().unwrap_or(TankState::Idle);
        Some(EcuTankTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            fuel_tank_state,
            oxidizer_tank_state,
            fuel_tank_pressure_pa: self.fuel_tank_pressure_pa,
            oxidizer_tank_pressure_pa: self.oxidizer_tank_pressure_pa,
        })
    }

    pub fn update_sensor_data(&mut self, data: &EcuSensor) {
        match data {
            EcuSensor::FuelTankPressure(data) => {
                self.fuel_tank_pressure_pa = data.pressure_pa;
            },
            EcuSensor::OxidizerTankPressure(data) => {
                self.oxidizer_tank_pressure_pa = data.pressure_pa;
            },
            EcuSensor::IgniterChamberPressure(data) => {
                self.igniter_chamber_pressure_pa = data.pressure_pa;
            },
        }

        if self.debug_info_enabled {
            self.send_packet(
                &Packet::EcuDebugSensorMeasurement(data.clone()),
                NetworkAddress::MissionControl,
            );
        }
    }

    pub fn configure_ecu(&mut self, config: EcuConfig) {
        self.config = config;

        if self.config.tanks_config.is_some() {
            self.fuel_tank = Some(ControllerEntity::new(
                self,
                tank_fsm::idle::Idle::new(
                    TankType::Fuel,
                    EcuBinaryOutput::FuelPressValve,
                    EcuBinaryOutput::FuelVentValve,
                ),
            ));
            self.oxidizer_tank = Some(ControllerEntity::new(
                self,
                tank_fsm::idle::Idle::new(
                    TankType::Oxidizer,
                    EcuBinaryOutput::OxidizerPressValve,
                    EcuBinaryOutput::OxidizerVentValve,
                ),
            ));
        } else {
            self.fuel_tank = None;
            self.oxidizer_tank = None;
        }
    }

    pub(crate) fn send_packet(&mut self, packet: &Packet, destination: NetworkAddress) {
        let _ = self.comms.send_packet(packet, destination);
    }

    pub(crate) fn igniter_state(&self) -> IgniterState {
        self.igniter.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(IgniterState::Idle)
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
