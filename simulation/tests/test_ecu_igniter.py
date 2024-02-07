import software_in_loop as sil
from simulation.pysim.simulation.igniter import IgniterSimulation
from simulation.pysim.config import SimConfig
import pytest

from simulation.tests.test_ecu_tank import tanks_pressurized

@pytest.fixture
def config():
    config = SimConfig()
    config.sim_update_rate = 0.0005 # Seconds
    config.ecu_tank_vent_diamter_m = 0.0075
    return config

def test_igniter_init_state(config):
    sim = IgniterSimulation(config)

    assert sim.ecu['igniter_state'] == 'Idle'

def test_no_startup_with_no_pressurized_tanks(config):
    sim = IgniterSimulation(config)
    sim.advance_timestep()

    sim.mission_ctrl.send_fire_igniter_packet(0)
    sim.advance_timestep()

    assert not sim.simulate_until(lambda s: s.ecu['igniter_state'] != 'Idle', 3.0)

def test_no_ignition_without_spark(config):
    sim = IgniterSimulation(config)
    sim.glue.test_allow_igniter_ignition = False
    sim.advance_timestep()

    sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    assert sim.simulate_until(lambda s: tanks_pressurized(s), 5.0)

    sim.mission_ctrl.send_fire_igniter_packet(0)
    assert sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Startup', 1.0)

    start_t = sim.t
    while sim.t - start_t < 5.0:
        sim.advance_timestep()

        assert sim.ecu['igniter_state'] in ['Startup', 'Idle', 'Shutdown']
        assert sim.igniter_dynamics.chamber_pressure_pa < sil.ATMOSPHERIC_PRESSURE_PA * 1.1

def test_ignition(config):
    sim = IgniterSimulation(config)
    sim.advance_timestep()

    sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    assert sim.simulate_until(lambda s: tanks_pressurized(s), 5.0)

    sim.mission_ctrl.send_fire_igniter_packet(0)
    assert sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Startup', 1.0)
    assert sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Firing', 2.0)
    assert sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Shutdown', 2.0)
    assert sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Idle', 3.0)

