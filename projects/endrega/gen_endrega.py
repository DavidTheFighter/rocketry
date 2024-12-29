from util import deep_update

import projects.endrega.gen_sim_tank
import projects.endrega.gen_engine

def generate_config():
    config = {}

    deep_update(config, projects.endrega.gen_sim_tank.generate_config())
    deep_update(config, projects.endrega.gen_engine.generate_config())

    return config
