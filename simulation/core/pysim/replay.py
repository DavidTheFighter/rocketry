import time
from pysim.config import *
import numpy as np
from software_in_loop import Logger

class SimReplay():
    def __init__(self, logger: Logger):
        self.logger = logger

    def replay(self, logging):
        lt = time.time()
        dt = self.logger.dt

        print(self.logger.num_timesteps())
        for i in range(self.logger.num_timesteps()):
            t = float(i) * dt

            if i % (int(FCU_UPDATE_RATE / dt)) == 0:
                print('Time {:.2f} s'.format(t))

                data = self.logger.grab_timestep_frame(i)

                # Do some calcs to make the webview easier
                data['fcu_position'] = data['position']
                data['fcu_velocity'] = data['velocity']
                data['fcu_acceleration'] = data['acceleration']
                data['fcu_orientation'] = data['orientation']
                data['fcu_angular_velocity'] = data['angular_velocity']
                data['fcu_angular_acceleration'] = data['angular_acceleration']

                data['dposition'] = [x2 - x1 for x1, x2 in zip(data['position'], data['fcu_position'])]
                data['dvelocity'] = [x2 - x1 for x1, x2 in zip(data['velocity'], data['fcu_velocity'])]
                data['dacceleration'] = [x2 - x1 for x1, x2 in zip(data['acceleration'], data['fcu_acceleration'])]
                data['dangular_velocity'] = [x2 - x1 for x1, x2 in zip(data['angular_velocity'], data['fcu_angular_velocity'])]

                data['dorientation'] = [np.degrees(np.arccos(np.clip(np.dot(
                    data['orientation'],
                    data['fcu_orientation']
                ), -1.0, 1.0)))]

                data['sim_data'] = {
                    'time': t,
                    'speed': pow(sum([pow(s, 2.0) for s in data['velocity']]), 0.5),
                    'measured_speed': pow(sum([pow(s, 2.0) for s in data['fcu_velocity']]), 0.5),
                    'altitude_agl': data['position'][1],
                    'measured_altitude_agl': data['fcu_position'][1],
                }

                logging.put(data)

                while time.time() < lt + FCU_UPDATE_RATE:
                    pass
                lt = time.time()