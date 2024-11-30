import json
import sys

def generate_config(filename):
    config = {}

    config["hardwareConfig"] = {
        "feedConfig": {
            "pressurePa": 2000.0 * 6894.76, # PSI to Pascals,
            "setPointPa": 500.0 * 6894.76, # PSI to Pascals,
            "orificeDiameterMeters": 0.004,
            "orificeCd": 0.6,
            "temperatureKelvin": 293.15,
            "feedGas": {
                "name": "GN2",
                "molecularWeightKg": 0.028,
                "specificHeatRatio": 1.039
            }
        },
        "fuelConfig": {
            "ventDiameterMeters": 0.0025,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.005,
            "fuelLiquid": {
                "name": "75% IPA",
                "densityKgPerM3": 846.0
            }
        },
        "oxidizerConfig": {
            "ventDiameterMeters": 0.0025,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.01,
            "oxidizerLiquid": {
                "name": "LOX",
                "densityKgPerM3": 1141.0
            }
        },
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
            "FuelTankPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 200 * 6894.75729, # PSI to pascals
            },
            "OxidizerTankPressure": {
                "rate": 0.001, # Seconds
                "std_dev": 3500, # Pascals
                "pressure_min": 0, # Pascals
                "pressure_max": 200 * 6894.75729, # PSI to pascals
            },
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
                "engine_firing_duration_s": 4.0,
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
            "tanks_config": {
                'target_fuel_pressure_pa': 500 * 6894.75729, # PSI to pascals
                'target_oxidizer_pressure_pa': 500 * 6894.75729, # PSI to pascals
            },
            "telemetry_rate_s": 0.02
        }
    }

    # Write config to file
    with open(filename, "w") as f:
        f.write(json.dumps(config, indent=4))

    return config

if __name__ == "__main__":
    if len(sys.argv) < 2:
        generate_config("endrega_config.json")
    else:
        generate_config(sys.argv[1])
