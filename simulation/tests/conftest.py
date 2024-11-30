import pytest
from simulation.pysim.simulation.simulation import build_config

ENDREGA_CONFIG = build_config(
        "../projects/endrega/gen_endrega_config.py",
        "../projects/endrega/endrega_config.json",
    )

@pytest.fixture
def endrega_config():
    return ENDREGA_CONFIG.copy()
