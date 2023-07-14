import software_in_loop
import time
import math
from threading import Thread
import numpy as np

from pysim.config import *
from pysim.dynamics import *
from pysim.noise import *
from pysim.replay import SimReplay

# [x, y, z] - [east, height, north]

class Simulation:
    def __init__(self, loggingQueue) -> None:
        self.fcu = software_in_loop.SoftwareInLoop()
        self.dynamics = software_in_loop.Dynamics()
        self.logger = software_in_loop.Logger()
        self.logger.dt = SIM_UPDATE_RATE
        self.logging = loggingQueue
        self.dt = SIM_UPDATE_RATE
        # self.log = {
        #     'dt': self.dt,
        #     'telemetry': [],
        #     'detailed_state': [],
        #     'position': [],
        #     'velocity': [],
        #     'acceleration': [],
        #     'orientation': [],
        #     'angular_velocity': [],
        #     'angular_acceleration': [],
        # }

    def simulate(self):
        t = 0.0

        self.fcu.update_fcu_config(FCU_CONFIG)

        start_time = time.time()

        while True:
            if t > THRUST_TIME + THRUST_WAIT and self.dynamics.position[1] <= 0.0:
                print("Vehicle landed at {:.6f} s".format(t))
                break

            if t < THRUST_WAIT:
                self.dynamics.motor_thrust = [0.0]*3
                self.dynamics.landed = True
            elif t > THRUST_WAIT and t < THRUST_WAIT + THRUST_TIME:
                thrust = THRUST / VEHICLE_MASS
                thrust_t = (t - THRUST_WAIT) / THRUST_TIME
                thrust *= pow(math.cos(thrust_t * math.pi - math.pi / 2.0), 0.2)
                self.dynamics.motor_thrust = [0.0, thrust, 0.0]
                self.dynamics.landed = False
            else:
                self.dynamics.motor_thrust = [0.0]*3
                self.dynamics.landed = False

            self.dynamics.update(self.dt)

            if math.fmod(t, ACCEL_RATE) <= self.dt:
                accel = accel_noise(self.dynamics.acceleration_body_frame)
                self.fcu.update_acceleration(accel)

            if math.fmod(t, BARO_RATE) <= self.dt:
                altitude = baro_noise(self.dynamics.position[1])
                self.fcu.update_barometric_altitude(altitude)

            if math.fmod(t, ANGULAR_RATE) <= self.dt:
                angular_velocity = angular_vel_noise(self.dynamics.angular_velocity)
                self.fcu.update_angular_velocity(angular_velocity)

            if math.fmod(t, GPS_RATE) <= self.dt and t > THRUST_WAIT:
                self.fcu.update_gps(gps_noise(self.dynamics.position))

            if math.fmod(t, FCU_UPDATE_RATE) <= self.dt:
                self.fcu.update(FCU_UPDATE_RATE)

            self.logger.log_telemetry(self.fcu)
            self.logger.log_detailed_state(self.fcu)
            self.logger.log_position(self.dynamics.position)
            self.logger.log_velocity(self.dynamics.velocity)
            self.logger.log_acceleration(self.dynamics.acceleration_world_frame)
            self.logger.log_orientation(self.dynamics.orientation)
            self.logger.log_angular_velocity(self.dynamics.angular_velocity)
            self.logger.log_angular_acceleration(self.dynamics.angular_acceleration)

            t += self.dt

        # plt.plot(np.arange(len(vehicle_accels)), vehicle_accels)
        # plt.title('title name')
        # plt.xlabel('x_axis name')
        # plt.ylabel('y_axis name')
        # plt.show()

        print("Simulation took {:.2f} s".format(time.time() - start_time))

        self.fcu.reset_telemetry()

        Thread(target=self.logger.dump_to_file, daemon=False).start()

    def replay(self):
        replay = SimReplay(self.logger)
        replay.replay(self.logging)

if __name__ == "__main__":
    Simulation().simulate()