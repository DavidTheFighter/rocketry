
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
        self.test_allow_igniter_ignition = True

    def update(self, dt: float):
        if self.ecu != None:
            if self.fuel_tank_dynamics != None:
                self.fuel_tank_dynamics.feed_valve_open = self.ecu['binary_valves']['FuelPressValve']
                self.fuel_tank_dynamics.vent_valve_open = self.ecu['binary_valves']['FuelVentValve']

            if self.oxidizer_tank_dynamics != None:
                self.oxidizer_tank_dynamics.feed_valve_open = self.ecu['binary_valves']['OxidizerPressValve']
                self.oxidizer_tank_dynamics.vent_valve_open = self.ecu['binary_valves']['OxidizerVentValve']

            if self.igniter_dynamics != None:
                self.igniter_dynamics.fuel_valve_open = self.ecu['binary_valves']['IgniterFuelValve']
                self.igniter_dynamics.oxidizer_valve_open = self.ecu['binary_valves']['IgniterOxidizerValve']
                self.igniter_dynamics.has_ignition_source = self.ecu['sparking'] and self.test_allow_igniter_ignition

        if self.igniter_dynamics != None:
            if self.fuel_tank_dynamics != None:
                self.igniter_dynamics.fuel_pressure_pa = self.fuel_tank_dynamics.tank_pressure_pa
                # TODO Update upstream tank outlet volume flow rate

            if self.oxidizer_tank_dynamics != None:
                self.igniter_dynamics.oxidizer_pressure_pa = self.oxidizer_tank_dynamics.tank_pressure_pa
                # TODO Update upstream tank outlet volume flow rate

