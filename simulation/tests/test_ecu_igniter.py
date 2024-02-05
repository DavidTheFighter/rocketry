from simulation.pysim.simulation.igniter import IgniterSimulation
from simulation.pysim.config import SimConfig
import pytest

@pytest.fixture
def config():
    return SimConfig()

def test_igniter_init_state(config):
    sim = IgniterSimulation(config)

    assert sim.ecu['igniter_state'] == 'Idle'

