import math

from software_in_loop import FcuSil, Dynamics
from pysim.config import SimConfig

SOLID_MOTOR_IGNITER_NAME = 'SolidMotorIgniter'

class VehicleComponents:
    def __init__(self, fcu: FcuSil, dynamics: Dynamics, config: SimConfig):
        self.fcu = fcu
        self.dynamics = dynamics
        self.config = config

        self.set_solid_motor_igniter_continuity(True)

        self.auto_ignite_ignition_packet_sent = False
        self.solid_motor_ignited = False
        self.solid_motor_burning = False
        self.ignition_time = 0.0

    def update(self, t: float, dt: float):
        if self.config.auto_ignite_solid_motor and self.fcu['vehicle_state'] == 'Armed' and not self.auto_ignite_ignition_packet_sent:
            self.fcu.send_ignite_solid_motor_packet()
            self.auto_ignite_ignition_packet_sent = True

        if not self.solid_motor_ignited and self.fcu['outputs'][SOLID_MOTOR_IGNITER_NAME]:
            self.try_ignite_solid_motor(t)

        if self.solid_motor_burning:
            thrust = self.config.thrust / self.config.vehicle_mass
            thrust_t = (t - self.ignition_time) / self.config.thrust_time
            thrust *= pow(math.cos(thrust_t * math.pi - math.pi / 2.0), 0.2)

            self.dynamics.motor_thrust = [0.0, thrust, 0.0]
            self.dynamics.landed = False # TODO Have dynamics figure this out on its own

            if t - self.ignition_time >= self.config.thrust_time:
                self.solid_motor_burning = False
                self.dynamics.motor_thrust = [0.0]*3

    def set_solid_motor_igniter_continuity(self, state: bool):
        self.fcu.set_output_continuity(SOLID_MOTOR_IGNITER_NAME, state)

    def try_ignite_solid_motor(self, t: float):
        if not self.solid_motor_ignited and self.fcu['output_continuities'][SOLID_MOTOR_IGNITER_NAME]:
            print(f'Solid motor ignited at {t}')
            self.solid_motor_ignited = True
            self.solid_motor_burning = True
            self.ignition_time = t

            self.set_solid_motor_igniter_continuity(False)
