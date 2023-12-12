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
    () => { };
    ($($arg:tt)*) => { };
}

mod dev_stats;
pub mod state_vector;
pub mod vehicle_fsm;

use big_brother::BigBrother;
use dev_stats::DevStatsCollector;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{
        FcuConfig, FcuDebugInfo, FcuDriver, FcuSensorData, FcuTelemetryFrame,
        PwmChannel, VehicleState, FcuAlertCondition,
    }, alerts::AlertManager, DataPointLogger, COMMS_NETWORK_MAP_SIZE,
};
use mint::Vector3;
use state_vector::StateVector;
use strum::EnumCount;

pub const HEARTBEAT_RATE: f32 = 0.25;
pub const ALERT_RATE: f32 = 1.0;
pub const PACKET_QUEUE_SIZE: usize = 16;

pub type FcuBigBrother<'a> = BigBrother<'a, COMMS_NETWORK_MAP_SIZE, Packet, NetworkAddress>;

pub struct Fcu<'a> {
    config: FcuConfig,
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
    pub comms: &'a mut FcuBigBrother<'a>,
    pub data_logger: &'a mut dyn DataPointLogger<FcuSensorData>,
    pub state_vector: StateVector,
    alert_manager: AlertManager<FcuAlertCondition>,
    dev_stats: DevStatsCollector,
    vehicle_fsm_state: Option<vehicle_fsm::FsmState>,
    time_since_last_telemetry: f32,
    time_since_last_heartbeat: f32,
    time_since_last_alert_update: f32,
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
            position_kalman_process_variance: 1e-3,
            calibration_duration: 5.0,
            accelerometer_noise_std_dev: Vector3 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
            barometer_noise_std_dev: 0.01,
            gps_noise_std_dev: Vector3 {
                x: 1.5,
                y: 3.0,
                z: 1.5,
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
            alert_manager: AlertManager::new(),
            dev_stats: DevStatsCollector::new(),
            vehicle_fsm_state: None,
            time_since_last_telemetry: 0.0,
            time_since_last_heartbeat: 0.0,
            time_since_last_alert_update: 0.0,
            apogee: 0.0,
        };
        fcu.init_vehicle_fsm();
        let _ = fcu.comms.send_packet(&Packet::DeviceBooted, NetworkAddress::Broadcast);

        fcu
    }

    pub fn update(&mut self, dt: f32) {
        let timestamp = self.driver.timestamp();

        self.poll_interfaces();

        let mut packets = empty_packet_array();
        let mut num_packets = 0;
        while let Some((packet, source)) = self.comms.recv_packet().ok().flatten() {
            silprintln!("FCU: From {:?} received packet: {:?}", source, packet);
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
            self.send_packet(
                NetworkAddress::MissionControl,
                Packet::FcuTelemetry(self.generate_telemetry_frame()),
            );
            self.time_since_last_telemetry = 0.0;
        }

        if self.time_since_last_heartbeat >= HEARTBEAT_RATE {
            self.comms
                .send_packet(&Packet::Heartbeat, NetworkAddress::Broadcast)
                .unwrap();
            self.time_since_last_heartbeat = 0.0;
        }

        if self.time_since_last_alert_update > ALERT_RATE || self.alert_manager.has_pending_update() {
            let alert_packet = self.alert_manager.get_condition_packet();
            self.send_packet(
                NetworkAddress::MissionControl,
                alert_packet,
            );
            self.time_since_last_alert_update = 0.0;
        }

        for (source, packet) in packets {
            self.handle_packet(*source, packet);
        }

        if let Some(frame) = self.dev_stats.pop_stats_frame() {
            self.send_packet(
                NetworkAddress::MissionControl,
                Packet::FcuDevStatsFrame(frame),
            );
        }

        self.update_vehicle_fsm(dt, packets);
        self.dev_stats.log_update_end(self.driver.timestamp());
    }

    pub fn poll_interfaces(&mut self) {
        self.comms.poll((self.driver.timestamp() * 1e3) as u32);
    }

    fn send_packet(&mut self, destination: NetworkAddress, packet: Packet) {
        self.comms.send_packet(&packet, destination);
    }

    fn handle_packet(&mut self, source: NetworkAddress, packet: &Packet) {
        match packet {
            Packet::ConfigureFcu(config) => {
                self.configure_fcu(config.clone());
            }
            Packet::EraseDataLogFlash => {
                self.driver.erase_flash_chip();
            }
            Packet::EnableDataLogging(state) => {
                self.data_logger.set_logging_enabled(*state);
            }
            Packet::RetrieveDataLogPage(addr) => {
                self.driver.retrieve_log_flash_page(*addr);
            }
            Packet::StartDevStatsFrame => {
                self.dev_stats.start_collection(self.driver.timestamp());
            }
            Packet::RequestFcuDebugInfo => {
                let frame = self.generate_debug_info();
                let packet = Packet::FcuDebugInfo(frame);

                self.send_packet(source, packet);
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
            output_channels_continuity_bitmask: 0,
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logger.get_bytes_logged(),
        }
    }

    pub fn generate_debug_info(&self) -> FcuDebugInfo {
        FcuDebugInfo {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            vehicle_state: self.vehicle_state,
            position: self.state_vector.get_position().into(),
            velocity: self.state_vector.get_velocity().into(),
            acceleration: self.state_vector.get_acceleration().into(),
            orientation: self.state_vector.get_orientation().into(),
            angular_velocity: self.state_vector.get_angular_velocity().into(),
            angular_acceleration: self.state_vector.get_angular_acceleration(),
            position_error: self.state_vector.get_position_std_dev().into(),
            velocity_error: self.state_vector.get_velocity_std_dev().into(),
            acceleration_error: self.state_vector.get_acceleration_std_dev().into(),
            output_channels_bitmask: 0,
            output_channels_continuity_bitmask: 0,
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logger.get_bytes_logged(),
            raw_accelerometer: self.state_vector.sensor_data.accelerometer_raw.into(),
            raw_gyroscope: self.state_vector.sensor_data.gyroscope_raw.into(),
            raw_magnetometer: self.state_vector.sensor_data.magnetometer_raw.into(),
            raw_barometer: self.state_vector.sensor_data.barometer_raw,
            raw_barometric_altitude: self.state_vector.sensor_data.barometer_altitude,
            accelerometer_calibration: self.state_vector.sensor_calibration.accelerometer.into(),
        }
    }

    pub fn update_sensor_data(&mut self, data: FcuSensorData) {
        self.data_logger.log_data_point(&data);

        self.state_vector.update_sensor_data(data);
    }

    fn configure_fcu(&mut self, config: FcuConfig) {
        self.config = config.clone();
        self.state_vector.update_config(&config);
    }

    pub fn get_fcu_config(&self) -> FcuConfig {
        self.config.clone()
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