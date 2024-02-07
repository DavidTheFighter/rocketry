import time, math, abc

from pysim.config import SimConfig
from pysim.replay import SimReplay

class SimulationBase:
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False):
        self.config = config
        self.logging = loggingQueue
        self.log_to_file = log_to_file

        self.dt = self.config.sim_update_rate
        self.t = 0.0

    @abc.abstractmethod
    def advance_timestep(self):
        return

    def simulate_for(self, duration_s):
        start_time = self.t

        while self.t - start_time < duration_s:
            if not self.advance_timestep():
                break

    def simulate_until(self, condition, timeout_s):
        start_time = self.t

        while self.t - start_time < timeout_s:
            if not self.advance_timestep():
                return False

            if condition(self):
                break

        return condition(self)

    def replay(self):
        replay = SimReplay(self.config, self.logger)
        replay.replay(self.logging)
