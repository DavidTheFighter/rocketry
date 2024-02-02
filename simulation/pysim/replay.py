import socket
import time
from pysim.config import *
import numpy as np
from software_in_loop import Logger

MISSION_CTRL_PORT = 25560

class SimReplay():
    def __init__(self, config: SimConfig, logger: Logger):
        self.logger = logger
        self.config = config

    def replay(self, logging):
        lt = time.time()
        dt = self.logger.dt

        udp_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

        packet_accum = []

        for i in range(self.logger.num_timesteps()):
            t = float(i) * dt

            packet_accum += self.logger.get_network_packet_bytes(i)

            if i % (int(self.config.fcu_update_rate / dt)) == 0:
                data = self.logger.grab_timestep_frame(i)

                if i % (int(0.1 / dt)) == 0:
                    print('Time {:.2f} s'.format(t))

                if self.logger.has_fcu_logs():
                    # Do some calcs to make the webview easier
                    data['fcu_position'] = data['fcu_telemetry']['position']
                    data['fcu_velocity'] = data['fcu_telemetry']['velocity']
                    data['fcu_acceleration'] = data['fcu_telemetry']['acceleration']
                    data['fcu_orientation'] = data['fcu_telemetry']['orientation']
                    data['fcu_angular_velocity'] = data['fcu_telemetry']['angular_velocity']
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

                while len(packet_accum) > 0:
                    packet_data = bytearray(packet_accum.pop(0))
                    udp_socket.sendto(packet_data, ("localhost", MISSION_CTRL_PORT))

                while time.time() < lt + self.config.fcu_update_rate:
                    pass
                lt = time.time()

        # dev_stats_frames = self.logger.get_dev_stat_frames()
        # for frame in dev_stats_frames:
        #     print("Dev stats: {}".format(frame))
