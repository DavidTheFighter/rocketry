import software_in_loop
import time
import math
from threading import Thread
import numpy as np

from pysim.config import SimConfig
from pysim.noise import *
from pysim.replay import SimReplay
from pysim.vehicle_components import VehicleComponents

# [x, y, z] - [east, height, north]

class Simulation:
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False) -> None:
        self.fcu = software_in_loop.FcuSil()
        self.dynamics = software_in_loop.Dynamics()
        self.logger = software_in_loop.Logger()
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

        self.fcu.send_arm_vehicle_packet()
        self.simulate_for(self.config.fcu_update_rate)

        self.fcu.send_ignite_solid_motor_packet()
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

        if self.dynamics.position[1] < -1e-3:
            print("Vehicle landed at {:.6f} s".format(self.t))
            self.t += self.dt
            return False

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
            self.logger.log_dev_stats(self.fcu)
            self.fcu.start_dev_stats_frame()

        self.logger.log_telemetry(self.fcu)
        self.logger.log_detailed_state(self.fcu)
        self.logger.log_position(self.dynamics.position)
        self.logger.log_velocity(self.dynamics.velocity)
        self.logger.log_acceleration(self.dynamics.acceleration_world_frame)
        self.logger.log_orientation(self.dynamics.orientation)
        self.logger.log_angular_velocity(self.dynamics.angular_velocity)
        self.logger.log_angular_acceleration(self.dynamics.angular_acceleration)

        self.t += self.dt

        return True

    def replay(self):
        replay = SimReplay(self.logger)
        replay.replay(self.logging)

if __name__ == "__main__":
    Simulation().simulate()