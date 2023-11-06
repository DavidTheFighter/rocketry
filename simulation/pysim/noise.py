import numpy as np

from pysim.config import SimConfig

def accel_noise(accel, config: SimConfig):
    noise = [np.random.normal(config.accel_bias, config.accel_noise) for _ in range(3)]

    return [accel[0] + noise[0], accel[1] + noise[1], accel[2] + noise[2]]

def gps_noise(position, config: SimConfig):
    noise = [np.random.normal(0, std) for std in [config.gps_xz_noise, config.gps_y_noise, config.gps_y_noise]]

    return [position[0] + noise[0], position[1] + noise[1], position[2] + noise[2]]

def baro_noise(altitude, config: SimConfig):
    return altitude + np.random.normal(config.baro_bias, config.baro_noise)

def gyro_noise(angular_vel, config: SimConfig):
    noise = [np.random.normal(config.gyro_bias, config.gyro_noise) for _ in range(3)]

    return [angular_vel[0] + noise[0], angular_vel[1] + noise[1], angular_vel[2] + noise[2]]