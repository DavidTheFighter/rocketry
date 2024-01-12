from queue import Empty
from threading import Thread
import json
import time
from bottle import route, run, response, app
from bottle_cors_plugin import cors_plugin

app = app()
app.install(cors_plugin('*'))

GRAPH_LENGTH_SECONDS = 10
DATA_LENGTH = 200

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
    # dataset = dataset_default()
    return ret_val


def handle_queue(data_queue):
    global dataset

    lastUpdate = time.time()

    while True:
        try:
            data = data_queue.get(block=True, timeout=0.1)

            dataset['fcu_telemetry'] = data['fcu_telemetry']
            dataset['fcu_debug_info'] = data['fcu_debug_info']
            dataset['sim_data'] = data['sim_data']

            if time.time() - lastUpdate >= GRAPH_LENGTH_SECONDS / DATA_LENGTH:
                for key in dataset_default().keys():
                    if key in data:
                        distr(dataset[key], data[key], len(dataset_default()[key]))

                lastUpdate = time.time()
            time.sleep(0.001)
        except Empty:
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

        if len(lst[i]) > DATA_LENGTH:
            lst[i].pop(0)

def process_func(data_queue):
    th = Thread(target=handle_queue, args=[data_queue])
    th.daemon = True
    th.start()
    run(host='0.0.0.0', port=5000, quiet=True)
    # app.run()
