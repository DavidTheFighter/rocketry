def generate_config():
    config = {}

    config["hardwareConfig"] = {
        "fuelConfig": {
            "ventDiameterMeters": 0.0025,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.005,
            "propellantMassKg": 4.0,
            "propellantLiquid": {
                "name": "75% IPA",
                "densityKgPerM3": 846.0,
                "vaporPressurePa": 4.1,
            },
            "ullageGas": {
                "name": "N2O",
                "molecularWeightKg": 0.04401,
                "specificHeatRatio": 0.875,
            },
        },
        "oxidizerConfig": {
            "ventDiameterMeters": 0.005,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.03,
            "propellantMassKg": 10.0,
            "propellantLiquid": {
                "name": "N2O",
                "densityKgPerM3": 1220.0,
                "vaporPressurePa": 5137000.0,
            },
            "ullageGas": {
                "name": "N2O",
                "molecularWeightKg": 0.04401,
                "specificHeatRatio": 0.875,
            },
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
        }
    }

    config["softwareConfig"] = {
        "ecu0": {
            "fuel_tank_config": {
                'press_valve': None,
                'vent_valve': "FuelVentValve",
                'fill_valve': "FuelFillValve",
                'press_min_threshold_pa': 500.0 * 6894.76, # PSI to Pascals
                'press_max_threshold_pa': 900.0 * 6894.76, # PSI to Pascals
            },
            "oxidizer_tank_config": {
                'press_valve': None,
                'vent_valve': "OxidizerVentValve",
                'fill_valve': "OxidizerFillValve",
                'press_min_threshold_pa': 500.0 * 6894.76, # PSI to Pascals
                'press_max_threshold_pa': 900.0 * 6894.76, # PSI to Pascals
            },
        }
    }

    return config
