import time, math, abc, json, subprocess, typing

from pysim.replay import SimReplay

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

def build_sim_from_argv(
        simulation: SimulationBase,
        argv: typing.List[str],
):
    realtime = "-r" in argv

    if len(argv) < 2:
        print(f"Usage: python {simulation.__name__} <optional gen script> <config_file>")
        return

    if len(argv) == 2:
        project_config_gen_script = None
        project_config_file = argv[1]
    else:
        project_config_gen_script = argv[1]
        project_config_file = argv[2]

    project_config = build_config(
        project_config_gen_script,
        project_config_file,
    )

    simulation.initialize(project_config, realtime)

def build_config(
        project_config_gen_script: str,
        project_config_file: str,
) -> dict:
    if project_config_gen_script is not None:
            subprocess.check_output([
                "python",
                project_config_gen_script,
                project_config_file,
            ])

    with open(project_config_file, "r") as f:
        project_config = json.load(f)

    return project_config
