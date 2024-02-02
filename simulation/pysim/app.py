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

def simulate_app(config: SimConfig, simulation_class_name: str):
    data_queue = multiprocessing.Queue()
    process = multiprocessing.Process(target=pysim.server.process_func, args=(data_queue,))
    process.daemon = True
    process.start()

    time.sleep(1)

    sim = simulation_class_name(config, data_queue, log_to_file=True)
    print("Simulating...")
    sim.simulate_until_done()
    print("Done! Replaying")
    sim.replay()
    print("Done! Exiting...")

if __name__ == "__main__":
    print("This file is not meant to be run directly")
