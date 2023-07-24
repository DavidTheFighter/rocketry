from pysim.test import Simulation
from threading import Thread
import json
import time
import math
from bottle import route, run, response, app
from bottle_cors_plugin import cors_plugin

app = app()
app.install(cors_plugin('*'))

GRAPH_LEN = 20
DATA_LEN = 150

def dataset_default():
    return {
        'position': [[], [], []],
        'velocity': [[], [], []],
        'acceleration': [[], [], []],
        'angular_velocity': [[], [], []],
        'dposition': [[], [], []],
        'dvelocity': [[], [], []],
        'dorientation': [[]],
        'dangular_velocity': [[], [], []],
        'fcu_position': [[], [], []],
        'fcu_velocity': [[], [], []],
        'fcu_acceleration': [[], [], []],
    }

dataset = dataset_default()

@route('/simdata', method='GET')
def simdata():
    global dataset

    ret_val = json.dumps(dataset)
    dataset = dataset_default()
    return ret_val


def handle_queue(data_queue):
    global dataset

    lastUpdate = time.time()

    while True:
        try:
            data = data_queue.get(block=True, timeout=0.1)

            dataset['telemetry'] = data['telemetry']
            dataset['detailed_state'] = data['detailed_state']
            dataset['sim_data'] = data['sim_data']

            if time.time() - lastUpdate >= GRAPH_LEN / DATA_LEN:
                for key in dataset_default().keys():
                    if key in data:
                        distr(dataset[key], data[key], len(dataset_default()[key]))

                lastUpdate = time.time()
            time.sleep(0)
        except:
            pass

def delapp(lst: list, val):
    lst.pop(0)
    lst.append(val)

def delappvec(lst: list, val, size: int):
    for i in range(size):
        delapp(lst[i], val[i])

def distr(lst: list, val, size: int):
    for i in range(size):
        lst[i].append(val[i])

def process_func(data_queue):
   th = Thread(target=handle_queue, args=[data_queue])
   th.daemon = True
   th.start()
   run(host='0.0.0.0', port=5000, quiet=True)
    # app.run()
