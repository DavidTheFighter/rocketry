from simulation.pysim.app import Simulation
from simulation.pysim.config import SimConfig
from software_in_loop import convert_pressure_to_altitude
import pytest

@pytest.fixture
def config():
    return SimConfig()

def test_fcu_init_state(config):
    sim = Simulation(config)

    assert sim.fcu['vehicle_state'] == 'Calibrating'

def test_sim_until_idle(config):
    sim = Simulation(config)
    sim.simulate_until_idle()
    assert sim.fcu['vehicle_state'] == 'Idle'

# def test_sim_until_arm(config)
# def test_arming_safety(Config)

# Tests that config is indeed updated via packet
def test_send_fcu_config(config):
    sim = Simulation(config)

    config.fcu_config['telemetry_rate'] = 0.05
    sim.fcu.update_fcu_config(config.fcu_config)
    sim.simulate_for(config.fcu_update_rate)
    assert abs(sim.fcu.fcu_config()['telemetry_rate'] - 0.05) < 1e-3

    config.fcu_config['telemetry_rate'] = 0.1
    sim.fcu.update_fcu_config(config.fcu_config)
    sim.simulate_for(config.fcu_update_rate)
    assert abs(sim.fcu.fcu_config()['telemetry_rate'] - 0.1) < 1e-3


@pytest.mark.parametrize("accel_bias,gyro_bias,baro_bias", [
    (-0.75, -0.75, -0.75), (0.75, 0.75, 0.75), (0.0, 0.0, 0.0),
    (-0.75, 0.75, 0.2), (0.2, -0.4, 0.0), (0.1, -0.2, 0.3),
])
def test_fcu_calibration(config, accel_bias, gyro_bias, baro_bias):
    config.accel_bias = accel_bias
    config.gyro_bias = gyro_bias
    config.baro_bias = baro_bias
    config.accel_noise = 0.1
    config.gyro_noise = 0.1
    config.baro_noise = 0.1
    sim = Simulation(config)

    assert sim.fcu['vehicle_state'] == 'Calibrating'
    sim.simulate_for(config.fcu_config['calibration_duration'] - config.fcu_update_rate)
    assert sim.fcu['vehicle_state'] == 'Calibrating'
    sim.simulate_for(config.fcu_update_rate)
    assert sim.fcu['vehicle_state'] == 'Idle'

    calibration = sim.fcu.state_vector()['sensor_calibration']
    print(calibration)
    barometric_altitude = convert_pressure_to_altitude(-calibration['barometer_pressure'], 20.0)

    assert vec_f_eq(calibration['accelerometer'], -config.accel_bias, config.accel_noise)
    assert vec_f_eq(calibration['gyroscope'], -config.gyro_bias, config.gyro_noise)
    assert abs(barometric_altitude - config.baro_bias) < config.baro_noise

def vec_f_eq(vec, f, epsilon):
    for i in range(len(vec)):
        if abs(vec[i] - f) > epsilon:
            return False
    return True
