import software_in_loop as sil
from simulation.pysim.simulation.igniter import IgniterSimulation
import pytest

from simulation.pysim.simulation.simulation import build_config

@pytest.fixture
def igniter_sim(endrega_config):
    sim_config = {
        "ecu_update_rate": 0.001,
        "sim_update_rate": 0.0005,
    }

    simulation = IgniterSimulation(sim_config)
    project_config = endrega_config

    project_config["hardwareConfig"]["fuelConfig"]["ventDiameterMeters"] = 0.1
    project_config["hardwareConfig"]["oxidizerConfig"]["ventDiameterMeters"] = 0.1
    project_config["hardwareConfig"]["feedConfig"]["setPointPa"] = 200 * 6894.75729 # PSI to pascals

    simulation.initialize(project_config, False) # False for no realtime

    return simulation

def tanks_pressurized(sim: IgniterSimulation):
    threshold_pa = sim.project_config["hardwareConfig"]["feedConfig"]["setPointPa"] * 0.9
    return sim.fuel_tank_dynamics.tank_pressure_pa > threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa > threshold_pa

def tanks_depressurized(sim: IgniterSimulation):
    threshold_pa = sim.project_config["hardwareConfig"]["feedConfig"]["setPointPa"] * 0.9
    return sim.fuel_tank_dynamics.tank_pressure_pa < threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa < threshold_pa

def test_tanks_init_state(igniter_sim):
    igniter_sim.advance_timestep()
    MAX_PRESSURE_PA = sil.ATMOSPHERIC_PRESSURE_PA + 10

    def assert_idle_state(igniter_sim: IgniterSimulation):
        assert igniter_sim.ecu['igniter_state'] == 'Idle'
        assert igniter_sim.ecu['binary_valves']['FuelPressValve'] == False
        assert igniter_sim.ecu['binary_valves']['OxidizerPressValve'] == False
        # Note: Intentionally ignoring vent valve states for now
        assert igniter_sim.fuel_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA
        assert igniter_sim.oxidizer_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA

    igniter_sim.simulate_assert(assert_idle_state, 5.0)

def test_fuel_tank_press_and_depress(igniter_sim):
    igniter_sim.advance_timestep()

    tank_set_point_pa = igniter_sim.project_config["hardwareConfig"]["feedConfig"]["setPointPa"]

    igniter_sim.mission_ctrl.send_set_fuel_tank_packet(0, True)

    assert igniter_sim.simulate_until(lambda s: s.fuel_tank_dynamics.tank_pressure_pa > tank_set_point_pa * 0.9, 5.0)

    assert igniter_sim.ecu['fuel_tank_state'] == 'Pressurized'
    assert igniter_sim.ecu['oxidizer_tank_state'] == 'Idle'
    assert igniter_sim.ecu['binary_valves']['FuelPressValve'] == True
    assert igniter_sim.ecu['binary_valves']['OxidizerPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['FuelVentValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerVentValve'] == False

    igniter_sim.mission_ctrl.send_set_fuel_tank_packet(0, False)

    assert igniter_sim.simulate_until(lambda s: s.fuel_tank_dynamics.tank_pressure_pa < tank_set_point_pa * 0.5, 5.0)

    assert igniter_sim.ecu['fuel_tank_state'] == 'Depressurized'
    assert igniter_sim.ecu['oxidizer_tank_state'] == 'Idle'
    assert igniter_sim.ecu['binary_valves']['FuelPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['FuelVentValve'] == True
    assert igniter_sim.ecu['binary_valves']['OxidizerVentValve'] == False

def test_oxidizer_tank_press_and_depress(igniter_sim):
    igniter_sim.advance_timestep()

    tank_set_point_pa = igniter_sim.project_config["hardwareConfig"]["feedConfig"]["setPointPa"]

    igniter_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

    assert igniter_sim.simulate_until(lambda s: s.oxidizer_tank_dynamics.tank_pressure_pa > tank_set_point_pa * 0.9, 5.0)

    assert igniter_sim.ecu['fuel_tank_state'] == 'Idle'
    assert igniter_sim.ecu['oxidizer_tank_state'] == 'Pressurized'
    assert igniter_sim.ecu['binary_valves']['FuelPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerPressValve'] == True
    assert igniter_sim.ecu['binary_valves']['FuelVentValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerVentValve'] == False

    igniter_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, False)

    assert igniter_sim.simulate_until(lambda s: s.oxidizer_tank_dynamics.tank_pressure_pa < tank_set_point_pa * 0.5, 5.0)

    assert igniter_sim.ecu['fuel_tank_state'] == 'Idle'
    assert igniter_sim.ecu['oxidizer_tank_state'] == 'Depressurized'
    assert igniter_sim.ecu['binary_valves']['FuelPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerPressValve'] == False
    assert igniter_sim.ecu['binary_valves']['FuelVentValve'] == False
    assert igniter_sim.ecu['binary_valves']['OxidizerVentValve'] == True

