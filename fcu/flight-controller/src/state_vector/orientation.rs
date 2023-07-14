use hal::fcu_hal::FcuConfig;
use nalgebra::{Vector3, Quaternion, Vector4, UnitQuaternion};

use super::StateVector;

impl StateVector {
    pub(super) fn predict_orientation(&mut self, dt: f32) {
        let angular_velocity_magnitude = self.angular_velocity.magnitude();
        if angular_velocity_magnitude < 1e-5 {
            return;
        }

        let angle = angular_velocity_magnitude * dt * 0.5;
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        let angular_velocity_quat = Quaternion {
            coords: Vector4::new(
                self.angular_velocity.x * sin_angle / angular_velocity_magnitude,
                self.angular_velocity.y * sin_angle / angular_velocity_magnitude,
                self.angular_velocity.z * sin_angle / angular_velocity_magnitude,
                cos_angle,
            ),
        };
        let angular_velocity_quat = UnitQuaternion::from_quaternion(angular_velocity_quat);

        self.orientation = angular_velocity_quat * self.orientation;
    }

    pub fn update_angular_velocity(&mut self, angular_velocity: Vector3<f32>, timestamp: f32) {
        if self.last_angular_velocity_timestamp > 1e-4 {
            let dt = timestamp - self.last_angular_velocity_timestamp;
            self.angular_acceleration = (angular_velocity - self.angular_velocity) / dt;
        }

        self.last_angular_velocity_timestamp = timestamp;
        self.angular_velocity = angular_velocity;
    }

    pub fn update_magnetic_field(&mut self, _magnetic_field: Vector3<f32>) {
        // something
    }

    pub(super) fn update_config_orientation(&mut self, _config: &FcuConfig) {

    }
}