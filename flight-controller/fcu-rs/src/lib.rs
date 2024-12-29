// Define no_std except for testing and sil feature
#![cfg_attr(not(any(test, feature = "sil")), no_std)]
#![deny(unsafe_code)]

#[cfg(any(test, feature = "sil"))]
macro_rules! silprintln {
    () => { println!() };
    ($($arg:tt)*) => { println!($($arg)*) };
}

#[cfg(not(any(test, feature = "sil")))]
macro_rules! silprintln {
    () => {};
    ($($arg:tt)*) => {};
}

mod alert_watchdog;
pub mod debug_info;
mod dev_stats;
pub mod state_vector;
pub mod vehicle_fsm;

use big_brother::BigBrother;
use dev_stats::DevStatsCollector;
use mint::Vector3;
use shared::{
    alerts::AlertManager,
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{
        FcuAlertCondition, FcuConfig, FcuDebugInfoVariant, FcuDriver, FcuSensorData,
        FcuTelemetryFrame, OutputChannel, PwmChannel, VehicleCommand, VehicleState,
    },
    DataPointLogger, COMMS_NETWORK_MAP_SIZE,
};
use state_vector::StateVector;
use strum::{EnumCount, IntoEnumIterator};

pub const HEARTBEAT_RATE: f32 = 0.25;
pub const ALERT_RATE: f32 = 0.1;
pub const PACKET_QUEUE_SIZE: usize = 16;

pub type FcuBigBrother<'a> = BigBrother<'a, COMMS_NETWORK_MAP_SIZE, Packet, NetworkAddress>;

pub struct Fcu<'a> {
    config: FcuConfig,
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
    pub comms: &'a mut FcuBigBrother<'a>,
    pub data_logger: &'a mut dyn DataPointLogger<FcuSensorData>,
    pub state_vector: StateVector,
    pub last_telemetry_frame: Option<FcuTelemetryFrame>,
    debug_info_enabled: bool,
    alert_manager: AlertManager<FcuAlertCondition>,
    dev_stats: DevStatsCollector,
    vehicle_fsm_state: Option<vehicle_fsm::FsmState>,
    time_since_last_telemetry: f32,
    time_since_last_heartbeat: f32,
    apogee: f32,
}

impl<'a> Fcu<'a> {
    pub fn new(
        driver: &'a mut dyn FcuDriver,
        comms: &'a mut FcuBigBrother<'a>,
        data_logger: &'a mut dyn DataPointLogger<FcuSensorData>,
    ) -> Self {
        let default_fcu_config = FcuConfig {
            telemetry_rate: 0.02,
            startup_acceleration_threshold: 0.1,
            startup_acceleration_timeout: 5.0,
            calibration_duration: 5.0,
            kalman_process_variance: 1e-1,
            accelerometer_noise_std_dev: Vector3 {
                x: 0.01,
                y: 0.01,
                z: 0.01,
            },
            barometer_noise_std_dev: 0.5,
            gps_noise_std_dev: Vector3 {
                x: 1.5,
                y: 3.0,
                z: 1.5,
            },
            gyro_noise_std_dev: Vector3 {
                x: 0.1,
                y: 0.1,
                z: 0.1,
            },
        };

        let state_vector = StateVector::new(&default_fcu_config);

        let mut fcu = Self {
            config: default_fcu_config,
            vehicle_state: VehicleState::Calibrating,
            driver,
            comms,
            data_logger,
            state_vector,
            last_telemetry_frame: None,
            debug_info_enabled: true,
            alert_manager: AlertManager::new(ALERT_RATE),
            dev_stats: DevStatsCollector::new(),
            vehicle_fsm_state: None,
            time_since_last_telemetry: 0.0,
            time_since_last_heartbeat: 0.0,
            apogee: 0.0,
        };
        fcu.init_vehicle_fsm();
        let _ = fcu
            .comms
            .send_packet(&Packet::DeviceBooted, NetworkAddress::Broadcast);

        fcu
    }

