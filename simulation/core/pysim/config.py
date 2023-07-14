FCU_UPDATE_RATE = 0.02 # Seconds
SIM_UPDATE_RATE = 0.001 # Seconds

VEHICLE_MASS    = 1 # Kilograms

THRUST          = 30 # Newtons
THRUST_TIME     = 10# Seconds
THRUST_WAIT     = 0.0 # Seconds

ACCEL_RATE      = 0.02 # Seconds
GPS_RATE        = 1.0 # Seconds
BARO_RATE       = 0.1 # Seconds
ANGULAR_RATE    = 0.01 # Seconds

ACCEL_NOISE     = 0.1 # Meters per second squared
GPS_XZ_NOISE    = 5.0 # Meters
GPS_Y_NOISE     = 10.0 # Meters
BARO_NOISE      = 0.01 # Meters
ANGULAR_NOISE   = 0.0 # Radians per second

ACCEL_BIAS      = 0.1 # Meters per second squared

FCU_CONFIG = {
    "telemetry_rate": 0.02, # Seconds
    "startup_acceleration_threshold": 0.1, # Meters per second squared
    "position_kalman_process_variance": 1e-2,
    "accelerometer_noise_std_dev": [ACCEL_NOISE]*3, # Meters per second squared
    "barometer_noise_std_dev": BARO_NOISE, # Meters
    "gps_noise_std_dev": [GPS_XZ_NOISE, GPS_Y_NOISE, GPS_XZ_NOISE], # Meters
}