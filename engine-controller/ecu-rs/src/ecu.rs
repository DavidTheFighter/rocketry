use big_brother::BigBrother;
use shared::{
    alerts::AlertManager, comms_hal::{NetworkAddress, Packet}, ecu_hal::{
        EcuAlert, EcuBinaryOutput, EcuCommand, EcuConfig, EcuDebugInfoVariant, EcuDriver, EcuLinearOutput, EcuResponse, EcuSensor, EcuTankTelemetryFrame, EcuTelemetry, EcuTelemetryFrame, EngineState, IgniterState, PumpState, PumpType, TankState, TankType
    }, ControllerEntity, SensorData, COMMS_NETWORK_MAP_SIZE
};

use crate::{
    engine_fsm::{self, EngineFsm}, igniter_fsm::{self, IgniterFsm}, pump_fsm::{self, PumpFsm}, silprintln, state_vector::StateVector, tank_fsm::{self, TankFsm}
};

use strum::IntoEnumIterator;

pub const PACKET_QUEUE_SIZE: usize = 16;
pub const LOCAL_COMMAND_QUEUE_SIZE: usize = 8;

pub type EcuBigBrother<'a> = BigBrother<'a, COMMS_NETWORK_MAP_SIZE, Packet, NetworkAddress>;

pub struct Ecu<'a> {
    pub config: EcuConfig,
    pub debug_info_enabled: bool,
    pub driver: &'a mut dyn EcuDriver,
    pub comms: &'a mut EcuBigBrother<'a>,
    pub state_vector: StateVector,
    pub alert_manager: AlertManager<EcuAlert>,

    pub engine: Option<ControllerEntity<EngineFsm, Ecu<'a>, EngineState>>,
    pub igniter: Option<ControllerEntity<IgniterFsm, Ecu<'a>, IgniterState>>,
    pub fuel_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,
    pub oxidizer_tank: Option<ControllerEntity<TankFsm, Ecu<'a>, TankState>>,
    pub fuel_pump: Option<ControllerEntity<PumpFsm, Ecu<'a>, PumpState>>,
    pub oxidizer_pump: Option<ControllerEntity<PumpFsm, Ecu<'a>, PumpState>>,

    pub last_telemetry_frame: Option<EcuTelemetryFrame>,
    time_since_last_telemetry: f32,

    pub local_command_queue: [Option<EcuCommand>; LOCAL_COMMAND_QUEUE_SIZE],
}

impl<'a> Ecu<'a> {
    pub fn new(driver: &'a mut dyn EcuDriver, comms: &'a mut EcuBigBrother<'a>) -> Self {
        let mut ecu = Self {
            config: EcuConfig::default(),
            debug_info_enabled: true,
            driver,
            comms,
            state_vector: StateVector::new(),
            alert_manager: AlertManager::new(0.1),
            engine: None,
            igniter: None,
            fuel_tank: None,
            oxidizer_tank: None,
            fuel_pump: None,
            oxidizer_pump: None,
            last_telemetry_frame: None,
            time_since_last_telemetry: 1e3,
            local_command_queue: empty_command_array(),
        };

        ecu.engine = Some(ControllerEntity::new(
            &mut ecu,
            engine_fsm::idle::Idle::new(),
        ));

        ecu.igniter = Some(ControllerEntity::new(
            &mut ecu,
            igniter_fsm::idle::Idle::new(),
        ));

        ecu.fuel_pump = Some(ControllerEntity::new(
            &mut ecu,
            pump_fsm::idle::Idle::new(PumpType::FuelMain, EcuLinearOutput::FuelPump),
        ));

        ecu.oxidizer_pump = Some(ControllerEntity::new(
            &mut ecu,
            pump_fsm::idle::Idle::new(PumpType::OxidizerMain, EcuLinearOutput::OxidizerPump),
        ));

        ecu
    }

    pub fn update(&mut self, dt: f32) {
        self.poll_interfaces();

        let mut num_packets = 0;
        let mut packet_queue = empty_packet_array();
        while let Some((packet, source)) = self.comms.recv_packet().ok().flatten() {
            silprintln!("Received from {:?} got {:?}", source, packet);
            packet_queue[num_packets] = (source, packet);
            num_packets += 1;

            if num_packets >= PACKET_QUEUE_SIZE {
                silprintln!("Packet queue full?!");
                break;
            }
        }

        for command in &mut self.local_command_queue {
            if let Some(command) = command.take() {
                packet_queue[num_packets] = (NetworkAddress::MissionControl, Packet::EcuCommand(command));
                num_packets += 1;

                if num_packets >= PACKET_QUEUE_SIZE {
                    silprintln!("Packet queue full?! (from commands");
                    break;
                }
            } else {
                break;
            }
        }
        let packets = &packet_queue[..num_packets];

        self.handle_non_fsm_commands(packets);

        if let Some(mut engine) = self.engine.take() {
            engine.update(self, dt, packets);
            self.engine = Some(engine);
        }

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

        if let Some(mut fuel_pump) = self.fuel_pump.take() {
            fuel_pump.update(self, dt, packets);
            self.fuel_pump = Some(fuel_pump);
        }

        if let Some(mut oxidizer_pump) = self.oxidizer_pump.take() {
            oxidizer_pump.update(self, dt, packets);
            self.oxidizer_pump = Some(oxidizer_pump);
        }

        self.time_since_last_telemetry += dt;
        if self.time_since_last_telemetry >= self.config.telemetry_rate_s {
            self.time_since_last_telemetry = 0.0;
            let telemetry_frame = self.generate_telemetry_frame();
            self.last_telemetry_frame = Some(telemetry_frame.clone());
            self.send_telemetry_packet(
                EcuTelemetry::Telemetry(telemetry_frame),
                NetworkAddress::MissionControl,
            );

            if let Some(tank_telemetry_frame) = self.generate_tank_telemetry_frame() {
                self.send_telemetry_packet(
                    EcuTelemetry::TankTelemetry(tank_telemetry_frame),
                    NetworkAddress::MissionControl,
                );
            }
        }

        self.update_alert_watchdog();

        let alert_packets = self.alert_manager.update(dt);
        for packet in alert_packets.iter().flatten() {
            self.send_packet(&packet, NetworkAddress::MissionControl);
        }

        if self.debug_info_enabled {
            for variant in EcuDebugInfoVariant::iter() {
                let variant_data = self.generate_debug_info(variant);
                self.send_telemetry_packet(
                    EcuTelemetry::DebugInfo(variant_data),
                    NetworkAddress::MissionControl,
                );
            }
        }
    }

