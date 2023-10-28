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

use dev_stats::DevStatsCollector;
use shared::{
    comms_hal::{NetworkAddress, Packet},
    fcu_hal::{
        FcuConfig, FcuDebugInfo, FcuDriver, FcuSensorData, FcuTelemetryFrame, OutputChannel,
        PwmChannel, VehicleState,
    },
};
use mint::Vector3;
use state_vector::StateVector;
use strum::EnumCount;

pub const HEARTBEAT_RATE: f32 = 0.25;

pub struct Fcu<'a> {
    config: FcuConfig,
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
    pub state_vector: StateVector,
    dev_stats: DevStatsCollector,
    vehicle_fsm_state: Option<vehicle_fsm::FsmState>,
    time_since_last_telemetry: f32,
    time_since_last_heartbeat: f32,
    data_logged_bytes: u32,
    apogee: f32,
}

impl<'a> Fcu<'a> {
    pub fn new(driver: &'a mut dyn FcuDriver) -> Self {
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
            state_vector,
            dev_stats: DevStatsCollector::new(),
            vehicle_fsm_state: None,
            time_since_last_telemetry: 0.0,
            time_since_last_heartbeat: 0.0,
            data_logged_bytes: 0,
            apogee: 0.0,
        };
        fcu.init_vehicle_fsm();

        fcu
    }

    pub fn update(&mut self, dt: f32, packets: &[(NetworkAddress, Packet)]) {
        let timestamp = self.driver.timestamp();

        self.dev_stats
            .log_update_start(timestamp, packets.len() as u32, 0.0);
        self.state_vector.predict(dt);

        self.apogee = self.apogee.max(self.state_vector.get_position().y);

        self.time_since_last_telemetry += dt;
        self.time_since_last_heartbeat += dt;

        if self.time_since_last_telemetry >= self.config.telemetry_rate {
            self.driver.send_packet(
                Packet::FcuTelemetry(self.generate_telemetry_frame()),
                NetworkAddress::MissionControl,
            );
            self.time_since_last_telemetry = 0.0;
        }

        if self.time_since_last_heartbeat >= HEARTBEAT_RATE {
            self.driver
                .send_packet(Packet::Heartbeat, NetworkAddress::MissionControl);
            self.time_since_last_heartbeat = 0.0;
        }

        for (source, packet) in packets {
            self.handle_packet(*source, packet);
        }

        if let Some(frame) = self.dev_stats.pop_stats_frame() {
            self.driver.send_packet(
                Packet::FcuDevStatsFrame(frame),
                NetworkAddress::MissionControl,
            );
        }

        self.update_vehicle_fsm(dt, packets);
        self.dev_stats.log_update_end(self.driver.timestamp());
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
                if *state {
                    self.driver.enable_logging_to_flash();
                } else {
                    self.driver.disable_logging_to_flash();
                }
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

                self.driver.send_packet(packet, source);
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
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logged_bytes,
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
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logged_bytes,
            raw_accelerometer: self.state_vector.sensor_data.accelerometer_raw.into(),
            raw_gyroscope: self.state_vector.sensor_data.gyroscope_raw.into(),
            raw_magnetometer: self.state_vector.sensor_data.magnetometer_raw.into(),
            raw_barometer: self.state_vector.sensor_data.barometer_raw,
            raw_barometric_altitude: self.state_vector.sensor_data.barometer_altitude,
            accelerometer_calibration: self.state_vector.sensor_calibration.accelerometer.into(),
        }
    }

    pub fn update_sensor_data(&mut self, data: FcuSensorData) {
        self.state_vector.update_sensor_data(data);
    }

    pub fn update_data_logged_bytes(&mut self, bytes: u32) {
        self.data_logged_bytes = bytes;
    }

    pub fn configure_fcu(&mut self, config: FcuConfig) {
        self.config = config.clone();
        self.state_vector.update_config(&config);
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Fcu<'_> {}

pub(crate) trait FiniteStateMachine<D> {
    fn update(fcu: &mut Fcu, dt: f32, packets: &[(NetworkAddress, Packet)]) -> Option<D>;
    fn setup_state(fcu: &mut Fcu);
}
