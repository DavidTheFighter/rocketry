import time, abc, json, subprocess, typing, importlib

from simulation.pysim.replay import SimReplay

class SimulationBase:
    def __init__(
            self,
            sim_config: dict,
        ):
        self.sim_config = sim_config

        self.dt = self.sim_config["sim_update_rate"]
        self.t = 0.0

        self.time_since_last_realtime_wait = 0
        self.last_realtime_wait_time = time.time()

    @abc.abstractmethod
    def initialize(self, project_config: dict, realtime: bool):
        return

    @abc.abstractmethod
    def advance_timestep(self):
        return

    def simulate_for(self, duration_s):
        start_time = self.t

        while self.t - start_time < duration_s:
            if not self.advance_timestep():
                break

    def simulate_until(self, condition_fn, timeout_s):
        start_time = self.t

        while self.t - start_time < timeout_s:
            if not self.advance_timestep():
                return False

            if condition_fn(self):
                break

        return condition_fn(self)

    def simulate_assert(self, assert_fn, duration_s):
        start_time = self.t

        while self.t - start_time < duration_s:
            if not self.advance_timestep():
                break

            assert_fn(self)

    def simulate_until_with_assert(self, condition_fn, assert_fn, timeout_s):
        start_time = self.t

        while self.t - start_time < timeout_s:
            if not self.advance_timestep():
                return False

            assert_fn(self)

            if condition_fn(self):
                break

        return condition_fn(self)

    def realtime_wait(self):
        if self.time_since_last_realtime_wait >= 0.01:
            now = time.time()
            while time.time() - self.last_realtime_wait_time <= 0.01:
                pass
            elapsed = time.time() - now
            self.last_realtime_wait_time = time.time() + (0.01 - elapsed)
            self.time_since_last_realtime_wait -= 0.01

        self.time_since_last_realtime_wait += self.dt

    def replay(self):
        replay = SimReplay(self.sim_config["replay_update_rate"], self.logger)
        replay.replay()

def build_sim_from_argv(simulation: SimulationBase, argv: typing.List[str]):
    realtime = "-r" in argv

    if len(argv) < 2:
        print(f"Usage: python {simulation.__name__} <config gen script>")
        return

    project_config_gen_module = argv[1]
    project_config = build_config(project_config_gen_module)

    simulation.initialize(project_config, realtime)

def build_config(project_config_gen_module: str) -> dict:
    config_gen_module = importlib.import_module(project_config_gen_module)
    config = config_gen_module.generate_config()

    return config