    pub fn poll_interfaces(&mut self) {
        self.comms.poll_1ms((self.driver.timestamp() * 1e3) as u32);
    }

    pub fn handle_non_fsm_commands(&mut self, packets: &[(NetworkAddress, Packet)]) {
        for (remote, packet) in packets {
            match packet {
                Packet::EcuCommand(EcuCommand::GetConfig) => {
                    silprintln!("Received get config command");
                    self.send_response_packet(EcuResponse::Config(self.config.clone()), *remote);
                },
                _ => {},
            }
        }
    }

    pub fn generate_telemetry_frame(&self) -> EcuTelemetryFrame {
        EcuTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            engine_state: self.engine_state(),
            igniter_state: self.igniter_state(),
            engine_chamber_pressure_pa: self.state_vector.sensor_data.engine_chamber_pressure_pa,
            engine_fuel_injector_pressure_pa: self.state_vector.sensor_data.engine_fuel_injector_pressure_pa,
            engine_oxidizer_injector_pressure_pa: self.state_vector.sensor_data.engine_oxidizer_injector_pressure_pa,
            igniter_chamber_pressure_pa: self.state_vector.sensor_data.igniter_chamber_pressure_pa,
            fuel_pump_state: self.fuel_pump.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(PumpState::Idle),
            oxidizer_pump_state: self.oxidizer_pump.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(PumpState::Idle),
            fuel_pump_outlet_pressure_pa: self.state_vector.sensor_data.fuel_pump_outlet_pressure_pa,
            oxidizer_pump_outlet_pressure_pa: self.state_vector.sensor_data.oxidizer_pump_outlet_pressure_pa,
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
            fuel_tank_pressure_pa: self.state_vector.sensor_data.fuel_tank_pressure_pa.unwrap_or(0.0),
            oxidizer_tank_pressure_pa: self.state_vector.sensor_data.oxidizer_tank_pressure_pa.unwrap_or(0.0),
        })
    }

    pub fn update_sensor_data(&mut self, sensor: EcuSensor, data: &SensorData) {
        self.state_vector.update_sensor_data(sensor, data);

        if self.debug_info_enabled {
            self.send_telemetry_packet(
                EcuTelemetry::DebugSensorMeasurement((sensor, data.clone())),
                NetworkAddress::MissionControl,
            );
        }
    }

    pub fn configure_ecu(&mut self, config: EcuConfig) {
        self.config = config;

        if let Some(tanks_config) = self.config.tanks_config.clone() {
            self.fuel_tank = Some(ControllerEntity::new(
                self,
                tank_fsm::idle::Idle::new(
                    TankType::FuelMain,
                    tanks_config.fuel_press_valve,
                    tanks_config.fuel_fill_valve,
                    tanks_config.fuel_vent_valve,
                ),
            ));
            self.oxidizer_tank = Some(ControllerEntity::new(
                self,
                tank_fsm::idle::Idle::new(
                    TankType::OxidizerMain,
                    tanks_config.oxidizer_press_valve,
                    tanks_config.oxidizer_fill_valve,
                    tanks_config.oxidizer_vent_valve,
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

    pub(crate) fn send_telemetry_packet(&mut self, telemetry: EcuTelemetry, destination: NetworkAddress) {
        self.send_packet(&Packet::EcuTelemetry(telemetry), destination);
    }

    pub(crate) fn send_response_packet(&mut self, response: EcuResponse, destination: NetworkAddress) {
        self.send_packet(&Packet::EcuResponse(response), destination);
    }

    pub(crate) fn engine_state(&self) -> EngineState {
        self.engine.as_ref().map(|fsm| fsm.hal_state()).unwrap_or(EngineState::Idle)
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

    pub(crate) fn enqueue_command(&mut self, command: EcuCommand) -> bool {
        for element in &mut self.local_command_queue {
            if element.is_none() {
                *element = Some(command);
                return true;
            }
        }

        return false;
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

fn empty_command_array() -> [Option<EcuCommand>; LOCAL_COMMAND_QUEUE_SIZE] {
    [None, None, None, None, None, None, None, None]
}


