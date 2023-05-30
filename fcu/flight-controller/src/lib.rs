#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

pub mod kalman;

use hal::{fcu_hal::{FcuDriver, VehicleState, FcuTelemetryFrame, OutputChannel, PwmChannel}, comms_hal::{Packet, NetworkAddress}};
use mint::{Vector3, Quaternion};
use strum::EnumCount;

use num_traits::float::Float;

pub struct Fcu<'a> {
    pub vehicle_state: VehicleState,
    pub driver: &'a mut dyn FcuDriver,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub orientation: Quaternion<f32>,
    pub angular_velocity: Vector3<f32>,
    pub mangetic_field: Vector3<f32>,
    time_since_last_telemetry: f32,
    data_logged_bytes: u32,
}

impl<'a> Fcu<'a> {
    pub fn new(driver: &'a mut dyn FcuDriver) -> Self {
        Self {
            vehicle_state: VehicleState::Idle,
            driver,
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            orientation: Quaternion { s: 1.0, v: Vector3 { x: 0.0, y: 0.0, z: 0.0 } },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            mangetic_field: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            time_since_last_telemetry: 0.0,
            data_logged_bytes: 0,
        }
    }

    pub fn update(&mut self, dt: f32, packet: Option<Packet>) {
        if dt > 1e-4 {
            self.velocity.x += self.acceleration.x * dt;
            self.velocity.y += self.acceleration.y * dt;
            self.velocity.z += self.acceleration.z * dt;
            self.position.x += self.velocity.x * dt;
            self.position.y += self.velocity.y * dt;
            self.position.z += self.velocity.z * dt;

            if self.vehicle_state == VehicleState::Idle && self.acceleration.y > 1e-1 {
                self.vehicle_state = VehicleState::Ascent;
            } else if self.vehicle_state == VehicleState::Ascent && self.velocity.y < 0.0 {
                self.vehicle_state = VehicleState::Descent;
            }

            self.timestep_orientation(dt);

            self.time_since_last_telemetry += dt;
            if self.time_since_last_telemetry >= 0.02 {
                let telem_frame = FcuTelemetryFrame {
                    timestamp: 0,
                    vehicle_state: self.vehicle_state,
                    position: self.position,
                    velocity: self.velocity,
                    acceleration: self.acceleration,
                    orientation: self.orientation,
                    angular_velocity: self.angular_velocity,
                    angular_acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                    magnetometer: self.mangetic_field,
                    output_channels: [false; OutputChannel::COUNT],
                    pwm_channels: [0.0; PwmChannel::COUNT],
                    battery_voltage: 11.1169875,
                    data_logged_bytes: self.data_logged_bytes,
                };

                self.driver.send_packet(
                    Packet::FcuTelemetry(telem_frame),
                    NetworkAddress::MissionControl,
                );
                self.time_since_last_telemetry = 0.0;
            }
        }

        if let Some(packet) = packet {
            match packet {
                Packet::EraseDataLogFlash => {
                    self.driver.erase_flash_chip();
                },
                Packet::EnableDataLogging(state) => {
                    if state {
                        self.driver.enable_logging_to_flash();
                    } else {
                        self.driver.disable_logging_to_flash();
                    }
                },
                Packet::RetrieveDataLogPage(addr) => {
                    self.driver.retrieve_log_flash_page(addr);
                },
                _ => {}
            }
        }
    }

    pub fn update_acceleration(&mut self, acceleration: Vector3<f32>) {
        self.acceleration = acceleration;
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: Vector3<f32>) {
        self.angular_velocity = angular_velocity;
    }

    pub fn update_magnetic_field(&mut self, magnetic_field: Vector3<f32>) {
        self.mangetic_field = magnetic_field;
    }

    pub fn update_barometric_pressure(&mut self, barometric_pressure: f32) {
        // something
    }

    pub fn update_gps(&mut self, gps: Vector3<f32>) {
        // something
    }

    pub fn update_data_logged_bytes(&mut self, bytes: u32) {
        self.data_logged_bytes = bytes;
    }

    fn timestep_orientation(&mut self, dt: f32) {
        let angular_velocity_magnitude = (
            self.angular_velocity.x.powi(2)
            + self.angular_velocity.y.powi(2)
            + self.angular_velocity.z.powi(2)
        ).sqrt();

        if angular_velocity_magnitude < 1e-5 {
            return;
        }

        let angle = angular_velocity_magnitude * dt * 0.5;
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        let angular_velocity_quat = Quaternion {
            s: cos_angle,
            v: Vector3 {
                x: self.angular_velocity.x * sin_angle / angular_velocity_magnitude,
                y: self.angular_velocity.y * sin_angle / angular_velocity_magnitude,
                z: self.angular_velocity.z * sin_angle / angular_velocity_magnitude,
            },
        };

        self.orientation = self.quat_norm(self.quat_mult(angular_velocity_quat, self.orientation));
    }

    fn quat_mult(&self, q1: Quaternion<f32>, q2: Quaternion<f32>) -> Quaternion<f32> {
        Quaternion {
            s: q1.s * q2.s - q1.v.x * q2.v.x - q1.v.y * q2.v.y - q1.v.z * q2.v.z,
            v: Vector3 {
                x: q1.s * q2.v.x + q1.v.x * q2.s + q1.v.y * q2.v.z - q1.v.z * q2.v.y,
                y: q1.s * q2.v.y - q1.v.x * q2.v.z + q1.v.y * q2.s + q1.v.z * q2.v.x,
                z: q1.s * q2.v.z + q1.v.x * q2.v.y - q1.v.y * q2.v.x + q1.v.z * q2.s,
            },
        }
    }

    fn quat_norm(&self, q: Quaternion<f32>) -> Quaternion<f32> {
        let norm = (q.s * q.s + q.v.x * q.v.x + q.v.y * q.v.y + q.v.z * q.v.z).sqrt();
        Quaternion {
            s: q.s / norm,
            v: Vector3 {
                x: q.v.x / norm,
                y: q.v.y / norm,
                z: q.v.z / norm,
            },
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Fcu<'_> {}