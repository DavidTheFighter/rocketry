# ECUTelemtryData { ecu_data: ECUDataFrame { time: 0.0, igniter_state: Idle, sensor_states: [0, 0, 0, 398, 0], valve_states: [0, 0, 0, 0], sparking: false }, avg_loop_time: 0.00009406, max_loop_time: 0.000000000000000000000000000000000000000023704 }
import matplotlib.pyplot as plt
import numpy as np
import os
import sys

sensor_pre_mappings = [[410, 3686], [410, 3686], [410, 3686], [410, 3686], [0, 4095]]
sensor_post_mappings = [[0, 300], [0, 200], [0, 200], [0, 300], [-250, 410]]
sensor_offsets = [0, 1, 2, 3, 5]

data_start_time = -2.0
data_end_time = 4.0
visual_start_time = -0.5
visual_end_time = 2.5

state_colors = {
    'Idle': 'white',
    'StartupGOxLead': 'cyan',
    'StartupIgnition': 'darkorange',
    'Firing': 'red',
    'Shutdown': 'yellow',
}
sensor_colors = ['orange', 'cyan', 'red', 'green', '#A9A9A9']

log_file_name = None

if len(sys.argv) == 1:
    files = os.listdir('.')
    files = list(filter(lambda x: ".csv" in x, sorted(files, reverse=True)))
    log_file_name = files[0]
else:
    log_file_name = sys.argv[1]

print(log_file_name)

file = open(log_file_name, 'r')

file_lines = file.readlines()

# Parse the CSV file

titles = file_lines[0].strip().split(",")
sensors = [[], [], [], [], []]
igniter_states = []

parse_start = int((len(file_lines) - 1) * ((visual_start_time - data_start_time) / (data_end_time - data_start_time)))
parse_end = int((len(file_lines) - 1) / (1.0 + (data_end_time - visual_end_time) / (data_end_time - data_start_time)))

print("Parsing from {} to {} of {} data points".format(parse_start, parse_end, len(file_lines) - 1))

for line in file_lines[parse_start:parse_end]:
    values = line.strip().split(",")

    igniter_states += [values[0]]

    for i in range(5):
        raw = float(values[3 + sensor_offsets[i]])
        lerp = (raw - sensor_pre_mappings[i][0]) / (sensor_pre_mappings[i][1] - sensor_pre_mappings[i][0])
        value = lerp * (sensor_post_mappings[i][1] - sensor_post_mappings[i][0]) + sensor_post_mappings[i][0]

        sensors[i] += [value]

# Plot the sensors

xpoints = np.array(np.arange(data_start_time, data_end_time, step=((data_end_time - data_start_time) / len(igniter_states))))
fig, ax = plt.subplots()
handles = []

for i in range(len(sensors) - 1):
    handles += [ax.plot(xpoints, sensors[i], color=sensor_colors[i])[0]]

tempax = ax.twinx()
handles += [tempax.plot(xpoints, sensors[-1], color=sensor_colors[-1])[0]]

labels = [titles[3 + sensor_offsets[i]] for i in range(len(sensors))]
tempax.legend(handles, labels, loc=1)

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