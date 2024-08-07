import software_in_loop as sil
import time
import math
from threading import Thread
import numpy as np

from pysim.config import SimConfig
from pysim.noise import *
from pysim.replay import SimReplay
from pysim.vehicle_components import VehicleComponents
from pysim.simulation.simulation import SimulationBase

# [x, y, z] - [east, height, north]

class SolidRocketSimulation(SimulationBase):
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False) -> None:
        super().__init__(config, loggingQueue, log_to_file)

        self.radio_network = sil.SilNetwork([10, 0, 0, 0])

        self.fcu_radio_phy = sil.SilNetworkPhy(self.radio_network)
        self.fcu_radio_iface = sil.SilNetworkIface(self.fcu_radio_phy)

        self.mission_ctrl_radio_phy = sil.SilNetworkPhy(self.radio_network)
        self.mission_ctrl_radio_iface = sil.SilNetworkIface(self.mission_ctrl_radio_phy)

        self.fcu = sil.FcuSil([self.fcu_radio_iface])
        self.mission_ctrl = sil.MissionControl([self.mission_ctrl_radio_iface])

        self.dynamics = sil.SilVehicleDynamics()
        self.logger = sil.Logger([self.radio_network])
        self.config = config
        self.vehicle_components = VehicleComponents(self.fcu, self.dynamics, self.config)
        self.logger.dt = self.config.sim_update_rate
        self.dt = self.config.sim_update_rate
        self.t = 0.0

        self.fcu.update_fcu_config(self.config.fcu_config)

    # Meant as an easy way for tests to simulate until in Idle state,
    # leaving one place that has this logic instead of every test
    def simulate_until_idle(self):
        self.simulate_for(self.config.fcu_config['calibration_duration'] + self.config.fcu_update_rate)

        assert self.fcu['vehicle_state'] == 'Idle'

    def simulate_until_ascent(self, ascent_timeout_s=5.0):
        self.simulate_until_idle()

        self.mission_ctrl.send_arm_vehicle_packet()
        self.simulate_for(self.config.fcu_update_rate)

        self.mission_ctrl.send_ignite_solid_motor_packet()
        self.simulate_for(self.config.fcu_update_rate)

        assert self.fcu['vehicle_state'] == 'Ignition'

        start_time = self.t
        while self.t - start_time < ascent_timeout_s:
            self.advance_timestep()
            if self.fcu['vehicle_state'] == 'Ascent':
                print(f'Took {self.t - start_time} seconds to detect ignition')
                return

        assert self.fcu['vehicle_state'] == 'Ascent'

    def advance_timestep(self):
        self.fcu.update_timestamp(self.t)

        self.mission_ctrl.update(self.dt)
        self.dynamics.update(self.dt)
        self.vehicle_components.update(self.t, self.dt)

        if math.fmod(self.t, self.config.accel_data_rate) <= self.dt:
            accel = accel_noise(self.dynamics.acceleration_body_frame, self.config)
            self.fcu.update_acceleration(accel)

        if math.fmod(self.t, self.config.baro_data_rate) <= self.dt:
            altitude = baro_noise(self.dynamics.position[1], self.config)
            self.fcu.update_barometric_altitude(altitude)

        if math.fmod(self.t, self.config.angular_data_rate) <= self.dt:
            angular_velocity = gyro_noise(self.dynamics.angular_velocity, self.config)
            self.fcu.update_angular_velocity(angular_velocity)

        if math.fmod(self.t, self.config.gps_data_rate) <= self.dt:
            self.fcu.update_gps(gps_noise(self.dynamics.position, self.config))

        # Apply random noise on ascent to simulate wind
        if math.fmod(self.t, self.config.fcu_update_rate) <= self.dt:
            if not self.config.is_time_before_thrust(self.t):
                noise = [np.random.normal(0.0, 0.1) for _ in range(3)]
                self.dynamics.angular_forces = noise
            else:
                self.dynamics.angular_forces = [0.0]*3

            self.fcu.update(self.config.fcu_update_rate)

        self.logger.log_common_data()
        self.logger.log_fcu_data(self.fcu)
        self.logger.log_dynamics_data(self.dynamics)

        if self.dynamics.position[1] < -1.0:
            print("Vehicle landed at {:.6f} s".format(self.t))
            self.t += self.dt
            return False

        self.t += self.dt

        return True

    def replay(self):
        replay = SimReplay(self.config, self.logger)
        replay.replay(self.logging)

if __name__ == "__main__":
    def solid_rocket_app():
        config = SimConfig()
        config.sim_update_rate = 0.0005 # Seconds

        armed = False
        ignited = False

        def tick_callback(sim: SolidRocketSimulation):
            nonlocal armed, ignited

            if sim.fcu['vehicle_state'] == 'Idle' and not armed:
                armed = True
                sim.mission_ctrl.send_arm_vehicle_packet()

            if sim.fcu['vehicle_state'] == 'Armed' and not ignited:
                ignited = True
                sim.mission_ctrl.send_ignite_solid_motor_packet()

            if sim.fcu['vehicle_state'] == 'Landed':
                return True

            return True

        sil.simulate_app_replay(SolidRocketSimulation(config), tick_callback)

    solid_rocket_app()
