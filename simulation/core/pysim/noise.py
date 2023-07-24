import numpy as np
from pysim.config import *

def accel_noise(accel):
    noise = [np.random.normal(ACCEL_BIAS, ACCEL_NOISE) for _ in range(3)]

    return [accel[0] + noise[0], accel[1] + noise[1], accel[2] + noise[2]]

def gps_noise(position):
    noise = [np.random.normal(0, std) for std in [GPS_XZ_NOISE, GPS_Y_NOISE, GPS_XZ_NOISE]]

    return [position[0] + noise[0], position[1] + noise[1], position[2] + noise[2]]

def baro_noise(altitude):
    return altitude + np.random.normal(0, BARO_NOISE)

def angular_vel_noise(angular_vel):
    noise = [np.random.normal(GYRO_BIAS, ANGULAR_NOISE) for _ in range(3)]

    return [angular_vel[0] + noise[0], angular_vel[1] + noise[1], angular_vel[2] + noise[2]]