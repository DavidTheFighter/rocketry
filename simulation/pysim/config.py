from dataclasses import dataclass, field

@dataclass
class SimConfig:
    fcu_update_rate: float = 0.02 # Seconds
    sim_update_rate: float = 0.001 # Seconds
    dev_stats_rate: float = 2.0 # Seconds

    vehicle_mass: float = 1 # Kilograms
    thrust: float = 25 # Newtons
    thrust_time: float = 3 # Seconds
    thrust_wait: float = 5.0 # Seconds

    accel_data_rate: float = 0.02 # Seconds
    gps_data_rate: float = 1.0 # Seconds
    baro_data_rate: float = 0.1 # Seconds
    angular_data_rate: float = 0.01 # Seconds

    # Make sure to update the noises in fcu_config
    accel_noise: float = 0.01 # Meters per second squared
    gps_xz_noise: float = 5.0 # Meters
    gps_y_noise: float = 10.0 # Meters
    baro_noise: float = 0.5 # Meters
    gyro_noise: float = 0.01 # Radians per second

    accel_bias: float = 0.05 # Meters per second squared
    gyro_bias: float = 0.001 # Radians per second
    baro_bias: float = 0.01 # Meters

    # Vehicle config
    auto_ignite_solid_motor: bool = False # Auto ignite motor in sim after arming

    fcu_config: dict = field(default_factory=lambda: {
        "telemetry_rate": 0.02, # Seconds
        "startup_acceleration_threshold": 0.5, # Meters per second squared
        "calibration_duration": 2.5, # Seconds
        "kalman_process_variance": 1e1,
        "accelerometer_noise_std_dev": [0.01]*3, # Meters per second squared
        "barometer_noise_std_dev": 1.0, # Meters
        "gps_noise_std_dev": [5.0, 10.0, 5.0], # Meters
        "gyro_noise_std_dev": [0.1]*3, # Radians per second
    })

    def is_time_before_thrust(self, t):
        return t < self.thrust_wait

    def is_time_during_thrust(self, t):
        return t > self.thrust_wait and t < self.thrust_wait + self.thrust_time

    def is_time_after_thrust(self, t):
        return t > self.thrust_wait + self.thrust_time
