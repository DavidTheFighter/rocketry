from dataclasses import dataclass, field

@dataclass
class SimConfig:
    fcu_update_rate: float = 0.02 # Seconds
    ecu_update_rate: float = 0.001 # Seconds
    sim_update_rate: float = 0.001 # Seconds
    replay_update_rate: float = 0.01 # Seconds

    vehicle_mass: float = 1 # Kilograms
    thrust: float = 25 # Newtons
    thrust_time: float = 3 # Seconds
    thrust_wait: float = 5.0 # Seconds

    accel_data_rate: float = 0.02 # Seconds
    gps_data_rate: float = 1.0 # Seconds
    baro_data_rate: float = 0.1 # Seconds
    angular_data_rate: float = 0.01 # Seconds

    ecu_pressure_sensor_rate: float = 0.001 # Seconds

    ecu_tank_vent_diamter_m: float = 0.0025 # Meters
    ecu_tank_pressure_set_point_pa: float = 50 * 6894.75729 # PSI to pascals

    main_fuel_pump_pressure_setpoint_pa: float = 300 * 6894.75729 # PSI to pascals
    main_oxidizer_pump_pressure_setpoint_pa: float = 300 * 6894.75729 # PSI to pascals

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
        "startup_acceleration_timeout": 5.0, # Seconds
        "calibration_duration": 2.5, # Seconds
        "kalman_process_variance": 1e1,
        "accelerometer_noise_std_dev": [0.01]*3, # Meters per second squared
        "barometer_noise_std_dev": 1.0, # Meters
        "gps_noise_std_dev": [5.0, 10.0, 5.0], # Meters
        "gyro_noise_std_dev": [0.1]*3, # Radians per second
    })

    ecu_config: dict = field(default_factory=lambda: {
        'engine_config': {
            'use_pumps': True,
            'fuel_injector_pressure_setpoint_pa': 500 * 6894.75729, # PSI to pascals
            'fuel_injector_startup_pressure_tolerance_pa': 25 * 6894.75729, # PSI to pascals
            'fuel_injector_running_pressure_tolerance_pa': 100 * 6894.75729, # PSI to pascals
            'oxidizer_injector_pressure_setpoint_pa': 500 * 6894.75729, # PSI to pascals
            'oxidizer_injector_startup_pressure_tolerance_pa': 25 * 6894.75729, # PSI to pascals
            'oxidizer_injector_running_pressure_tolerance_pa': 100 * 6894.75729, # PSI to pascals
            'engine_target_combustion_pressure_pa': 300 * 6894.75729, # PSI to pascals
            'engine_combustion_pressure_tolerance_pa': 200 * 6894.75729, # PSI to pascals
            'pump_startup_timeout_s': 5.0,
            'igniter_startup_timeout_s': 2.0,
            'engine_startup_timeout_s': 1.0,
            'engine_firing_duration_s': 5.0,
            'engine_shutdown_duration_s': 0.5,
        },
        'igniter_config': {
            'startup_timeout_s': 1.0,
            'startup_pressure_threshold_pa': 30 * 6894.75729, # PSI to pascals
            'startup_stable_time_s': 0.25,
            'test_firing_duration_s': 2.0,
            'shutdown_duration_s': 0.5,
            'max_throat_temp_k': 500 + 273.15, # Celsius to Kelvin
        },
        'tanks_config': {
            'target_fuel_pressure_pa': 200 * 6894.75729, # PSI to pascals
            'target_oxidizer_pressure_pa': 200 * 6894.75729, # PSI to pascals
        },
        'telemetry_rate_s': 0.02,
    })

    ecu_sensor_config: dict = field(default_factory=lambda: {
        'FuelTankPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 200 * 6894.75729, # PSI to pascals
        },
        'OxidizerTankPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 200 * 6894.75729, # PSI to pascals
        },
        'IgniterChamberPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'IgniterFuelInjectorPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'IgniterOxidizerInjectorPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'EngineChamberPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 10000, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'EngineFuelInjectorPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 10000, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'EngineOxidizerInjectorPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 10000, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'FuelPumpOutletPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
        'OxidizerPumpOutletPressure': {
            'rate': 0.001, # Seconds
            'std_dev': 3500, # Pascals
            'pressure_min': 0, # Pascals
            'pressure_max': 500 * 6894.75729, # PSI to pascals
        },
    })

    def is_time_before_thrust(self, t):
        return t < self.thrust_wait

    def is_time_during_thrust(self, t):
        return t > self.thrust_wait and t < self.thrust_wait + self.thrust_time

    def is_time_after_thrust(self, t):
        return t > self.thrust_wait + self.thrust_time
