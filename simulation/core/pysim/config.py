FCU_UPDATE_RATE = 0.02 # Seconds
SIM_UPDATE_RATE = 0.001 # Seconds
DEV_STATS_RATE  = 2.0 # Seconds

VEHICLE_MASS    = 1 # Kilograms

THRUST          = 25 # Newtons
THRUST_TIME     = 3 # Seconds
THRUST_WAIT     = 5.0 # Seconds

ACCEL_RATE      = 0.02 # Seconds
GPS_RATE        = 1.0 # Seconds
BARO_RATE       = 0.1 # Seconds
ANGULAR_RATE    = 0.01 # Seconds

ACCEL_NOISE     = 0.01 # Meters per second squared
GPS_XZ_NOISE    = 5.0 # Meters
GPS_Y_NOISE     = 10.0 # Meters
BARO_NOISE      = 0.1 # Meters
ANGULAR_NOISE   = 0.01 # Radians per second

ACCEL_BIAS      = 0.05 # Meters per second squared
GYRO_BIAS       = 0.05 # Radians per second

FCU_CONFIG = {
    "telemetry_rate": 0.02, # Seconds
    "startup_acceleration_threshold": 0.5, # Meters per second squared
    "position_kalman_process_variance": 1e-1,
    "calibration_duration": 2.5, # Seconds
    "accelerometer_noise_std_dev": [ACCEL_NOISE]*3, # Meters per second squared
    "barometer_noise_std_dev": BARO_NOISE, # Meters
    "gps_noise_std_dev": [GPS_XZ_NOISE, GPS_Y_NOISE, GPS_XZ_NOISE], # Meters
}