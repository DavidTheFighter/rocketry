import software_in_loop as sil
from simulation.pysim.simulation.igniter import IgniterSimulation
from simulation.pysim.config import SimConfig
import pytest

@pytest.fixture
def config():
    config = SimConfig()
    config.ecu_tank_vent_diamter_m = 0.0075

    return config

def tanks_pressurized(sim: IgniterSimulation):
    threshold_pa = sim.config.ecu_tank_pressure_set_point_pa * 0.9
    return sim.fuel_tank_dynamics.tank_pressure_pa > threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa > threshold_pa

def tanks_depressurized(sim: IgniterSimulation):
    threshold_pa = sim.config.ecu_tank_pressure_set_point_pa * 0.25
    return sim.fuel_tank_dynamics.tank_pressure_pa < threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa < threshold_pa

def test_tanks_init_state(config):
    sim = IgniterSimulation(config)
    sim.advance_timestep()
    MAX_PRESSURE_PA = sil.ATMOSPHERIC_PRESSURE_PA + 10

    assert sim.ecu['igniter_state'] == 'Idle'
    assert sim.ecu['binary_valves']['FuelPress'] == False
    assert sim.ecu['binary_valves']['OxidizerPress'] == False
    # Note: Intentionally ignoring vent valve states for now
    assert sim.fuel_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA
    assert sim.oxidizer_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA

    sim.simulate_for(5.0)

    assert sim.ecu['igniter_state'] == 'Idle'
    assert sim.ecu['binary_valves']['FuelPress'] == False
    assert sim.ecu['binary_valves']['OxidizerPress'] == False
    assert sim.fuel_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA
    assert sim.oxidizer_tank_dynamics.tank_pressure_pa < MAX_PRESSURE_PA

def test_tank_press_and_depress(config):
    sim = IgniterSimulation(config)
    sim.advance_timestep()

    sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    sim.advance_timestep()

    assert sim.ecu['fuel_tank_state'] == 'Pressurized'
    assert sim.ecu['oxidizer_tank_state'] == 'Pressurized'
    assert sim.ecu['binary_valves']['FuelPress'] == True
    assert sim.ecu['binary_valves']['OxidizerPress'] == True
    assert sim.ecu['binary_valves']['FuelVent'] == False
    assert sim.ecu['binary_valves']['OxidizerVent'] == False

    assert sim.simulate_until(lambda s: tanks_pressurized(s), 10.0)

    assert sim.ecu['fuel_tank_state'] == 'Pressurized'
    assert sim.ecu['oxidizer_tank_state'] == 'Pressurized'
    assert sim.ecu['binary_valves']['FuelPress'] == True
    assert sim.ecu['binary_valves']['OxidizerPress'] == True
    assert sim.ecu['binary_valves']['FuelVent'] == False
    assert sim.ecu['binary_valves']['OxidizerVent'] == False

    assert sim.fuel_tank_dynamics.tank_pressure_pa > config.ecu_tank_pressure_set_point_pa * 0.9
    assert sim.oxidizer_tank_dynamics.tank_pressure_pa > config.ecu_tank_pressure_set_point_pa * 0.9

    sim.mission_ctrl.send_set_fuel_tank_packet(0, False)
    sim.mission_ctrl.send_set_oxidizer_tank_packet(0, False)
    sim.advance_timestep()

    assert sim.ecu['fuel_tank_state'] == 'Depressurized'
    assert sim.ecu['oxidizer_tank_state'] == 'Depressurized'
    assert sim.ecu['binary_valves']['FuelPress'] == False
    assert sim.ecu['binary_valves']['OxidizerPress'] == False
    assert sim.ecu['binary_valves']['FuelVent'] == True
    assert sim.ecu['binary_valves']['OxidizerVent'] == True

    assert sim.simulate_until(lambda s: tanks_depressurized(s), 5.0)

    assert sim.ecu['fuel_tank_state'] == 'Depressurized'
    assert sim.ecu['oxidizer_tank_state'] == 'Depressurized'
    assert sim.ecu['binary_valves']['FuelPress'] == False
    assert sim.ecu['binary_valves']['OxidizerPress'] == False
    assert sim.ecu['binary_valves']['FuelVent'] == True
    assert sim.ecu['binary_valves']['OxidizerVent'] == True

    assert sim.fuel_tank_dynamics.tank_pressure_pa < config.ecu_tank_pressure_set_point_pa * 0.5
    assert sim.oxidizer_tank_dynamics.tank_pressure_pa < config.ecu_tank_pressure_set_point_pa * 0.5
