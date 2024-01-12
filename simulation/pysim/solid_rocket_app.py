import pysim.server
import sys
import time
import multiprocessing
import subprocess
import os
from threading import Thread
from queue import Queue
import json
import software_in_loop

from pysim.simulation.solid_rocket import SolidRocketSimulation
from pysim.replay import SimReplay
from pysim.config import SimConfig

def main():
    data_queue = multiprocessing.Queue()
    process = multiprocessing.Process(target=pysim.server.process_func, args=(data_queue,))
    process.daemon = True
    process.start()

    time.sleep(1)

    replay = False
    if len(sys.argv) >= 2:
        if sys.argv[1].lower() == "replay":
            replay = True

    config = SimConfig()

    if not replay:
        sim = SolidRocketSimulation(config, data_queue, log_to_file=True)
        print("Simulating...")
        sim.simulate_until_done()
        print("Done! Replaying")
        sim.replay()
    else:
        logs = software_in_loop.load_logs_from_file('last-sim.json')

        replay = SimReplay(config, logs)
        replay.replay(data_queue)

if __name__ == "__main__":
    main()