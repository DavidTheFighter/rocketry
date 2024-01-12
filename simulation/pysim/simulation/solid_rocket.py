import software_in_loop as sil
import time
import math
from threading import Thread
import numpy as np

from pysim.config import SimConfig
from pysim.noise import *
from pysim.replay import SimReplay
from pysim.vehicle_components import VehicleComponents

# [x, y, z] - [east, height, north]

class SolidRocketSimulation:
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False) -> None:
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
        self.logging = loggingQueue
        self.dt = self.config.sim_update_rate
        self.t = 0.0
        self.log_to_file = log_to_file

        self.fcu.update_fcu_config(self.config.fcu_config)

    def simulate_until_done(self):
        start_time = time.time()

        self.simulate_until_ascent()

        while self.advance_timestep():
            pass

        print("Simulation took {:.2f} s".format(time.time() - start_time))

        if self.log_to_file:
            self.logger.dump_to_file()

    def simulate_for(self, seconds):
        start_time = self.t

        while self.t - start_time <= seconds:
            self.advance_timestep()

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
        self.mission_ctrl.update_timestep(self.dt)
        self.fcu.update_timestamp(self.t)

        self.dynamics.update(self.dt)
        self.vehicle_components.update(self.t, self.dt)

        if math.fmod(self.t, self.config.accel_data_rate) <= self.dt:
            accel = accel_noise(self.dynamics.acceleration_body_frame, self.config)
            # accel_m = np.linalg.norm(accel)
            # jerk = np.subtract(accel, last_accel) / self.dt
            # jerk_m = np.linalg.norm(jerk)
            # last_accel = accel
            #print(f'Updating acceleration with {accel} noised from {self.dynamics.acceleration_body_frame} at {self.t}')
            # print(f'{self.t}: Accel sense {accel} (|{accel_m}|) with jerk {jerk} (|{jerk_m}|)\n')
            self.fcu.update_acceleration(accel)
            #print()

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

        if math.fmod(self.t, self.config.dev_stats_rate) <= self.dt:
            self.fcu.start_dev_stats_frame()

        self.logger.log_common_data()
        self.logger.log_fcu_data(self.fcu)
        self.logger.log_dynamics_data(self.dynamics)
        # self.logger.log_dev_stats(self.fcu)

        if self.dynamics.position[1] < -1e-3:
            print("Vehicle landed at {:.6f} s".format(self.t))
            self.t += self.dt
            return False

        self.t += self.dt

        return True

    def replay(self):
        replay = SimReplay(self.config, self.logger)
        replay.replay(self.logging)

if __name__ == "__main__":
    print("This file is not meant to be run directly")
