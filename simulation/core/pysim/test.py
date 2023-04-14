import software_in_loop
import time
import math
from config import *
from noise import *

# [x, y, z] - [east, height, north]

g = 9.81

def main():
    fcu = software_in_loop.SoftwareInLoop()

    vehicle_position = [0, 0, 0]
    vehicle_velocity = [0, 0, 0]
    vehicle_acceleration = [0, 0, 0]

    dt = 0.0001
    t = 0.0

    while True:
        if t > THRUST_TIME and vehicle_position[1] < 0:
            print("Vehicle landed")
            break

        if t < THRUST_TIME:
            vehicle_acceleration[1] = THRUST / VEHICLE_MASS - g
        else:
            vehicle_acceleration[1] = -g

        vehicle_velocity[0] += vehicle_acceleration[0] * dt
        vehicle_velocity[1] += vehicle_acceleration[1] * dt
        vehicle_velocity[2] += vehicle_acceleration[2] * dt

        vehicle_position[0] += vehicle_velocity[0] * dt
        vehicle_position[1] += vehicle_velocity[1] * dt
        vehicle_position[2] += vehicle_velocity[2] * dt

        if math.fmod(t, ACCEL_RATE) < 1e-4:
            fcu.update_acceleration(accel_noise(vehicle_acceleration))

        if math.fmod(t, GPS_RATE) < 1e-4:
            fcu.update_gps(gps_noise(vehicle_position))

        if int(t * 10000) % 1000 == 0:
            print("{:.2f}s: ".format(t), end="")
            print("[{:.2f}, {:.2f}, {:.2f}]".format(vehicle_position[0], vehicle_position[1], vehicle_position[2]), end="")
            print(" [{:.2f}, {:.2f}, {:.2f}]".format(vehicle_velocity[0], vehicle_velocity[1], vehicle_velocity[2]), end="")
            print(" [{:.2f}, {:.2f}, {:.2f}]".format(vehicle_acceleration[0], vehicle_acceleration[1], vehicle_acceleration[2]))
            time.sleep(0.025)

        t += dt

if __name__ == "__main__":
    main()