    pub fn update(&mut self, dt: f32) {
        let timestamp = self.driver.timestamp();

        self.poll_interfaces();

        let mut packets = empty_packet_array();
        let mut num_packets = 0;
        while let Some((packet, source)) = self.comms.recv_packet().ok().flatten() {
            packets[num_packets] = (source, packet);
            num_packets += 1;
        }
        let packets = &packets[..num_packets];

        self.dev_stats
            .log_update_start(timestamp, packets.len() as u32, 0.0);
        self.state_vector.predict(dt);

        self.apogee = self.apogee.max(self.state_vector.get_position().y);

        self.time_since_last_telemetry += dt;
        self.time_since_last_heartbeat += dt;

        if self.time_since_last_telemetry >= self.config.telemetry_rate {
            let telemetry = self.generate_telemetry_frame();
            self.last_telemetry_frame = Some(telemetry.clone());

            self.send_packet(
                NetworkAddress::MissionControl,
                Packet::FcuTelemetry(telemetry),
            );
            self.time_since_last_telemetry = 0.0;
        }

        self.alert_manager
            .set_condition(FcuAlertCondition::BatteryVoltageLow);

        if self.debug_info_enabled {
            for variant in FcuDebugInfoVariant::iter() {
                let variant_data = self.generate_debug_info(variant);
                self.send_packet(
                    NetworkAddress::MissionControl,
                    Packet::FcuDebugInfo(variant_data),
                );
            }
        }

        self.update_alert_watchdog();

        let am_packets = self.alert_manager.update(dt);
        for packet in am_packets {
            if let Some(packet) = packet {
                self.send_packet(NetworkAddress::MissionControl, packet);
            }
        }

        for (source, packet) in packets {
            // defmt::info!("Received packet from {:?}: {:?}", source, defmt::Debug2Format(packet));
            self.handle_packet(*source, packet);
        }

        self.update_vehicle_fsm(dt, packets);
        self.dev_stats.log_update_end(self.driver.timestamp());
    }

    pub fn poll_interfaces(&mut self) {
        self.comms.poll_1ms((self.driver.timestamp() * 1e3) as u32);
    }

    fn send_packet(&mut self, destination: NetworkAddress, packet: Packet) {
        self.comms.send_packet(&packet, destination);
    }

    fn handle_packet(&mut self, source: NetworkAddress, packet: &Packet) {
        match packet {
            Packet::VehicleCommand(command) => {
                self.handle_command(source, command);
            }
            Packet::EnableDataLogging(state) => {
                self.data_logger.set_logging_enabled(*state);
            }
            Packet::EnableDebugInfo(enable) => {
                self.debug_info_enabled = *enable;
            }
            Packet::ResetMcu { magic_number } => {
                if *magic_number == shared::RESET_MAGIC_NUMBER {
                    self.driver.reset_mcu();
                }
            }
            _ => {}
        }
    }

    fn handle_command(&mut self, _source: NetworkAddress, command: &VehicleCommand) {
        match command {
            VehicleCommand::Configure(config) => {
                self.configure_fcu(config.clone());
            }
            VehicleCommand::SetOutputChannel { channel, state } => {
                self.driver.set_output_channel(*channel, *state);
            }
            _ => {}
        }
    }

    pub fn generate_telemetry_frame(&self) -> FcuTelemetryFrame {
        FcuTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            vehicle_state: self.vehicle_state,
            position: self.state_vector.get_position().into(),
            velocity: self.state_vector.get_velocity().into(),
            acceleration: self.state_vector.get_acceleration().into(),
            orientation: self.state_vector.get_orientation().into(),
            angular_velocity: self.state_vector.get_angular_velocity().into(),
            position_error: self.state_vector.get_acceleration_std_dev().norm(),
            velocity_error: self.state_vector.get_velocity_std_dev().norm(),
            acceleration_error: self.state_vector.get_acceleration_std_dev().norm(),
            output_channels_bitmask: 0,
            output_channels_continuity_bitmask: self.get_output_channels_continuity_bitmask(),
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logger.get_bytes_logged(),
        }
    }

    pub fn update_sensor_data(&mut self, data: FcuSensorData) {
        self.data_logger.log_data_point(&data);

        self.state_vector.update_sensor_data(&data);

        if self.debug_info_enabled {
            self.send_packet(
                NetworkAddress::MissionControl,
                Packet::FcuDebugSensorMeasurement(data),
            );
        }
    }

    pub fn configure_fcu(&mut self, config: FcuConfig) {
        self.config = config.clone();
        self.state_vector.update_config(&config);
    }

    pub fn get_fcu_config(&self) -> FcuConfig {
        self.config.clone()
    }

    fn get_output_channels_continuity_bitmask(&self) -> u16 {
        let mut bitmask = 0;

        if self
            .driver
            .get_output_channel_continuity(OutputChannel::SolidMotorIgniter)
        {
            bitmask |= 1 << OutputChannel::SolidMotorIgniter.index();
        }

        bitmask
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Fcu<'_> {}

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
