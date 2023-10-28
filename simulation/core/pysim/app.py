import pysim.server
import sys
import time
import multiprocessing
import subprocess
import os
from threading import Thread
from pysim.simulation import Simulation
from queue import Queue
import json
import software_in_loop

from pysim.replay import SimReplay

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

    if not replay:
        sim = Simulation(data_queue)
        print("Simulating...")
        sim.simulate()
        print("Done! Replaying")
        sim.replay()
    else:
        logs = software_in_loop.load_logs_from_file('last-sim.json')

        replay = SimReplay(logs)
        replay.replay(data_queue)

if __name__ == "__main__":
    main()