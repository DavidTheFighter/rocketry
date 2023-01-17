# ECUTelemtryData { ecu_data: ECUDataFrame { time: 0.0, igniter_state: Idle, sensor_states: [0, 0, 0, 398, 0], valve_states: [0, 0, 0, 0], sparking: false }, avg_loop_time: 0.00009406, max_loop_time: 0.000000000000000000000000000000000000000023704 }
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
import os
import sys
import json

sensor_offsets = [0, 1, 2, 3, 5]
sensor_mappings = {}

with open('hardware.json') as f:
    hardware_config = json.load(f)

for mapping in hardware_config['sensor_mappings']:
    sensor_mappings[mapping['index']] = mapping['config']

data_end_time = 6
visual_start_time = 1.5
visual_end_time = 4.5
data_decimation_pct = 1.0 # 0.0227

state_colors = {
    'Idle': 'white',
    'StartupGOxLead': 'cyan',
    'StartupIgnition': 'darkorange',
    'Firing': 'red',
    'Shutdown': 'yellow',
}
sensor_colors = ['orange', 'cyan', 'red', 'green', '#A9A9A9']

files_to_plot = []

if len(sys.argv) == 1:
    files = list(filter(lambda x: ".csv" in x, sorted(os.listdir('.'), reverse=True)))
    files_to_plot += [files[0]]
else:
    if sys.argv[1] == 'all':
        files = list(filter(lambda x: ".csv" in x, sorted(os.listdir('.'))))
        files_to_plot += files
    else:
        files_to_plot += [sys.argv[1]]

def moving_average(x, w):
    mv = list(np.convolve(x, np.ones(w), 'valid') / w)

    mv = [y for y in x[:(w//2)]] + mv
    mv = mv + [y for y in x[len(x) - w//2:]]

    return mv

for log_file_name in files_to_plot:
    print(log_file_name)

    file = open(log_file_name, 'r')
    file_lines = file.readlines()

    # Parse the CSV file

    titles = file_lines[0].strip().split(",")
    sensors = [[], [], [], [], []]
    igniter_states = []
    xpoints = []

    parse_start = max(int((len(file_lines) - 1) * ((visual_start_time) / (data_end_time))), 1)
    parse_end = int((len(file_lines) - 1) / (1.0 + (data_end_time - visual_end_time) / (data_end_time)))
    timestep = (visual_end_time - visual_start_time) / (parse_end - parse_start)

    print("Parsing from {} to {} of {} data points".format(parse_start, parse_end, len(file_lines) - 1))

    startup_time = None

    t = 0
    for line in file_lines[parse_start:parse_end:int(1 / data_decimation_pct)]:
        values = line.strip().split(",")

        igniter_states += [values[0]]

        for i in range(len(sensor_offsets)):
            si = sensor_offsets[i]
            raw = float(values[3 + si])
            lerp = (raw - sensor_mappings[si]['premin']) / (sensor_mappings[si]['premax'] - sensor_mappings[si]['premin'])
            value = lerp * (sensor_mappings[si]['postmax'] - sensor_mappings[si]['postmin']) + sensor_mappings[si]['postmin']

            if 'calibration' in sensor_mappings[si]:
                calib = sensor_mappings[si]['calibration']
                value += calib['x0'] + calib['x1'] * value + calib['x2'] * value**2 + calib['x3'] * value**3

            sensors[i] += [value]

        if startup_time == None and igniter_states[-1] != 'Idle':
            startup_time = len(igniter_states) * timestep / data_decimation_pct

        xpoints += [t * timestep]
        t += int(1 / data_decimation_pct)

    xpoints = [x - startup_time for x in xpoints]

    # Condition the signals for viewing

    idle_values_end = 0
    for i in range(len(igniter_states)):
        if igniter_states[i] != 'Idle':
            idle_values_end = i
            break

    for i in [0, 1, 2]:
        bias = np.average(sensors[i][:idle_values_end])

        sensors[i] = [value - bias for value in sensors[i]]

    sensors[-1] = moving_average(sensors[-1], 9)

    # Plot the sensors

    step = ((visual_end_time - visual_start_time) / len(igniter_states))
    start_offset = startup_time * step + visual_start_time

    fig, ax = plt.subplots()
    xhandles = []

    for i in range(len(sensors) - 1):
        xhandles += [ax.plot(xpoints, sensors[i], color=sensor_colors[i])[0]]

    ax.set_ylabel('Pressure (PSI)')
    ax.xaxis.set_major_locator(ticker.MultipleLocator(0.1))

    plt.yticks(np.arange(-5, 300, 5))
    plt.grid()

    tempax = ax.twinx()
    tempax_handles = [tempax.plot(xpoints, sensors[-1], color=sensor_colors[-1])[0]]
    tempax.set_ylabel('Temperature (°C)')
    tempax.legend(tempax_handles, [titles[3 + sensor_offsets[-1]]], loc='upper right')

    labels = [titles[3 + sensor_offsets[i]] for i in range(len(sensors) - 1)]
    ax.legend(xhandles, labels, loc='upper left')

    # Color the igniter states

    current_state = igniter_states[0]
    state_start_index = 0
    for i in range(1, len(igniter_states)):
        if igniter_states[i] != current_state or i == len(igniter_states) - 1:
            color = state_colors[current_state] if current_state in state_colors.keys() else 'gray'
            ax.axvspan(xpoints[state_start_index], xpoints[i], facecolor=color, alpha=0.15)

            state_start_index = i
            current_state = igniter_states[i]

    # Do the thing

    plt.show()
