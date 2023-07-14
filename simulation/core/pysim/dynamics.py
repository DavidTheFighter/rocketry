import numpy as np
import quaternion

G = -9.81

class VehicleDynamics:
    def __init__(self):
        self.position = np.array([0.0, 0.0, 0.0]) # World frame
        self.velocity = np.array([0.0, 0.0, 0.0]) # World frame
        self.acceleration_world_frame = np.array([0.0, 0.0, 0.0])
        self.acceleration_body_frame = np.array([0.0, 0.0, 0.0])

        self.orientation = quaternion.quaternion(1.0, 0.0, 0.0, 0.0) # Body -> World
        self.angular_velocity = np.array([0.0, 0.0, 0.0]) # Body frame
        self.angular_acceleration = np.array([0.0, 0.0, 0.0]) # Body frame

        self.thrust_accel = 0.0
        self.ignore_g = False

    def update(self, dt):
        thrust_accel_body_frame = np.array([0.0, self.thrust_accel, 0.0])
        gravity_accel_body_frame = quaternion.rotate_vectors(self.orientation.inverse(), np.array([0.0, G, 0.0]))

        self.acceleration_body_frame = thrust_accel_body_frame
        if not self.ignore_g:
            self.acceleration_body_frame += gravity_accel_body_frame

        self.acceleration_world_frame = quaternion.rotate_vectors(self.orientation, self.acceleration_body_frame)

        self.velocity += self.acceleration_world_frame * dt
        self.position += self.velocity * dt

        # turbulence = [x for x in np.random.normal(0, 0.5, 3)]
        # self.angular_acceleration = np.array(turbulence)

        self.angular_velocity += self.angular_acceleration * dt
        self.orientation = integrate_angular_velocity_rk4(self.orientation, self.angular_velocity, dt)

    def set_thrust_accel(self, thrust_accel):
        self.thrust_accel = thrust_accel

def integrate_angular_velocity_rk4(q, w, dt):
    # Define the derivative function for the quaternion
    def q_dot(q, w):
        return 0.5 * quaternion.quaternion(0, w[0], w[1], w[2]) * q

    # Calculate the four sample slopes using Runge-Kutta method
    k1 = q_dot(q, w)
    k2 = q_dot(q + 0.5 * dt * k1, w)
    k3 = q_dot(q + 0.5 * dt * k2, w)
    k4 = q_dot(q + dt * k3, w)

    # Combine the slopes to estimate the quaternion derivative
    q_derivative = (k1 + 2 * k2 + 2 * k3 + k4) / 6

    # Integrate to get the new orientation
    q = q + q_derivative * dt

    # Normalize the quaternion
    q = q.normalized()

    return q