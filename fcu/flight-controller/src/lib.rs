#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

mod dev_stats;
pub mod state_vector;
pub mod vehicle_fsm;

use dev_stats::DevStatsCollector;
use hal::{fcu_hal::{FcuDriver, VehicleState, FcuTelemetryFrame, OutputChannel, PwmChannel, FcuConfig, FcuDebugInfo}, comms_hal::{Packet, NetworkAddress}, fcu_log::DataPoint};
use mint::Vector3;
use state_vector::StateVector;
use strum::EnumCount;

pub(crate) struct FcuSensorData {
    pub accelerometer: Vector3<f32>,
    pub raw_accelerometer: Vector3<i16>,
    pub gyroscope: Vector3<f32>,
    pub raw_gyroscope: Vector3<i16>,
    pub magnetometer: Vector3<f32>,
    pub raw_magnetometer: Vector3<i16>,
    pub raw_barometer: u32,
    pub barometric_altitude: f32,
}

pub struct Fcu<'a> {
    config: FcuConfig,
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
    pub state_vector: StateVector,
    dev_stats: DevStatsCollector,
    sensor_data: FcuSensorData,
    vehicle_fsm_storage: vehicle_fsm::FsmStorage,
    time_since_last_telemetry: f32,
    time_since_last_ping: f32,
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
            accelerometer_noise_std_dev: Vector3 { x: 0.5, y: 0.5, z: 0.5 },
            barometer_noise_std_dev: 0.01,
            gps_noise_std_dev: Vector3 { x: 1.5, y: 3.0, z: 1.5 },
        };

        let state_vector = StateVector::new(&default_fcu_config);

        let mut fcu = Self {
            config: default_fcu_config,
            vehicle_state: VehicleState::Calibrating,
            driver,
            state_vector,
            dev_stats: DevStatsCollector::new(),
            sensor_data: FcuSensorData {
                accelerometer: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                raw_accelerometer: Vector3 { x: 0, y: 0, z: 0 },
                gyroscope: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                raw_gyroscope: Vector3 { x: 0, y: 0, z: 0 },
                magnetometer: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                raw_magnetometer: Vector3 { x: 0, y: 0, z: 0 },
                raw_barometer: 0,
                barometric_altitude: 0.0,
            },
            vehicle_fsm_storage: vehicle_fsm::FsmStorage::Empty,
            time_since_last_telemetry: 0.0,
            time_since_last_ping: 0.0,
            data_logged_bytes: 0,
            apogee: 0.0,
        };
        fcu.init_vehicle_fsm();

        fcu
    }

    pub fn update(&mut self, dt: f32, packets: &[(NetworkAddress, Packet)]) {
        let timestamp = self.driver.timestamp();

        self.dev_stats.log_update_start(timestamp, packets.len() as u32, 0.0);
        self.state_vector.predict(dt);

        self.apogee = self.apogee.max(self.state_vector.get_position().y);

        self.time_since_last_telemetry += dt;
        self.time_since_last_ping += dt;

        if self.time_since_last_telemetry >= self.config.telemetry_rate {
            self.driver.send_packet(
                Packet::FcuTelemetry(self.generate_telemetry_frame()),
                NetworkAddress::MissionControl,
            );
            self.time_since_last_telemetry = 0.0;
        }

        if self.time_since_last_ping >= 0.5 {
            self.time_since_last_ping = 0.0;
        }

        for (source, packet) in packets {
            self.handle_packet(*source, packet);
        }

        if let Some(frame) = self.dev_stats.pop_stats_frame() {
            self.driver.send_packet(Packet::FcuDevStatsFrame(frame), NetworkAddress::MissionControl);
        }

        self.update_vehicle_fsm(dt, packets);
        self.dev_stats.log_update_end(self.driver.timestamp());
    }

    fn handle_packet(&mut self, source: NetworkAddress, packet: &Packet) {
        match packet {
            Packet::ConfigureFcu(config) => {
                self.configure_fcu(config.clone());
            },
            Packet::EraseDataLogFlash => {
                self.driver.erase_flash_chip();
            },
            Packet::EnableDataLogging(state) => {
                if *state {
                    self.driver.enable_logging_to_flash();
                } else {
                    self.driver.disable_logging_to_flash();
                }
            },
            Packet::RetrieveDataLogPage(addr) => {
                self.driver.retrieve_log_flash_page(*addr);
            },
            Packet::StartDevStatsFrame => {
                self.dev_stats.start_collection(self.driver.timestamp());
            },
            Packet::RequestFcuDebugInfo => {
                let frame = self.generate_debug_info();
                let packet = Packet::FcuDebugInfo(frame);

                self.driver.send_packet(packet, source);
            },
            _ => {}
        }
    }

    pub fn generate_telemetry_frame(&self) -> FcuTelemetryFrame {
        FcuTelemetryFrame {
            timestamp: (self.driver.timestamp() * 1e3) as u64,
            vehicle_state: self.vehicle_state,
            position: self.state_vector.get_position(),
            velocity: self.state_vector.get_velocity(),
            acceleration: self.state_vector.get_acceleration(),
            orientation: self.state_vector.get_orientation(),
            angular_velocity: self.state_vector.get_angular_velocity(),
            position_error: self.state_vector.get_position_std_dev_scalar(),
            velocity_error: self.state_vector.get_velocity_std_dev_scalar(),
            acceleration_error: self.state_vector.get_acceleration_std_dev_scalar(),
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
            position: self.state_vector.get_position(),
            velocity: self.state_vector.get_velocity(),
            acceleration: self.state_vector.get_acceleration(),
            orientation: self.state_vector.get_orientation(),
            angular_velocity: self.state_vector.get_angular_velocity(),
            angular_acceleration: self.state_vector.get_angular_acceleration(),
            position_error: self.state_vector.get_position_std_dev(),
            velocity_error: self.state_vector.get_velocity_std_dev(),
            acceleration_error: self.state_vector.get_acceleration_std_dev(),
            output_channels: [false; OutputChannel::COUNT],
            pwm_channels: [0.0; PwmChannel::COUNT],
            apogee: self.apogee,
            battery_voltage: 11.1169875,
            data_logged_bytes: self.data_logged_bytes,
            raw_accelerometer: self.sensor_data.raw_accelerometer,
            raw_gyroscope: self.sensor_data.raw_gyroscope,
            raw_magnetometer: self.sensor_data.raw_magnetometer,
            raw_barometer: self.sensor_data.raw_barometer,
            raw_barometric_altitude: self.sensor_data.barometric_altitude,
        }
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>, raw: Vector3<i16>) {
        self.state_vector.update_acceleration(acceleration.into());
        self.sensor_data.accelerometer = acceleration;
        self.sensor_data.raw_accelerometer = raw;
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: Vector3<f32>, raw: Vector3<i16>) {
        self.state_vector.update_angular_velocity(angular_velocity.into());
        self.sensor_data.gyroscope = angular_velocity;
        self.sensor_data.raw_gyroscope = raw;
    }

    pub fn update_magnetic_field(&mut self, magnetic_field: Vector3<f32>, raw: Vector3<i16>) {
        self.state_vector.update_magnetic_field(magnetic_field.into());
        self.sensor_data.magnetometer = magnetic_field;
        self.sensor_data.raw_magnetometer = raw;
    }

    pub fn update_barometric_pressure(
        &mut self,
        barometric_pressure: f32,
        temperature: f32,
        raw_pressure: u32,
    ) {
        self.state_vector.update_barometric_pressure(barometric_pressure.into());
        self.sensor_data.raw_barometer = raw_pressure;

        let barometric_altitude = shared::standard_atmosphere::convert_pressure_to_altitude(
            barometric_pressure,
            temperature,
        );

        self.sensor_data.barometric_altitude = barometric_altitude;
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        self.state_vector.update_gps(gps.into());
    }

    pub fn log_data_point(&mut self, _data: DataPoint) {

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