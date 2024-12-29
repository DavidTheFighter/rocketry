def generate_config():
    config = {}

    config["hardwareConfig"] = {
        "igniterConfig": {
            "fuelInjectorDiameterMeters": 0.0003302,
            "fuelInjectorCd": 0.75,
            "oxidizerInjectorDiameterMeters": 0.0003302,
            "oxidizerInjectorCd": 0.75,
            "throatDiameterMeters": 0.004
        },
        "fuelPumpConfig": {
            "setPointPsi": 500.0
        },
        "oxidizerPumpConfig": {
            "setPointPsi": 500.0
        },
        "engineConfig": {
            "usePump": False,
            "fuelInjectorDiameterMeters": 0.00373,
            "fuelInjectorCd": 0.75,
            "oxidizerInjectorDiameterMeters": 0.003792,
            "oxidizerInjectorCd": 0.75,
            "throatDiameterMeters": 0.03
        },
        "ecuSensorConfig": {
            "IgniterChamberPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "IgniterFuelInjectorPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "IgniterOxidizerInjectorPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "EngineChamberPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 10000, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "EngineFuelInjectorPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 10000, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "EngineOxidizerInjectorPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 10000, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "FuelPumpOutletPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
            "OxidizerPumpOutletPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 500 * 6894.75729, # PSI to pascals
            },
        }
    }

    config["softwareConfig"] = {
        "ecu0": {
            "engine_config": {
                "use_pumps": False,
                "fuel_injector_pressure_setpoint_pa": 500.0 * 6894.76, # PSI to Pascals
                "fuel_injector_startup_pressure_tolerance_pa": 25.0 * 6894.76, # PSI to Pascals
                "fuel_injector_running_pressure_tolerance_pa": 100.0 * 6894.76, # PSI to Pascals
                "oxidizer_injector_pressure_setpoint_pa": 500.0 * 6894.76, # PSI to Pascals
                "oxidizer_injector_startup_pressure_tolerance_pa": 25.0 * 6894.76, # PSI to Pascals
                "oxidizer_injector_running_pressure_tolerance_pa": 100.0 * 6894.76, # PSI to Pascals
                "engine_target_combustion_pressure_pa": 300.0 * 6894.76, # PSI to Pascals
                "engine_combustion_pressure_tolerance_pa": 200.0 * 6894.76, # PSI to Pascals
                "pump_startup_timeout_s": 1.0,
                "igniter_startup_timeout_s": 1.0,
                "engine_startup_timeout_s": 1.0,
                "engine_firing_duration_s": 10.0,
                "engine_shutdown_duration_s": 0.5,
            },
            "igniter_config": {
                "startup_timeout_s": 1.0,
                "startup_pressure_threshold_pa": 30.0 * 6894.76, # PSI to Pascals
                "startup_stable_time_s": 0.25,
                "test_firing_duration_s": 0.75,
                "shutdown_duration_s": 0.5,
                "max_throat_temp_k": 500.0,
            },
            "telemetry_rate_s": 0.02,
        }
    }

    return config
