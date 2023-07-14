import pysim.server
import sys
import time
import multiprocessing
from threading import Thread
from pysim.test import Simulation
from queue import Queue
import json

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
        with open('last-sim.json', 'r') as f:
            logs = json.load(f)

        replay = SimReplay(logs)
        replay.replay(data_queue)

if __name__ == "__main__":
    main()