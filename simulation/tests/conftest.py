import pytest
from simulation.simulation import build_config

ENDREGA_CONFIG = build_config(
    "projects.endrega.gen_endrega",
)

@pytest.fixture
def endrega_config():
    return ENDREGA_CONFIG.copy()

@pytest.fixture
def generic_ecu_sensor_config():
    sensor_config = {
        "FuelTankPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "OxidizerTankPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "IgniterChamberPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "IgniterFuelInjectorPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "IgniterOxidizerInjectorPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "EngineChamberPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "EngineFuelInjectorPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "EngineOxidizerInjectorPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "FuelPumpOutletPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "OxidizerPumpOutletPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "FuelPumpInletPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "FuelPumpInducerPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "OxidizerPumpInletPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
        "OxidizerPumpInducerPressure": {
            "rate": 0.001, # Seconds
            "std_dev": 10000, # Pascals
            "pressure_min": 0, # Pascals
            "pressure_max": 1000 * 6894.75729, # PSI to pascals
        },
    }

    return sensor_config
