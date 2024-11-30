import software_in_loop as sil
from simulation.pysim.simulation.igniter import IgniterSimulation
import pytest

from simulation.tests.test_ecu_tank import tanks_pressurized

@pytest.fixture
def igniter_sim(endrega_config):
    sim_config = {
        "ecu_update_rate": 0.001,
        "sim_update_rate": 0.0005,
    }

    simulation = IgniterSimulation(sim_config)
    project_config = endrega_config

    simulation.initialize(project_config, False) # False for no realtime

    return simulation

def test_igniter_init_state(igniter_sim):
    assert igniter_sim.ecu['igniter_state'] == 'Idle'

def test_no_startup_with_no_pressurized_tanks(igniter_sim):
    igniter_sim.advance_timestep()

    igniter_sim.mission_ctrl.send_fire_igniter_packet(0)
    igniter_sim.advance_timestep()

    assert not igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] != 'Idle', 3.0)

def test_no_ignition_without_spark(igniter_sim):
    igniter_sim.igniter_dynamics.allow_ignition = False
    igniter_sim.advance_timestep()

    igniter_sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    igniter_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    assert igniter_sim.simulate_until(lambda s: tanks_pressurized(s), 5.0)

    igniter_sim.mission_ctrl.send_fire_igniter_packet(0)
    assert igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Startup', 1.0)

    start_t = igniter_sim.t
    while igniter_sim.t - start_t < 5.0:
        igniter_sim.advance_timestep()

        assert igniter_sim.ecu['igniter_state'] in ['Startup', 'Idle', 'Shutdown']
        assert igniter_sim.igniter_dynamics.chamber_pressure_pa < sil.ATMOSPHERIC_PRESSURE_PA * 1.1

def test_ignition(igniter_sim):
    igniter_sim.advance_timestep()

    igniter_sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    igniter_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    assert igniter_sim.simulate_until(lambda s: tanks_pressurized(s), 5.0)

    igniter_sim.mission_ctrl.send_fire_igniter_packet(0)
    assert igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Startup', 1.0)
    assert igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Firing', 2.0)
    assert igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Shutdown', 2.0)
    assert igniter_sim.simulate_until(lambda s: s.ecu['igniter_state'] == 'Idle', 3.0)

def test_unstable_pressure_no_ignition(igniter_sim):
    igniter_sim.advance_timestep()

    def combustion_modifier(pressure_pa: float) -> float:
        nonlocal igniter_sim

        return sil.ATMOSPHERIC_PRESSURE_PA # pressure_pa * (math.sin(sim.t * 1000.0) * 0.5 + 0.5)

    igniter_sim.igniter_dynamics.set_combustion_pressure_modifier(combustion_modifier)

    igniter_sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
    igniter_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)
    assert igniter_sim.simulate_until(lambda s: tanks_pressurized(s), 5.0)

    def no_ignition_assert(igniter_sim: IgniterSimulation):
        assert igniter_sim.ecu['igniter_state'] != 'Firing'
        assert igniter_sim.igniter_dynamics.chamber_pressure_pa < sil.ATMOSPHERIC_PRESSURE_PA * 1.1

    igniter_sim.mission_ctrl.send_fire_igniter_packet(0)
    assert igniter_sim.simulate_until_with_assert(
        condition_fn=lambda s: s.ecu['igniter_state'] == 'Shutdown',
        assert_fn=no_ignition_assert,
        timeout_s=5.0,
    )
