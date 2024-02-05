
import software_in_loop as sil

class SilGlue:
    ecu: sil.EcuSil = None
    fcu: sil.FcuSil = None
    mission_ctrl: sil.MissionControl = None

    vehicle_dynamics: sil.SilVehicleDynamics = None
    fuel_tank_dynamics: sil.SilTankDynamics = None
    oxidizer_tank_dynamics: sil.SilTankDynamics = None
    igniter_dynamics: sil.SilIgniterDynamics = None

    def __init__(self):
        pass

    def update(self, dt: float):
        if self.ecu != None:
            if self.fuel_tank_dynamics != None:
                self.fuel_tank_dynamics.feed_valve_open = self.ecu['binary_valves']['FuelPress']
                self.fuel_tank_dynamics.vent_valve_open = self.ecu['binary_valves']['FuelVent']

            if self.oxidizer_tank_dynamics != None:
                self.oxidizer_tank_dynamics.feed_valve_open = self.ecu['binary_valves']['OxidizerPress']
                self.oxidizer_tank_dynamics.vent_valve_open = self.ecu['binary_valves']['OxidizerVent']

            if self.igniter_dynamics != None:
                self.igniter_dynamics.fuel_valve_open = self.ecu['binary_valves']['IgniterFuelMain']
                self.igniter_dynamics.oxidizer_valve_open = self.ecu['binary_valves']['IgniterGOxMain']
                self.igniter_dynamics.has_ignition_source = self.ecu['sparking']

        if self.igniter_dynamics != None:
            if self.fuel_tank_dynamics != None:
                self.igniter_dynamics.fuel_pressure_pa = self.fuel_tank_dynamics.tank_pressure_pa
                # TODO Update upstream tank outlet volume flow rate

            if self.oxidizer_tank_dynamics != None:
                self.igniter_dynamics.oxidizer_pressure_pa = self.oxidizer_tank_dynamics.tank_pressure_pa
                # TODO Update upstream tank outlet volume flow rate

