import sys
import re
import math
import json
import matplotlib.pyplot as plt

def main():
    # Load file from argv
    f = open(sys.argv[1], 'r')

    sensor_data = {
        'Accelerometer': [],
        'Gyroscope': [],
        'Barometer': [],
    }

    # i = 0
    for line in f:
        line = line.strip()
        if len(line) == 0:
            continue

        name = line.split('{')[0].strip()
        data = line.replace(name, '')

        # print(name)

        if name == 'Gyroscope':
            raw_data = re.search(r'angular_velocity[\s]*:[^\n]*\{[^\n]+\},', data).group(0)
            vars = re.findall(r'[\w]+:[\s]*([-.\d]+)', raw_data)

            # print('\t', data)
            # print('\t', raw_data)
            # print('\t', vars)

            sensor_data[name].append([float(var) for var in vars])
        elif name == 'Accelerometer':
            raw_data = re.search(r'acceleration[\s]*:[^\n]*\{[^\n]+\},', data).group(0)
            vars = re.findall(r'[\w]+:[\s]*([-.\d]+)', raw_data)

            # print('\t', data)
            # print('\t', raw_data)
            # print('\t', vars)

            sensor_data[name].append([float(var) for var in vars])
        elif name == 'Barometer':
            var = re.findall(r'pressure:[\s]*([.\d]+)', data)[0]
            # print('\t', data)
            # print('\t', var)

            sensor_data[name].append(float(var))

        # if i > 10:
        #     break
        # i += 1

    # print(sensor_data)

    medians = {}

    # Find median of all measurements
    for name in sensor_data:
        if name == 'Barometer':
            measurements = sensor_data[name]
            measurements.sort()
            median = measurements[int(len(measurements) / 2)]

            medians[name] = median
        elif name == 'Accelerometer' or name == 'Gyroscope':
            measurements = sensor_data[name]
            median = [0, 0, 0]
            for i in range(3):
                measurements.sort(key=lambda x: x[i])
                median[i] = measurements[int(len(measurements) / 2)][i]

            medians[name] = median

    print('Medians: ', medians)

    # Find noise of all measurements
    noise = { sensor: [] for sensor in sensor_data.keys() }

    for name in sensor_data:
        if name == 'Barometer':
            measurements = sensor_data[name]
            median = medians[name]
            for i in range(len(measurements)):
                noise[name].append(measurements[i] - median)

        elif name == 'Accelerometer' or name == 'Gyroscope':
            measurements = sensor_data[name]
            median = medians[name]
            for i in range(len(measurements)):
                noise[name].append([measurements[i][j] - median[j] for j in range(3)])

    # Dump as JSON
    with open('sensors.json', 'w') as outfile:
        json.dump(noise, outfile)

if __name__ == "__main__":
    main()
